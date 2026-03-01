use heck::ToSnakeCase;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use super::{ParsedField, ParsedItem, ParsedVariant};

impl ParsedItem {
	/// Builds the `tuple_len` body for structs and enums.
	pub(crate) fn tuple_len(&self) -> syn::Result<TokenStream2> {
		match &self.data {
			darling::ast::Data::Struct(fields) => {
				let terms = fields
					.iter()
					.enumerate()
					.map(|(field_index, field)| field.tuple_len(field_index))
					.collect::<Vec<_>>();

				if terms.is_empty() {
					Ok(quote! { 0usize })
				} else {
					Ok(quote! { 0usize #(+ #terms)* })
				}
			},
			darling::ast::Data::Enum(variants) => {
				let match_arms = variants
					.iter()
					.map(ParsedVariant::tuple_len)
					.collect::<syn::Result<Vec<_>>>()?;

				Ok(quote! {
					match self {
						#(#match_arms),*
					}
				})
			},
		}
	}

	/// Builds the tuple element serialization body for structs and enums.
	pub(crate) fn serialize(&self) -> syn::Result<TokenStream2> {
		match &self.data {
			darling::ast::Data::Struct(fields) => {
				let statements = fields
					.iter()
					.enumerate()
					.map(|(field_index, field)| field.serialize(field_index))
					.collect::<Vec<_>>();

				Ok(quote! {
					#(#statements)*
				})
			},
			darling::ast::Data::Enum(variants) => {
				let match_arms = variants
					.iter()
					.map(ParsedVariant::serialize)
					.collect::<syn::Result<Vec<_>>>()?;

				Ok(quote! {
					match self {
						#(#match_arms),*
					}
				})
			},
		}
	}
}

impl ParsedField {
	/// Returns `&self.<field>` access for named or tuple fields.
	pub(crate) fn access_field(&self, field_index: usize) -> TokenStream2 {
		if let Some(ref ident) = self.ident {
			return quote! { &self.#ident };
		}

		let index = syn::Index::from(field_index);
		quote! { &self.#index }
	}

	/// Returns this field's contribution to total tuple length.
	fn tuple_len(&self, field_index: usize) -> TokenStream2 {
		let field = self.access_field(field_index);

		if !self.flatten {
			return quote! { 1usize };
		}

		quote_spanned!(self.ty.span() => {
			::nvui_serde::SerializeTupleElements::tuple_len(#field)
		})
	}

	/// Returns serialization code for this field in a struct context.
	fn serialize(&self, field_index: usize) -> TokenStream2 {
		let field = self.access_field(field_index);

		if !self.flatten {
			return quote! { ::serde::ser::SerializeTuple::serialize_element(tuple, #field)?; };
		}

		quote_spanned!(self.ty.span() => {
			::nvui_serde::SerializeTupleElements::serialize_tuple_elements(#field, tuple)?;
		})
	}

	/// Returns serialization code for a matched variant value.
	fn serialize_variant_value(&self, value: &TokenStream2) -> TokenStream2 {
		if !self.flatten {
			return quote! { ::serde::ser::SerializeTuple::serialize_element(tuple, #value)?; };
		}

		quote_spanned!(self.ty.span() => {
			::nvui_serde::SerializeTupleElements::serialize_tuple_elements(#value, tuple)?;
		})
	}

	/// Returns a named match pattern and its tuple length term.
	fn named_pattern_and_term(&self, index: usize) -> syn::Result<(TokenStream2, TokenStream2)> {
		let field_ident = self
			.ident
			.as_ref()
			.ok_or_else(|| syn::Error::new_spanned(&self.ty, "expected named field identifier"))?;

		if !self.flatten {
			return Ok((quote! { #field_ident: _ }, quote! { 1usize }));
		}

		let binding_ident = format_ident!("field_{index}");

		Ok((
			quote! { #field_ident: #binding_ident },
			quote! { ::nvui_serde::SerializeTupleElements::tuple_len(#binding_ident) },
		))
	}

	/// Returns an unnamed match pattern and its tuple length term.
	fn unnamed_pattern_and_term(&self, index: usize) -> (TokenStream2, TokenStream2) {
		if !self.flatten {
			return (quote! { _ }, quote! { 1usize });
		}

		let binding_ident = format_ident!("field_{index}");
		(
			quote! { #binding_ident },
			quote! { ::nvui_serde::SerializeTupleElements::tuple_len(#binding_ident) },
		)
	}

	/// Returns named-pattern and value pair for variant serialization.
	fn named_pattern_and_value(&self) -> syn::Result<(TokenStream2, TokenStream2)> {
		let field_ident = self
			.ident
			.as_ref()
			.ok_or_else(|| syn::Error::new_spanned(&self.ty, "expected named field identifier"))?;
		Ok((quote! { #field_ident }, quote! { #field_ident }))
	}
}

impl ParsedVariant {
	/// Builds one `match` arm used by enum `tuple_len`.
	pub(crate) fn tuple_len(&self) -> syn::Result<TokenStream2> {
		let variant_ident = &self.ident;
		let (pattern, field_terms) = self.pattern_and_terms()?;

		let mut terms = vec![quote! { 1usize }];
		terms.extend(field_terms);

		Ok(quote! {
			Self::#variant_ident #pattern => { 0usize #(+ #terms)* }
		})
	}

	/// Builds one `match` arm used by enum serialization.
	pub(crate) fn serialize(&self) -> syn::Result<TokenStream2> {
		let variant_ident = &self.ident;
		let (pattern, values) = self.pattern_and_values()?;
		let variant_tag = self.variant_tag_tokens();

		let statements = std::iter::once(
			quote! { ::serde::ser::SerializeTuple::serialize_element(tuple, &(#variant_tag))?; },
		)
		.chain(
			self.fields
				.iter()
				.zip(values.iter())
				.map(|(field, value)| field.serialize_variant_value(value)),
		)
		.collect::<Vec<_>>();

		Ok(quote! {
			Self::#variant_ident #pattern => {
				#(#statements)*
			}
		})
	}

	/// Returns the variant tag token or default snake_case name.
	fn variant_tag_tokens(&self) -> TokenStream2 {
		if let Some(rename) = &self.rename {
			return quote! { #rename };
		}

		let default_name = self.ident.to_string().to_snake_case();
		quote! { #default_name }
	}

	/// Builds variant match pattern and length terms.
	fn pattern_and_terms(&self) -> syn::Result<(TokenStream2, Vec<TokenStream2>)> {
		match self.fields.style {
			darling::ast::Style::Unit => Ok((quote! {}, Vec::new())),
			darling::ast::Style::Struct => {
				let pairs = self
					.fields
					.iter()
					.enumerate()
					.map(|(index, field)| field.named_pattern_and_term(index))
					.collect::<syn::Result<Vec<_>>>()?;
				let (pattern_items, terms): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();

				Ok((quote! { { #(#pattern_items),* } }, terms))
			},
			darling::ast::Style::Tuple => {
				let (pattern_items, terms): (Vec<_>, Vec<_>) = self
					.fields
					.iter()
					.enumerate()
					.map(|(index, field)| field.unnamed_pattern_and_term(index))
					.unzip();

				Ok((quote! { ( #(#pattern_items),* ) }, terms))
			},
		}
	}

	/// Builds variant match pattern and bound values.
	fn pattern_and_values(&self) -> syn::Result<(TokenStream2, Vec<TokenStream2>)> {
		match self.fields.style {
			darling::ast::Style::Unit => Ok((quote! {}, Vec::new())),
			darling::ast::Style::Struct => {
				let pairs = self
					.fields
					.iter()
					.map(ParsedField::named_pattern_and_value)
					.collect::<syn::Result<Vec<_>>>()?;
				let (pattern_items, values): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();

				Ok((quote! { { #(#pattern_items),* } }, values))
			},
			darling::ast::Style::Tuple => {
				let binding_idents = (0..self.fields.len())
					.map(|index| format_ident!("__field_{index}"))
					.collect::<Vec<_>>();

				let pattern = quote! { ( #(#binding_idents),* ) };
				let bindings =
					binding_idents.into_iter().map(|binding| quote! { #binding }).collect();
				Ok((pattern, bindings))
			},
		}
	}
}
