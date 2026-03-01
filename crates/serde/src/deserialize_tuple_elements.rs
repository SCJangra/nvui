/// Describes a type that can be reconstructed from tuple elements.
///
/// This trait models deserialization from a tuple *fragment* rather than a
/// complete standalone value. Implementations are expected to consume only the
/// elements that belong to the value from an already-open
/// [`serde::de::SeqAccess`].
///
/// The derive macro [`crate::DeserializeTuple`] generates both this trait and
/// [`serde::Deserialize`] for the same type.
pub trait DeserializeTupleElements<'de>: Sized {
	/// Reconstructs a value by consuming its tuple elements from `seq`.
	fn deserialize_tuple_elements<A>(seq: &mut A) -> Result<Self, A::Error>
	where
		A: serde::de::SeqAccess<'de>;
}
