use darling::{FromDeriveInput, FromField};
use syn::{Ident, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(map), supports(struct_named))]
pub(crate) struct MapStructMacro {
	pub(crate) ident: Ident,
	pub(crate) generics: syn::Generics,
	pub(crate) data: darling::ast::Data<darling::util::Ignored, MapStructField>,
}

#[derive(Debug, FromField)]
pub(crate) struct MapStructField {
	pub(crate) ident: Option<Ident>,
	pub(crate) ty: Type,
}

#[derive(Debug)]
pub(crate) struct MapStruct {
	pub(crate) ident: Ident,
	pub(crate) generics: syn::Generics,
	pub(crate) fields: Vec<MapStructField>,
}

impl MapStructMacro {
	pub(crate) fn validate(self) -> darling::Result<MapStruct> {
		let mut err = darling::Error::accumulator();

		let darling::ast::Data::Struct(fields) = self.data else {
			err.push(
				darling::Error::custom("SerializeMap can only be derived for named structs")
					.with_span(&self.ident),
			);
			return err.finish().map(|_| unreachable!());
		};

		if fields.style != darling::ast::Style::Struct {
			err.push(
				darling::Error::custom("SerializeMap supports only named structs")
					.with_span(&self.ident),
			);
		}

		fields.iter().for_each(|field| {
			if field.ident.is_none() {
				err.push(
					darling::Error::custom("SerializeMap supports only named struct fields")
						.with_span(&field.ty),
				);
			}
		});

		err.finish().map(|_| MapStruct {
			ident: self.ident,
			generics: self.generics,
			fields: fields.fields,
		})
	}
}
