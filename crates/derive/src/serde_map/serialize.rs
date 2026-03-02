use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::spanned::Spanned;

use super::types::{MapStruct, MapStructField};

pub(super) fn impl_serialize_map(
	input: &DeriveInput,
	mut map: MapStruct,
) -> darling::Result<TokenStream> {
	map.add_serialize_trait_bounds();

	let ident = &map.ident;
	let (impl_generics, ty_generics, where_clause) = map.generics.split_for_impl();
	let serialize_body = map
		.build_serialize_map_body()
		.map_err(|error| darling::Error::custom(error.to_string()).with_span(input))?;

	Ok(quote! {
		impl #impl_generics ::serde::Serialize for #ident #ty_generics #where_clause {
			fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
			where
				S: ::serde::Serializer,
			{
				#serialize_body
			}
		}
	})
}

impl MapStruct {
	pub(crate) fn add_serialize_trait_bounds(&mut self) {
		let where_clause = self.generics.make_where_clause();
		for field in &self.fields {
			let ty = &field.ty;
			where_clause
				.predicates
				.push(syn::parse_quote_spanned!(ty.span()=> #ty: ::serde::Serialize));
		}
	}

	fn build_serialize_map_body(&self) -> syn::Result<TokenStream> {
		let entries = self
			.fields
			.iter()
			.map(MapStructField::build_serialize_entry)
			.collect::<syn::Result<Vec<_>>>()?;

		let field_count = self.fields.len();
		Ok(quote! {
			let mut map = serializer.serialize_map(Some(#field_count))?;
			#(#entries)*
			::serde::ser::SerializeMap::end(map)
		})
	}
}

impl MapStructField {
	fn build_serialize_entry(&self) -> syn::Result<TokenStream> {
		let field_ident = self
			.ident
			.as_ref()
			.ok_or_else(|| syn::Error::new_spanned(&self.ty, "expected named field identifier"))?;
		let key = field_ident.to_string();
		Ok(quote! {
			::serde::ser::SerializeMap::serialize_entry(&mut map, #key, &self.#field_ident)?;
		})
	}
}
