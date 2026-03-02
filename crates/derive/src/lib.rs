//! Derive implementations for tuple/map-oriented serde support.

use proc_macro::TokenStream;

mod serde_map;
mod serde_tuple;
mod serde_tuple_elements;
mod utils;

#[proc_macro_derive(SerializeTuple, attributes(tuple))]
/// Derives tuple-style `serde::Serialize` for enums.
///
/// Enum values are serialized as `[variant_tag, ...fields]`.
pub fn derive_serialize_tuple(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as syn::DeriveInput);
	serde_tuple::impl_serialize_tuple(&input)
		.unwrap_or_else(|error| error.write_errors())
		.into()
}

#[proc_macro_derive(DeserializeTuple, attributes(tuple))]
/// Derives tuple-style `serde::Deserialize` for enums.
///
/// Enum values are deserialized from `[variant_tag, ...fields]`.
pub fn derive_deserialize_tuple(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as syn::DeriveInput);
	serde_tuple::impl_deserialize_tuple(&input)
		.unwrap_or_else(|error| error.write_errors())
		.into()
}

#[proc_macro_derive(SerializeTupleElements, attributes(tuple))]
/// Derives `nvui_serde::SerializeTupleElements` for structs.
///
/// This derive does not implement `serde::Serialize`.
pub fn derive_serialize_tuple_elements(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as syn::DeriveInput);
	serde_tuple_elements::impl_serialize_tuple_elements(&input)
		.unwrap_or_else(|error| error.write_errors())
		.into()
}

#[proc_macro_derive(DeserializeTupleElements, attributes(tuple))]
/// Derives `nvui_serde::DeserializeTupleElements` for structs.
///
/// This derive does not implement `serde::Deserialize`.
pub fn derive_deserialize_tuple_elements(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as syn::DeriveInput);
	serde_tuple_elements::impl_deserialize_tuple_elements(&input)
		.unwrap_or_else(|error| error.write_errors())
		.into()
}

#[proc_macro_derive(SerializeMap, attributes(map))]
/// Derives map-style `serde::Serialize` for named structs.
pub fn derive_serialize_map(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as syn::DeriveInput);
	serde_map::impl_serialize_map(&input)
		.unwrap_or_else(|error| error.write_errors())
		.into()
}
