use heck::ToSnakeCase;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Lit, LitStr};

use super::{ParsedField, ParsedItem, ParsedVariant};

enum DeserializeVariantTag {
	String(LitStr),
	Unsigned(u64),
}

impl ParsedItem {
	/// Builds tuple-element deserialization body for structs and enums.
	pub(crate) fn deserialize(&self) -> syn::Result<TokenStream2> {
		match &self.data {
			darling::ast::Data::Struct(fields) => deserialize_struct(fields),
			darling::ast::Data::Enum(variants) => deserialize_enum(variants),
		}
	}
}

impl ParsedField {
	fn deserialize_expr(&self, field_index: usize) -> TokenStream2 {
		if self.flatten {
			let ty = &self.ty;
			return quote_spanned!(ty.span()=> {
				<#ty as ::nvui_serde::DeserializeTupleElements<'__de>>::deserialize_tuple_elements(seq)?
			});
		}

		quote! {
			seq
				.next_element()?
				.ok_or_else(|| ::serde::de::Error::invalid_length(#field_index, &"tuple with enough elements"))?
		}
	}

	fn deserialize_named_pair(
		&self,
		field_index: usize,
	) -> syn::Result<(TokenStream2, TokenStream2)> {
		let field_ident = self
			.ident
			.as_ref()
			.ok_or_else(|| syn::Error::new_spanned(&self.ty, "expected named field identifier"))?;
		Ok((quote! { #field_ident }, self.deserialize_expr(field_index)))
	}
}

impl ParsedVariant {
	fn deserialize_tag(&self) -> syn::Result<DeserializeVariantTag> {
		match &self.rename {
			Some(Lit::Str(value)) => Ok(DeserializeVariantTag::String(value.clone())),
			Some(Lit::Int(value)) => {
				let parsed = value.base10_parse::<u64>().map_err(|_| {
					syn::Error::new_spanned(
						value,
						"DeserializeTuple integer enum tag must fit into u64",
					)
				})?;
				Ok(DeserializeVariantTag::Unsigned(parsed))
			},
			Some(other) => Err(syn::Error::new_spanned(
				other,
				"DeserializeTuple requires #[tuple(rename = ...)] to be a string or integer literal",
			)),
			None => Ok(DeserializeVariantTag::String(LitStr::new(
				&self.ident.to_string().to_snake_case(),
				self.ident.span(),
			))),
		}
	}

	fn deserialize_constructor(&self, start_index: usize) -> syn::Result<TokenStream2> {
		let variant_ident = &self.ident;

		let (statements, constructor) = match self.fields.style {
			darling::ast::Style::Unit => (Vec::new(), quote! { Self::#variant_ident }),
			darling::ast::Style::Struct => {
				let pairs = self
					.fields
					.iter()
					.enumerate()
					.map(|(idx, field)| field.deserialize_named_pair(start_index + idx))
					.collect::<syn::Result<Vec<_>>>()?;

				let statements = pairs
					.iter()
					.map(|(name, expr)| quote! { let #name = #expr; })
					.collect::<Vec<_>>();
				let names = pairs.iter().map(|(name, _)| quote! { #name }).collect::<Vec<_>>();
				(statements, quote! { Self::#variant_ident { #(#names),* } })
			},
			darling::ast::Style::Tuple => {
				let bindings = (0..self.fields.len())
					.map(|index| format_ident!("__field_{index}"))
					.collect::<Vec<_>>();

				let statements = self
					.fields
					.iter()
					.enumerate()
					.map(|(idx, field)| {
						let binding = &bindings[idx];
						let expr = field.deserialize_expr(start_index + idx);
						quote! { let #binding = #expr; }
					})
					.collect::<Vec<_>>();

				(statements, quote! { Self::#variant_ident(#(#bindings),*) })
			},
		};

		Ok(quote! {
			#(#statements)*
			Ok(#constructor)
		})
	}
}

fn deserialize_struct(fields: &darling::ast::Fields<ParsedField>) -> syn::Result<TokenStream2> {
	match fields.style {
		darling::ast::Style::Unit => Ok(quote! { Ok(Self) }),
		darling::ast::Style::Struct => {
			let pairs = fields
				.iter()
				.enumerate()
				.map(|(index, field)| field.deserialize_named_pair(index))
				.collect::<syn::Result<Vec<_>>>()?;

			let statements = pairs
				.iter()
				.map(|(name, expr)| quote! { let #name = #expr; })
				.collect::<Vec<_>>();
			let names = pairs.iter().map(|(name, _)| quote! { #name }).collect::<Vec<_>>();

			Ok(quote! {
				#(#statements)*
				Ok(Self { #(#names),* })
			})
		},
		darling::ast::Style::Tuple => {
			let bindings = (0..fields.len())
				.map(|index| format_ident!("__field_{index}"))
				.collect::<Vec<_>>();

			let statements = fields
				.iter()
				.enumerate()
				.map(|(index, field)| {
					let binding = &bindings[index];
					let expr = field.deserialize_expr(index);
					quote! { let #binding = #expr; }
				})
				.collect::<Vec<_>>();

			Ok(quote! {
				#(#statements)*
				Ok(Self(#(#bindings),*))
			})
		},
	}
}

fn deserialize_enum(variants: &[ParsedVariant]) -> syn::Result<TokenStream2> {
	let mut string_arms = Vec::new();
	let mut valid_string_tags = Vec::new();
	let mut u64_arms = Vec::new();
	let mut i64_arms = Vec::new();
	let mut valid_numeric_tags = Vec::new();

	for variant in variants {
		let constructor = variant.deserialize_constructor(1)?;

		match variant.deserialize_tag()? {
			DeserializeVariantTag::String(tag) => {
				valid_string_tags.push(tag.clone());
				string_arms.push(quote! {
					#tag => {
						#constructor
					},
				});
			},
			DeserializeVariantTag::Unsigned(tag) => {
				valid_numeric_tags.push(tag.to_string());
				u64_arms.push(quote! {
					#tag => {
						#constructor
					},
				});

				if tag <= i64::MAX as u64 {
					let signed_tag = tag as i64;
					i64_arms.push(quote! {
						#signed_tag => {
							#constructor
						},
					});
				}
			},
		}
	}

	let expected_numeric = if valid_numeric_tags.is_empty() {
		String::from("<none>")
	} else {
		valid_numeric_tags.join(", ")
	};
	let expected_numeric_lit = LitStr::new(&expected_numeric, proc_macro2::Span::call_site());

	Ok(quote! {
		#[derive(::serde::Deserialize)]
		#[serde(untagged)]
		enum __NvuiEnumTag<'a> {
			Str(::std::borrow::Cow<'a, str>),
			I64(i64),
			U64(u64),
		}

		let __tag: __NvuiEnumTag<'__de> = seq
			.next_element()?
			.ok_or_else(|| ::serde::de::Error::invalid_length(0usize, &"enum tag"))?;

		match __tag {
			__NvuiEnumTag::Str(__value) => match __value.as_ref() {
				#(#string_arms)*
				_ => Err(::serde::de::Error::unknown_variant(__value.as_ref(), &[#(#valid_string_tags),*])),
			},
			__NvuiEnumTag::I64(__value) => match __value {
				#(#i64_arms)*
				_ => Err(::serde::de::Error::custom(::std::format!(
					"unknown numeric enum tag: {}; expected one of [{}]",
					__value,
					#expected_numeric_lit
				))),
			},
			__NvuiEnumTag::U64(__value) => match __value {
				#(#u64_arms)*
				_ => Err(::serde::de::Error::custom(::std::format!(
					"unknown numeric enum tag: {}; expected one of [{}]",
					__value,
					#expected_numeric_lit
				))),
			},
		}
	})
}
