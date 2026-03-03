mod tagged_enum;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_derive(DeserializeTaggedEnum, attributes(tagged_enum))]
pub fn derive_deserialize_tagged_enum(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as syn::DeriveInput);
	tagged_enum::impl_deserialize(input).unwrap_or_else(|e| e.write_errors()).into()
}
