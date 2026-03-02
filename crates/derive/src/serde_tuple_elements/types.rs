use darling::{FromDeriveInput, FromField};
use syn::{Ident, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(tuple), supports(struct_any))]
pub(crate) struct TupleStructMacro {
	pub(crate) ident: Ident,
	pub(crate) generics: syn::Generics,
	pub(crate) data: darling::ast::Data<darling::util::Ignored, TupleStructField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(tuple))]
pub(crate) struct TupleStructField {
	pub(crate) ident: Option<Ident>,
	pub(crate) ty: Type,

	#[darling(default)]
	pub(crate) flatten: bool,
}

#[derive(Debug)]
pub(crate) struct TupleStruct {
	pub(crate) ident: Ident,
	pub(crate) generics: syn::Generics,
	pub(crate) style: darling::ast::Style,
	pub(crate) fields: Vec<TupleStructField>,
}

impl TupleStructMacro {
	pub(crate) fn validate(self) -> darling::Result<TupleStruct> {
		let mut err = darling::Error::accumulator();

		let darling::ast::Data::Struct(fields) = self.data else {
			err.push(
				darling::Error::custom(
					"SerializeTupleElements/DeserializeTupleElements can only be derived for structs",
				)
				.with_span(&self.ident),
			);
			return err.finish().map(|_| unreachable!());
		};

		if fields.style == darling::ast::Style::Struct {
			fields.iter().for_each(|field| {
				if field.ident.is_none() {
					err.push(
						darling::Error::custom("expected named field identifier")
							.with_span(&field.ty),
					);
				}
			});
		}

		err.finish().map(|_| TupleStruct {
			ident: self.ident,
			generics: self.generics,
			style: fields.style,
			fields: fields.fields,
		})
	}
}
