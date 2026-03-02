use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn deserialize_impl_generics(generics: &syn::Generics) -> TokenStream {
	let mut impl_generics = generics.clone();
	impl_generics.params.insert(0, syn::parse_quote!('__de));
	let (impl_generics, _, _) = impl_generics.split_for_impl();
	quote! { #impl_generics }
}
