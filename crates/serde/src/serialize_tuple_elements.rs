/// Describes a type that can write itself as one or more tuple elements.
///
/// This trait models a tuple *fragment* rather than a full standalone value.
/// Implementations are expected to write only elements into an already-open
/// [`serde::ser::SerializeTuple`] and leave tuple finalization to the caller.
///
/// The derive macro [`crate::SerializeTuple`] generates both this trait and
/// [`serde::Serialize`] for the same type.
pub trait SerializeTupleElements {
	/// Returns the number of tuple elements this value writes.
	///
	/// For enums this is usually runtime-dependent because each variant can have
	/// a different number of payload elements.
	fn tuple_len(&self) -> usize;

	/// Writes this value's tuple elements into an existing tuple serializer.
	///
	/// This method should call `serialize_element` for normal fields and may call
	/// nested [`SerializeTupleElements::serialize_tuple_elements`] for flattened
	/// fields.
	fn serialize_tuple_elements<S>(&self, tuple: &mut S) -> Result<(), S::Error>
	where
		S: serde::ser::SerializeTuple;
}
