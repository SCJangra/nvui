use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use super::types::{TupleStruct, TupleStructField};

pub(super) fn impl_serialize_tuple_elements(
	mut tuple: TupleStruct,
) -> darling::Result<TokenStream> {
	tuple.add_serialize_trait_bounds();

	let ident = &tuple.ident;
	let (impl_generics, ty_generics, where_clause) = tuple.generics.split_for_impl();
	let tuple_len = tuple.build_tuple_len_body();
	let serialize_elements = tuple.build_serialize_elements_body();

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
	})
}

impl TupleStruct {
	pub(crate) fn add_serialize_trait_bounds(&mut self) {
		let where_clause = self.generics.make_where_clause();

		for field in &self.fields {
			let ty = &field.ty;
			let predicate = if field.flatten {
				syn::parse_quote_spanned!(ty.span()=> #ty: ::nvui_serde::SerializeTupleElements)
			} else {
				syn::parse_quote_spanned!(ty.span()=> #ty: ::serde::Serialize)
			};
			where_clause.predicates.push(predicate);
		}
	}

	fn build_tuple_len_body(&self) -> TokenStream {
		let terms = self
			.fields
			.iter()
			.enumerate()
			.map(|(field_index, field)| field.build_tuple_len_term(field_index))
			.collect::<Vec<_>>();

		if terms.is_empty() {
			quote! { 0usize }
		} else {
			quote! { 0usize #(+ #terms)* }
		}
	}

	fn build_serialize_elements_body(&self) -> TokenStream {
		let statements = self
			.fields
			.iter()
			.enumerate()
			.map(|(field_index, field)| field.build_serialize_statement(field_index))
			.collect::<Vec<_>>();

		quote! {
			#(#statements)*
		}
	}
}

impl TupleStructField {
	fn build_access_field(&self, field_index: usize) -> TokenStream {
		if let Some(ref ident) = self.ident {
			return quote! { &self.#ident };
		}

		let index = syn::Index::from(field_index);
		quote! { &self.#index }
	}

	fn build_tuple_len_term(&self, field_index: usize) -> TokenStream {
		let field = self.build_access_field(field_index);

		if !self.flatten {
			return quote! { 1usize };
		}

		quote_spanned!(self.ty.span() => {
			::nvui_serde::SerializeTupleElements::tuple_len(#field)
		})
	}

	fn build_serialize_statement(&self, field_index: usize) -> TokenStream {
		let field = self.build_access_field(field_index);

		if !self.flatten {
			return quote! { ::serde::ser::SerializeTuple::serialize_element(tuple, #field)?; };
		}

		quote_spanned!(self.ty.span() => {
			::nvui_serde::SerializeTupleElements::serialize_tuple_elements(#field, tuple)?;
		})
	}
}
