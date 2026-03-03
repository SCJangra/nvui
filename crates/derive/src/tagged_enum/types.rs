use darling::{FromDeriveInput, FromField, FromVariant, ast};
use syn::{Ident, Type};

use crate::utils;

#[derive(FromDeriveInput)]
#[darling(attributes(tagged_enum), supports(enum_named, enum_tuple))]
pub(crate) struct TaggedEnumInput {
	ident: Ident,
	data: ast::Data<EnumVariantInput, ()>,
}

pub(crate) struct TaggedEnum {
	pub(crate) ident: Ident,
	pub(crate) variants: Vec<EnumVariant>,
}

#[derive(FromVariant)]
#[darling(attributes(tagged_enum))]
pub(crate) struct EnumVariantInput {
	pub(crate) ident: Ident,
	pub(crate) fields: ast::Fields<EnumVariantFieldInput>,
}

pub(crate) struct EnumVariant {
	pub(crate) ident: Ident,
	pub(crate) fields: ast::Fields<EnumVariantField>,
}

#[derive(FromField)]
#[darling(attributes(tagged_enum))]
pub(crate) struct EnumVariantFieldInput {
	pub(crate) ident: Option<Ident>,
	pub(crate) ty: Type,

	#[darling(default)]
	pub(crate) flatten: bool,
}

pub(crate) struct EnumVariantField {
	pub(crate) ident: Option<Ident>,
	pub(crate) ty: Type,
	pub(crate) flatten: Option<Type>,
}

impl TryFrom<EnumVariantInput> for EnumVariant {
	type Error = darling::Error;

	fn try_from(value: EnumVariantInput) -> Result<Self, Self::Error> {
		let mut error = darling::Error::accumulator();

		let style = value.fields.style;
		let fields = value
			.fields
			.into_iter()
			.map(EnumVariantField::try_from)
			.filter_map(|result| result.inspect_err(|err| error.push(err.clone())).ok())
			.collect();

		let fields = ast::Fields::new(style, fields);

		error.finish().map(|()| Self { ident: value.ident, fields })
	}
}

impl TryFrom<EnumVariantFieldInput> for EnumVariantField {
	type Error = darling::Error;

	fn try_from(value: EnumVariantFieldInput) -> Result<Self, Self::Error> {
		let flatten = value.flatten.then(|| utils::vec_inner_type(&value.ty)).flatten();

		if value.flatten && flatten.is_none() {
			let error =
				darling::Error::custom("flattened fields must be Vec<T>").with_span(&value.ty);
			return Err(error);
		}

		Ok(Self { ident: value.ident, ty: value.ty, flatten })
	}
}

impl TaggedEnumInput {
	pub(crate) fn validate(self) -> darling::Result<TaggedEnum> {
		let mut error = darling::Error::accumulator();

		let variants = match self.data {
			ast::Data::Enum(variants) => variants,
			ast::Data::Struct(_) => unreachable!(),
		};

		let variants = variants
			.into_iter()
			.map(EnumVariant::try_from)
			.flat_map(|result| result.inspect_err(|err| error.push(err.clone())).ok())
			.collect::<Vec<_>>();

		let mut saw_other = false;

		variants.iter().for_each(|v| match v.validate() {
			Ok(val) if val == true => saw_other = true,
			Ok(_) => (),
			Err(err) => error.push(err),
		});

		if !saw_other {
			let msg = "tagged enum must define `Other { method: String, value: T }`";
			error.push(darling::Error::custom(msg).with_span(&self.ident))
		}

		error.finish().map(|()| TaggedEnum { ident: self.ident, variants })
	}
}

impl EnumVariant {
	fn validate(&self) -> darling::Result<bool> {
		if self.ident != "Other" {
			return Ok(false);
		}

		self.validate_other().map(|_| true)
	}

	fn validate_other(&self) -> darling::Result<()> {
		if self.fields.style != ast::Style::Struct {
			let msg = "`Other` must be a struct variant: `Other { method: String, value: T }`";
			return Err(darling::Error::custom(msg).with_span(&self.ident));
		}

		if self.fields.fields.len() != 2 {
			let msg = "`Other` must contain exactly two fields: `method` and `value`";
			return Err(darling::Error::custom(msg).with_span(&self.ident));
		}

		let method_field = self
			.fields
			.fields
			.iter()
			.find(|field| field.ident.as_ref().is_some_and(|ident| ident == "method"))
			.ok_or_else(|| {
				let msg = "`Other` must contain a `method: String` field";
				darling::Error::custom(msg).with_span(&self.ident)
			})?;

		if !utils::is_string_type(&method_field.ty) {
			let msg = "`Other.method` must have type `String`";
			return Err(darling::Error::custom(msg).with_span(&method_field.ty));
		}

		if method_field.flatten.is_some() {
			let msg = "`Other.method` cannot use `#[tagged_enum(flatten)]`";
			return Err(darling::Error::custom(msg).with_span(&method_field.ty));
		}

		let has_value = self
			.fields
			.fields
			.iter()
			.any(|field| field.ident.as_ref().is_some_and(|ident| ident == "value"));

		if !has_value {
			let msg = "`Other` must contain a `value` field";
			return Err(darling::Error::custom(msg).with_span(&self.ident));
		}

		Ok(())
	}
}
