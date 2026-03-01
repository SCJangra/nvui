mod deserialize;
mod serialize;

use darling::{FromDeriveInput, FromField, FromVariant};
use syn::spanned::Spanned;
use syn::{Ident, Lit, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(tuple), supports(struct_any, enum_any))]
pub struct ParsedItem {
	pub(crate) ident: Ident,
	pub(crate) generics: syn::Generics,
	pub(crate) data: darling::ast::Data<ParsedVariant, ParsedField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(tuple))]
pub struct ParsedField {
	pub(crate) ident: Option<Ident>,
	pub(crate) ty: Type,

	#[darling(default)]
	pub(crate) flatten: bool,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(tuple))]
pub struct ParsedVariant {
	pub(crate) ident: Ident,
	pub(crate) fields: darling::ast::Fields<ParsedField>,

	#[darling(default)]
	pub(crate) rename: Option<Lit>,
}

impl ParsedItem {
	/// Adds required trait bounds for all parsed field types.
	pub(crate) fn add_serialize_trait_bounds(&mut self) {
		let where_clause = self.generics.make_where_clause();

		let to_predicate = |field: &ParsedField| {
			let ty = &field.ty;
			if field.flatten {
				Some(
					syn::parse_quote_spanned!(ty.span()=> #ty: ::nvui_serde::SerializeTupleElements),
				)
			} else {
				Some(syn::parse_quote_spanned!(ty.span()=> #ty: ::serde::Serialize))
			}
		};

		match &self.data {
			darling::ast::Data::Struct(fields) => fields
				.iter()
				.filter_map(to_predicate)
				.for_each(|p| where_clause.predicates.push(p)),
			darling::ast::Data::Enum(variants) => variants
				.iter()
				.flat_map(|v| v.fields.iter())
				.filter_map(to_predicate)
				.for_each(|p| where_clause.predicates.push(p)),
		}
	}

	/// Adds required deserialize bounds for all parsed field types.
	pub(crate) fn add_deserialize_trait_bounds(&mut self) {
		let where_clause = self.generics.make_where_clause();

		let to_predicate = |field: &ParsedField| {
			let ty = &field.ty;
			if field.flatten {
				Some(
					syn::parse_quote_spanned!(ty.span()=> #ty: ::nvui_serde::DeserializeTupleElements<'__de>),
				)
			} else {
				Some(syn::parse_quote_spanned!(ty.span()=> #ty: ::serde::Deserialize<'__de>))
			}
		};

		match &self.data {
			darling::ast::Data::Struct(fields) => fields
				.iter()
				.filter_map(to_predicate)
				.for_each(|p| where_clause.predicates.push(p)),
			darling::ast::Data::Enum(variants) => variants
				.iter()
				.flat_map(|v| v.fields.iter())
				.filter_map(to_predicate)
				.for_each(|p| where_clause.predicates.push(p)),
		}
	}
}
