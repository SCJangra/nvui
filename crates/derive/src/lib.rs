//! Derive implementations for tuple-oriented serde traits.
//!
//! - `#[derive(SerializeTuple)]` expands into:
//!   1. `impl nvui_serde::SerializeTupleElements`
//!   2. `impl serde::Serialize`
//! - `#[derive(DeserializeTuple)]` expands into:
//!   1. `impl nvui_serde::DeserializeTupleElements`
//!   2. `impl serde::Deserialize`
//!
//! The generated `serde::Serialize` impl is intentionally thin and delegates
//! all tuple-shape logic to `SerializeTupleElements`:
//! - call `tuple_len()`
//! - open `serializer.serialize_tuple(tuple_len)`
//! - call `serialize_tuple_elements(...)`
//! - call `tuple.end()`
//!
//! ## Field and variant attributes
//! - `#[tuple(flatten)]` on a field: inline nested tuple elements.
//! - `#[tuple(rename = ...)]` on an enum variant: override the tag element.
//!
//! ## Enum encoding model
//! Enums are serialized as tuples where element `0` is the variant tag and the
//! rest are payload fields. Variant tags default to `snake_case` variant names.

mod parse;

use darling::FromDeriveInput;
use parse::ParsedItem;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::DeriveInput;
use syn::parse_macro_input;

#[proc_macro_derive(SerializeTuple, attributes(tuple))]
pub fn derive_serialize_tuple(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	match expand_serialize_derive(&input) {
		Ok(tokens) => tokens.into(),
		Err(error) => expand_serialize_error_fallback(&input, error).into(),
	}
}

#[proc_macro_derive(DeserializeTuple, attributes(tuple))]
pub fn derive_deserialize_tuple(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	match expand_deserialize_derive(&input) {
		Ok(tokens) => tokens.into(),
		Err(error) => expand_deserialize_error_fallback(&input, error).into(),
	}
}

fn expand_serialize_error_fallback(input: &DeriveInput, error: syn::Error) -> TokenStream2 {
	let ident = &input.ident;
	let generics = input.generics.clone();
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
	let compile_error = error.to_compile_error();

	quote! {
		#compile_error

		impl #impl_generics ::nvui_serde::SerializeTupleElements for #ident #ty_generics #where_clause {
			fn tuple_len(&self) -> usize {
				0usize
			}

			fn serialize_tuple_elements<S>(&self, _tuple: &mut S) -> ::core::result::Result<(), S::Error>
			where
				S: ::serde::ser::SerializeTuple,
			{
				Ok(())
			}
		}

		impl #impl_generics ::serde::Serialize for #ident #ty_generics #where_clause {
			fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
			where
				S: ::serde::Serializer,
			{
				let tuple = serializer.serialize_tuple(0usize)?;
				::serde::ser::SerializeTuple::end(tuple)
			}
		}
	}
}

fn expand_deserialize_error_fallback(input: &DeriveInput, error: syn::Error) -> TokenStream2 {
	let ident = &input.ident;
	let generics = input.generics.clone();
	let (_, ty_generics, where_clause) = generics.split_for_impl();
	let de_impl_generics = deserialize_impl_generics(&generics);
	let compile_error = error.to_compile_error();

	quote! {
		#compile_error

		impl #de_impl_generics ::nvui_serde::DeserializeTupleElements<'__de> for #ident #ty_generics #where_clause {
			fn deserialize_tuple_elements<A>(_seq: &mut A) -> ::core::result::Result<Self, A::Error>
			where
				A: ::serde::de::SeqAccess<'__de>,
			{
				::core::panic!("DeserializeTuple derive fallback")
			}
		}

		impl #de_impl_generics ::serde::Deserialize<'__de> for #ident #ty_generics #where_clause {
			fn deserialize<D>(_deserializer: D) -> ::core::result::Result<Self, D::Error>
			where
				D: ::serde::Deserializer<'__de>,
			{
				::core::panic!("DeserializeTuple derive fallback")
			}
		}
	}
}

fn expand_serialize_derive(input: &DeriveInput) -> syn::Result<TokenStream2> {
	let mut parsed = ParsedItem::from_derive_input(input)
		.map_err(|error| syn::Error::new_spanned(input, error.to_string()))?;

	parsed.add_serialize_trait_bounds();

	let ident = &parsed.ident;
	let (impl_generics, ty_generics, where_clause) = parsed.generics.split_for_impl();

	// Generate bodies separately to keep expansion readable and testable.
	let tuple_len = parsed.tuple_len()?;
	let serialize_elements = parsed.serialize()?;

	Ok(quote! {
		impl #impl_generics ::nvui_serde::SerializeTupleElements for #ident #ty_generics #where_clause {
			fn tuple_len(&self) -> usize {
				#tuple_len
			}

			fn serialize_tuple_elements<S>(&self, tuple: &mut S) -> ::core::result::Result<(), S::Error>
			where
				S: ::serde::ser::SerializeTuple,
			{
				#serialize_elements
				Ok(())
			}
		}

		impl #impl_generics ::serde::Serialize for #ident #ty_generics #where_clause {
			fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
			where
				S: ::serde::Serializer,
			{
				let tuple_len = <Self as ::nvui_serde::SerializeTupleElements>::tuple_len(self);
				let mut tuple = serializer.serialize_tuple(tuple_len)?;
				<Self as ::nvui_serde::SerializeTupleElements>::serialize_tuple_elements(self, &mut tuple)?;
				::serde::ser::SerializeTuple::end(tuple)
			}
		}
	})
}

fn expand_deserialize_derive(input: &DeriveInput) -> syn::Result<TokenStream2> {
	let mut parsed = ParsedItem::from_derive_input(input)
		.map_err(|error| syn::Error::new_spanned(input, error.to_string()))?;

	parsed.add_deserialize_trait_bounds();

	let ident = &parsed.ident;
	let visitor_ident = format_ident!("__NvuiTupleVisitorFor{}", ident);
	let (impl_generics, ty_generics, where_clause) = parsed.generics.split_for_impl();
	let de_impl_generics = deserialize_impl_generics(&parsed.generics);

	let deserialize_elements = parsed.deserialize()?;

	Ok(quote! {
		impl #de_impl_generics ::nvui_serde::DeserializeTupleElements<'__de> for #ident #ty_generics #where_clause {
			fn deserialize_tuple_elements<A>(seq: &mut A) -> ::core::result::Result<Self, A::Error>
			where
				A: ::serde::de::SeqAccess<'__de>,
			{
				#deserialize_elements
			}
		}

		struct #visitor_ident #impl_generics (::core::marker::PhantomData<fn() -> #ident #ty_generics>);

		impl #de_impl_generics ::serde::de::Visitor<'__de> for #visitor_ident #ty_generics #where_clause {
			type Value = #ident #ty_generics;

			fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
				formatter.write_str("a tuple")
			}

			fn visit_seq<A>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error>
			where
				A: ::serde::de::SeqAccess<'__de>,
			{
				let value = <#ident #ty_generics as ::nvui_serde::DeserializeTupleElements<'__de>>::deserialize_tuple_elements(&mut seq)?;
				if ::core::option::Option::is_some(&seq.next_element::<::serde::de::IgnoredAny>()?) {
					return Err(::serde::de::Error::custom("unexpected extra tuple elements"));
				}
				Ok(value)
			}
		}

		impl #de_impl_generics ::serde::Deserialize<'__de> for #ident #ty_generics #where_clause {
			fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
			where
				D: ::serde::Deserializer<'__de>,
			{
				deserializer.deserialize_seq(#visitor_ident(::core::marker::PhantomData))
			}
		}
	})
}

fn deserialize_impl_generics(generics: &syn::Generics) -> TokenStream2 {
	let mut impl_generics = generics.clone();
	impl_generics.params.insert(0, syn::parse_quote!('__de));
	let (impl_generics, _, _) = impl_generics.split_for_impl();
	quote! { #impl_generics }
}
