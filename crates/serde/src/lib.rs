//! Tuple-oriented serialization support for `nvui`.
//!
//! This crate exposes:
//! - [`SerializeTupleElements`], a trait for writing tuple fragments.
//! - [`SerializeTuple`], an enum-only derive macro that implements both
//!   [`SerializeTupleElements`] and [`serde::Serialize`].
//! - [`SerializeTupleElements`], a struct-only derive macro for tuple fragments.
//! - [`DeserializeTupleElements`], a trait for reading tuple fragments.
//! - [`DeserializeTuple`], an enum-only derive macro that implements both
//!   [`DeserializeTupleElements`] and [`serde::Deserialize`].
//! - [`DeserializeTupleElements`], a struct-only derive macro for tuple fragments.
//! - [`SerializeMap`], a named-struct-only derive macro that forces map encoding.
//!
//! ## Derive behavior
//! - **Tuple enums:** first tuple element is a variant tag, followed by payload
//!   fields in declaration order.
//! - **Default enum tag:** `snake_case` variant name.
//! - **Variant rename:** `#[tuple(rename = ...)]`, where `...` is a literal
//!   (e.g. `"my_tag"`, `7`).
//! - **Flatten field:** `#[tuple(flatten)]` to inline nested
//!   [`SerializeTupleElements`].
mod deserialize_tuple_elements;
mod serialize_tuple_elements;

pub use deserialize_tuple_elements::DeserializeTupleElements;
pub use nvui_derive::DeserializeTuple;
pub use nvui_derive::DeserializeTupleElements;
pub use nvui_derive::SerializeMap;
pub use nvui_derive::SerializeTuple;
pub use nvui_derive::SerializeTupleElements;
pub use serialize_tuple_elements::SerializeTupleElements;

extern crate self as nvui_serde;
