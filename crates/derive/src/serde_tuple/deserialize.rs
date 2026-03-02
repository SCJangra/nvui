use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, LitStr};

use crate::utils;

use super::types::{TupleEnum, TupleEnumField, TupleVariant, VariantTag};

enum DeserializeVariantTag {
	String(LitStr),
	Unsigned(u64),
}

pub(super) fn impl_deserialize_tuple(
	input: &DeriveInput,
	mut tuple: TupleEnum,
) -> darling::Result<TokenStream> {
	tuple.add_deserialize_trait_bounds();

	let ident = &tuple.ident;
	let visitor_ident = format_ident!("__NvuiTupleVisitorFor{}", ident);
	let (impl_generics, ty_generics, where_clause) = tuple.generics.split_for_impl();
	let de_impl_generics = utils::deserialize_impl_generics(&tuple.generics);
	let deserialize_elements = tuple
		.build_deserialize_body()
		.map_err(|error| darling::Error::custom(error.to_string()).with_span(input))?;

	Ok(quote! {
		impl #de_impl_generics ::nvui_serde::DeserializeTupleElements<'__de> for #ident #ty_generics #where_clause {
			fn deserialize_tuple_elements<A>(seq: &mut A) -> ::core::result::Result<Self, A::Error>
			where
				A: ::serde::de::SeqAccess<'__de>,
			{
				#deserialize_elements
			}
		}

		struct #visitor_ident #impl_generics (::core::marker::PhantomData<fn() -> #ident #ty_generics>);

		impl #de_impl_generics ::serde::de::Visitor<'__de> for #visitor_ident #ty_generics #where_clause {
			type Value = #ident #ty_generics;

			fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				formatter.write_str("a tuple")
			}

			fn visit_seq<A>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error>
			where
				A: ::serde::de::SeqAccess<'__de>,
			{
				let value = <#ident #ty_generics as ::nvui_serde::DeserializeTupleElements<'__de>>::deserialize_tuple_elements(&mut seq)?;
				if ::core::option::Option::is_some(&seq.next_element::<::serde::de::IgnoredAny>()?) {
					return Err(::serde::de::Error::custom("unexpected extra tuple elements"));
				}
				Ok(value)
			}
		}

		impl #de_impl_generics ::serde::Deserialize<'__de> for #ident #ty_generics #where_clause {
			fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
			where
				D: ::serde::Deserializer<'__de>,
			{
				deserializer.deserialize_seq(#visitor_ident(::core::marker::PhantomData))
			}
		}
	})
}

impl TupleEnum {
	pub(crate) fn add_deserialize_trait_bounds(&mut self) {
		let where_clause = self.generics.make_where_clause();

		for field in self.variants.iter().flat_map(|variant| variant.fields.iter()) {
			let ty = &field.ty;
			let predicate = if field.flatten {
				syn::parse_quote_spanned!(ty.span()=> #ty: ::nvui_serde::DeserializeTupleElements<'__de>)
			} else {
				syn::parse_quote_spanned!(ty.span()=> #ty: ::serde::Deserialize<'__de>)
			};
			where_clause.predicates.push(predicate);
		}
	}

	fn build_deserialize_body(&self) -> syn::Result<TokenStream> {
		let mut string_arms = Vec::new();
		let mut valid_string_tags = Vec::new();
		let mut u64_arms = Vec::new();
		let mut i64_arms = Vec::new();
		let mut valid_numeric_tags = Vec::new();

		for variant in &self.variants {
			let constructor = variant.build_deserialize_constructor(1)?;

			match variant.build_deserialize_tag() {
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
}

impl TupleVariant {
	fn build_deserialize_tag(&self) -> DeserializeVariantTag {
		match &self.tag {
			VariantTag::String(value) => {
				DeserializeVariantTag::String(LitStr::new(value, self.ident.span()))
			},
			VariantTag::Unsigned(value) => DeserializeVariantTag::Unsigned(*value),
		}
	}

	fn build_deserialize_constructor(&self, start_index: usize) -> syn::Result<TokenStream> {
		let variant_ident = &self.ident;

		let (statements, constructor) = match self.style {
			darling::ast::Style::Unit => (Vec::new(), quote! { Self::#variant_ident }),
			darling::ast::Style::Struct => {
				let pairs = self
					.fields
					.iter()
					.enumerate()
					.map(|(idx, field)| field.build_deserialize_named_pair(start_index + idx))
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
						let expr = field.build_deserialize_expr(start_index + idx);
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

impl TupleEnumField {
	fn build_deserialize_expr(&self, field_index: usize) -> TokenStream {
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

	fn build_deserialize_named_pair(
		&self,
		field_index: usize,
	) -> syn::Result<(TokenStream, TokenStream)> {
		let field_ident = self
			.ident
			.as_ref()
			.ok_or_else(|| syn::Error::new_spanned(&self.ty, "expected named field identifier"))?;
		Ok((quote! { #field_ident }, self.build_deserialize_expr(field_index)))
	}
}
