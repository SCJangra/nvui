use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::DeriveInput;
use syn::spanned::Spanned;

use crate::utils;

use super::types::{TupleStruct, TupleStructField};

pub(super) fn impl_deserialize_tuple_elements(
	input: &DeriveInput,
	mut tuple: TupleStruct,
) -> darling::Result<TokenStream> {
	tuple.add_deserialize_trait_bounds();

	let ident = &tuple.ident;
	let (_, ty_generics, where_clause) = tuple.generics.split_for_impl();
	let de_impl_generics = utils::deserialize_impl_generics(&tuple.generics);
	let deserialize_elements = tuple
		.build_deserialize_elements_body()
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
	})
}

impl TupleStruct {
	pub(crate) fn add_deserialize_trait_bounds(&mut self) {
		let where_clause = self.generics.make_where_clause();

		for field in &self.fields {
			let ty = &field.ty;
			let predicate = if field.flatten {
				syn::parse_quote_spanned!(ty.span()=> #ty: ::nvui_serde::DeserializeTupleElements<'__de>)
			} else {
				syn::parse_quote_spanned!(ty.span()=> #ty: ::serde::Deserialize<'__de>)
			};
			where_clause.predicates.push(predicate);
		}
	}

	fn build_deserialize_elements_body(&self) -> syn::Result<TokenStream> {
		match self.style {
			darling::ast::Style::Unit => Ok(quote! { Ok(Self) }),
			darling::ast::Style::Struct => {
				let pairs = self
					.fields
					.iter()
					.enumerate()
					.map(|(index, field)| field.build_deserialize_named_pair(index))
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
				let bindings = (0..self.fields.len())
					.map(|index| format_ident!("__field_{index}"))
					.collect::<Vec<_>>();

				let statements = self
					.fields
					.iter()
					.enumerate()
					.map(|(index, field)| {
						let binding = &bindings[index];
						let expr = field.build_deserialize_expr(index);
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
}

impl TupleStructField {
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
