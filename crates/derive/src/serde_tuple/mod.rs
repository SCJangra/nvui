mod deserialize;
mod serialize;
mod types;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub(crate) fn impl_serialize_tuple(input: &DeriveInput) -> darling::Result<TokenStream> {
	let tuple = types::TupleEnumMacro::from_derive_input(input)?.validate()?;
	serialize::impl_serialize_tuple(input, tuple)
}

pub(crate) fn impl_deserialize_tuple(input: &DeriveInput) -> darling::Result<TokenStream> {
	let tuple = types::TupleEnumMacro::from_derive_input(input)?.validate()?;
	deserialize::impl_deserialize_tuple(input, tuple)
}
