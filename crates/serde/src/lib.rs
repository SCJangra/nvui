//! Tuple-oriented serialization support for `nvui`.
//!
//! This crate exposes:
//! - [`SerializeTupleElements`], a trait for writing tuple fragments.
//! - [`SerializeTuple`], a derive macro that implements both
//!   [`SerializeTupleElements`] and [`serde::Serialize`].
//! - [`DeserializeTupleElements`], a trait for reading tuple fragments.
//! - [`DeserializeTuple`], a derive macro that implements both
//!   [`DeserializeTupleElements`] and [`serde::Deserialize`].
//!
//! ## Derive behavior
//! - **Structs / tuple structs:** fields are serialized in declaration order.
//! - **Enums:** first tuple element is a variant tag, followed by payload
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
pub use nvui_derive::SerializeTuple;
pub use serialize_tuple_elements::SerializeTupleElements;

extern crate self as nvui_serde;
