use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::DeriveInput;
use syn::spanned::Spanned;

use super::types::{TupleEnum, TupleEnumField, TupleVariant, VariantTag};

pub(super) fn impl_serialize_tuple(
	input: &DeriveInput,
	mut tuple: TupleEnum,
) -> darling::Result<TokenStream> {
	tuple.add_serialize_trait_bounds();

	let ident = &tuple.ident;
	let (impl_generics, ty_generics, where_clause) = tuple.generics.split_for_impl();
	let tuple_len = tuple
		.build_tuple_len_body()
		.map_err(|error| darling::Error::custom(error.to_string()).with_span(input))?;
	let serialize_elements = tuple
		.build_serialize_elements_body()
		.map_err(|error| darling::Error::custom(error.to_string()).with_span(input))?;

	Ok(quote! {
		impl #impl_generics ::nvui_serde::SerializeTupleElements for #ident #ty_generics #where_clause {
			fn tuple_len(&self) -> usize {
				#tuple_len
			}

			fn serialize_tuple_elements<S>(&self, tuple: &mut S) -> ::core::result::Result<(), S::Error>
			where
				S: ::serde::ser::SerializeTuple,
			{
				#serialize_elements
				Ok(())
			}
		}

		impl #impl_generics ::serde::Serialize for #ident #ty_generics #where_clause {
			fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
			where
				S: ::serde::Serializer,
			{
				let tuple_len = <Self as ::nvui_serde::SerializeTupleElements>::tuple_len(self);
				let mut tuple = serializer.serialize_tuple(tuple_len)?;
				<Self as ::nvui_serde::SerializeTupleElements>::serialize_tuple_elements(self, &mut tuple)?;
				::serde::ser::SerializeTuple::end(tuple)
			}
		}
	})
}

impl TupleEnum {
	pub(crate) fn add_serialize_trait_bounds(&mut self) {
		let where_clause = self.generics.make_where_clause();

		for field in self.variants.iter().flat_map(|variant| variant.fields.iter()) {
			let ty = &field.ty;
			let predicate = if field.flatten {
				syn::parse_quote_spanned!(ty.span()=> #ty: ::nvui_serde::SerializeTupleElements)
			} else {
				syn::parse_quote_spanned!(ty.span()=> #ty: ::serde::Serialize)
			};
			where_clause.predicates.push(predicate);
		}
	}

	fn build_tuple_len_body(&self) -> syn::Result<TokenStream> {
		let arms = self
			.variants
			.iter()
			.map(TupleVariant::build_tuple_len_arm)
			.collect::<syn::Result<Vec<_>>>()?;

		Ok(quote! {
			match self {
				#(#arms),*
			}
		})
	}

	fn build_serialize_elements_body(&self) -> syn::Result<TokenStream> {
		let arms = self
			.variants
			.iter()
			.map(TupleVariant::build_serialize_arm)
			.collect::<syn::Result<Vec<_>>>()?;

		Ok(quote! {
			match self {
				#(#arms),*
			}
		})
	}
}

impl TupleVariant {
	fn build_tuple_len_arm(&self) -> syn::Result<TokenStream> {
		let variant_ident = &self.ident;
		let (pattern, field_terms) = self.build_pattern_and_terms()?;

		let mut terms = vec![quote! { 1usize }];
		terms.extend(field_terms);

		Ok(quote! {
			Self::#variant_ident #pattern => { 0usize #(+ #terms)* }
		})
	}

	fn build_serialize_arm(&self) -> syn::Result<TokenStream> {
		let variant_ident = &self.ident;
		let (pattern, values) = self.build_pattern_and_values()?;
		let variant_tag = self.build_variant_tag_tokens();

		let statements = std::iter::once(
			quote! { ::serde::ser::SerializeTuple::serialize_element(tuple, &(#variant_tag))?; },
		)
		.chain(
			self.fields
				.iter()
				.zip(values.iter())
				.map(|(field, value)| field.build_serialize_variant_value(value)),
		)
		.collect::<Vec<_>>();

		Ok(quote! {
			Self::#variant_ident #pattern => {
				#(#statements)*
			}
		})
	}

	fn build_variant_tag_tokens(&self) -> TokenStream {
		match &self.tag {
			VariantTag::String(tag) => quote! { #tag },
			VariantTag::Unsigned(tag) => quote! { #tag },
		}
	}

	fn build_pattern_and_terms(&self) -> syn::Result<(TokenStream, Vec<TokenStream>)> {
		match self.style {
			darling::ast::Style::Unit => Ok((quote! {}, Vec::new())),
			darling::ast::Style::Struct => {
				let pairs = self
					.fields
					.iter()
					.enumerate()
					.map(|(index, field)| field.build_named_pattern_and_term(index))
					.collect::<syn::Result<Vec<_>>>()?;
				let (pattern_items, terms): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();

				Ok((quote! { { #(#pattern_items),* } }, terms))
			},
			darling::ast::Style::Tuple => {
				let (pattern_items, terms): (Vec<_>, Vec<_>) = self
					.fields
					.iter()
					.enumerate()
					.map(|(index, field)| field.build_unnamed_pattern_and_term(index))
					.unzip();

				Ok((quote! { ( #(#pattern_items),* ) }, terms))
			},
		}
	}

	fn build_pattern_and_values(&self) -> syn::Result<(TokenStream, Vec<TokenStream>)> {
		match self.style {
			darling::ast::Style::Unit => Ok((quote! {}, Vec::new())),
			darling::ast::Style::Struct => {
				let pairs = self
					.fields
					.iter()
					.map(TupleEnumField::build_named_pattern_and_value)
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

impl TupleEnumField {
	fn build_serialize_variant_value(&self, value: &TokenStream) -> TokenStream {
		if !self.flatten {
			return quote! { ::serde::ser::SerializeTuple::serialize_element(tuple, #value)?; };
		}

		quote_spanned!(self.ty.span() => {
			::nvui_serde::SerializeTupleElements::serialize_tuple_elements(#value, tuple)?;
		})
	}

	fn build_named_pattern_and_term(
		&self,
		index: usize,
	) -> syn::Result<(TokenStream, TokenStream)> {
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

	fn build_unnamed_pattern_and_term(&self, index: usize) -> (TokenStream, TokenStream) {
		if !self.flatten {
			return (quote! { _ }, quote! { 1usize });
		}

		let binding_ident = format_ident!("field_{index}");
		(
			quote! { #binding_ident },
			quote! { ::nvui_serde::SerializeTupleElements::tuple_len(#binding_ident) },
		)
	}

	fn build_named_pattern_and_value(&self) -> syn::Result<(TokenStream, TokenStream)> {
		let field_ident = self
			.ident
			.as_ref()
			.ok_or_else(|| syn::Error::new_spanned(&self.ty, "expected named field identifier"))?;
		Ok((quote! { #field_ident }, quote! { #field_ident }))
	}
}
