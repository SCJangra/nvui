mod deserialize;
mod serialize;
mod types;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub(crate) fn impl_serialize_map(input: &DeriveInput) -> darling::Result<TokenStream> {
	let map = types::MapStructMacro::from_derive_input(input)?.validate()?;
	serialize::impl_serialize_map(input, map)
}
