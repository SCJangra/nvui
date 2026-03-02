use std::collections::HashSet;

use darling::{FromDeriveInput, FromField, FromVariant};
use heck::ToSnakeCase;
use syn::{Ident, Lit, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(tuple), supports(enum_any))]
pub(crate) struct TupleEnumMacro {
	pub(crate) ident: Ident,
	pub(crate) generics: syn::Generics,
	pub(crate) data: darling::ast::Data<TupleVariantMacro, darling::util::Ignored>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(tuple))]
pub(crate) struct TupleVariantMacro {
	pub(crate) ident: Ident,
	pub(crate) fields: darling::ast::Fields<TupleEnumField>,

	#[darling(default)]
	pub(crate) rename: Option<Lit>,
}

#[derive(Debug, FromField)]
#[darling(attributes(tuple))]
pub(crate) struct TupleEnumField {
	pub(crate) ident: Option<Ident>,
	pub(crate) ty: Type,

	#[darling(default)]
	pub(crate) flatten: bool,
}

#[derive(Debug)]
pub(crate) struct TupleEnum {
	pub(crate) ident: Ident,
	pub(crate) generics: syn::Generics,
	pub(crate) variants: Vec<TupleVariant>,
}

#[derive(Debug)]
pub(crate) struct TupleVariant {
	pub(crate) ident: Ident,
	pub(crate) style: darling::ast::Style,
	pub(crate) fields: Vec<TupleEnumField>,
	pub(crate) tag: VariantTag,
}

#[derive(Debug, Clone)]
pub(crate) enum VariantTag {
	String(String),
	Unsigned(u64),
}

impl TupleEnumMacro {
	pub(crate) fn validate(self) -> darling::Result<TupleEnum> {
		let mut err = darling::Error::accumulator();

		let darling::ast::Data::Enum(variants) = self.data else {
			err.push(
				darling::Error::custom(
					"SerializeTuple/DeserializeTuple can only be derived for enums",
				)
				.with_span(&self.ident),
			);
			return err.finish().map(|_| unreachable!());
		};

		let mut seen_string_tags = HashSet::new();
		let mut seen_numeric_tags = HashSet::new();

		let variants = variants
			.into_iter()
			.map(|variant| {
				validate_tuple_variant(variant, &mut seen_string_tags, &mut seen_numeric_tags)
			})
			.flat_map(|variant| variant.map_err(|error| err.push(error)))
			.collect();

		err.finish()
			.map(|_| TupleEnum { ident: self.ident, generics: self.generics, variants })
	}
}

fn validate_variant_tag(variant: &TupleVariantMacro) -> darling::Result<VariantTag> {
	match &variant.rename {
		Some(Lit::Str(value)) => Ok(VariantTag::String(value.value())),
		Some(Lit::Int(value)) => {
			let parsed = value.base10_parse::<u64>().map_err(|_| {
				darling::Error::custom("tuple enum integer tag must fit into u64").with_span(value)
			})?;
			Ok(VariantTag::Unsigned(parsed))
		},
		Some(other) => Err(darling::Error::custom(
			"#[tuple(rename = ...)] must be a string or integer literal",
		)
		.with_span(other)),
		None => Ok(VariantTag::String(variant.ident.to_string().to_snake_case())),
	}
}

fn validate_tuple_variant(
	variant: TupleVariantMacro,
	seen_string_tags: &mut HashSet<String>,
	seen_numeric_tags: &mut HashSet<u64>,
) -> darling::Result<TupleVariant> {
	let mut err = darling::Error::accumulator();

	let tag = match validate_variant_tag(&variant) {
		Ok(tag) => tag,
		Err(error) => {
			err.push(error);
			VariantTag::String(variant.ident.to_string().to_snake_case())
		},
	};

	match &tag {
		VariantTag::String(value) => {
			if !seen_string_tags.insert(value.clone()) {
				err.push(
					darling::Error::custom(format!("duplicate tuple enum string tag: {value}"))
						.with_span(&variant.ident),
				);
			}
		},
		VariantTag::Unsigned(value) => {
			if !seen_numeric_tags.insert(*value) {
				err.push(
					darling::Error::custom(format!("duplicate tuple enum numeric tag: {value}"))
						.with_span(&variant.ident),
				);
			}
		},
	}

	if variant.fields.style == darling::ast::Style::Struct {
		variant.fields.iter().for_each(|field| {
			if field.ident.is_none() {
				err.push(
					darling::Error::custom("expected named field identifier").with_span(&field.ty),
				);
			}
		});
	}

	err.finish().map(|_| TupleVariant {
		ident: variant.ident,
		style: variant.fields.style,
		fields: variant.fields.fields,
		tag,
	})
}
