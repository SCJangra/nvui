use nvui_serde::{
	DeserializeTuple, DeserializeTupleElements, SerializeMap, SerializeTuple,
	SerializeTupleElements,
};
use serde::Deserializer;

#[derive(Debug, Clone, PartialEq, SerializeTupleElements, DeserializeTupleElements)]
struct OrdinaryStruct {
	a: u32,
	b: String,
	c: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, SerializeTupleElements, DeserializeTupleElements)]
struct StructWithFlattenedFields {
	a: u32,
	#[tuple(flatten)]
	ordinary: OrdinaryStruct,
}

#[derive(Debug, Clone, PartialEq, SerializeTuple, DeserializeTuple)]
enum StructCarrier {
	#[tuple(rename = "ordinary")]
	Ordinary(#[tuple(flatten)] OrdinaryStruct),
	#[tuple(rename = "flattened")]
	Flattened(#[tuple(flatten)] StructWithFlattenedFields),
}

#[derive(Debug, Clone, PartialEq, SerializeTuple, DeserializeTuple)]
enum RandomEnum {
	First,
	Second(u64, String),
	#[tuple(rename = "random_rename")]
	Third {
		a: u64,
		b: u64,
		c: u64,
	},
}

#[derive(Debug, Clone, PartialEq, SerializeTuple, DeserializeTuple)]
enum NumericallyTaggedEnum {
	#[tuple(rename = 0)]
	Connected,
	#[tuple(rename = 1)]
	Disconnected(u64),
}

#[derive(Debug, SerializeMap)]
struct ForcedMapStruct {
	alpha: u32,
	beta: String,
}

fn sample_ordinary_struct() -> OrdinaryStruct {
	OrdinaryStruct {
		a: 10,
		b: String::from("random string"),
		c: vec![String::from("another random string"), String::from("a third random string")],
	}
}

fn sample_flattened_struct() -> StructWithFlattenedFields {
	StructWithFlattenedFields { a: 20, ordinary: sample_ordinary_struct() }
}

#[test]
fn roundtrip_struct_tuple_elements_through_enum() {
	let ordinary = sample_ordinary_struct();
	let ordinary_carrier = StructCarrier::Ordinary(ordinary.clone());
	let ordinary_json = serde_json::to_value(&ordinary_carrier).unwrap();
	let ordinary_back: StructCarrier = serde_json::from_value(ordinary_json.clone()).unwrap();

	assert_eq!(ordinary.tuple_len(), 3);
	assert_eq!(
		ordinary_json,
		serde_json::json!([
			"ordinary",
			10,
			"random string",
			["another random string", "a third random string"]
		])
	);
	assert_eq!(ordinary_back, ordinary_carrier);

	let flattened = sample_flattened_struct();
	let flattened_carrier = StructCarrier::Flattened(flattened.clone());
	let flattened_json = serde_json::to_value(&flattened_carrier).unwrap();
	let flattened_back: StructCarrier = serde_json::from_value(flattened_json.clone()).unwrap();

	assert_eq!(flattened.tuple_len(), 4);
	assert_eq!(
		flattened_json,
		serde_json::json!([
			"flattened",
			20,
			10,
			"random string",
			["another random string", "a third random string"]
		])
	);
	assert_eq!(flattened_back, flattened_carrier);
}

#[test]
fn roundtrip_enum_variants() {
	let first = RandomEnum::First;
	let first_json = serde_json::to_value(&first).unwrap();
	let first_back: RandomEnum = serde_json::from_value(first_json.clone()).unwrap();
	assert_eq!(first_json, serde_json::json!(["first"]));
	assert_eq!(first_back, first);

	let second = RandomEnum::Second(3, String::from("text"));
	let second_json = serde_json::to_value(&second).unwrap();
	let second_back: RandomEnum = serde_json::from_value(second_json.clone()).unwrap();
	assert_eq!(second_json, serde_json::json!(["second", 3, "text"]));
	assert_eq!(second_back, second);

	let third = RandomEnum::Third { a: 2, b: 80, c: 24 };
	let third_json = serde_json::to_value(&third).unwrap();
	let third_back: RandomEnum = serde_json::from_value(third_json.clone()).unwrap();
	assert_eq!(third_json, serde_json::json!(["random_rename", 2, 80, 24]));
	assert_eq!(third_back, third);
}

#[test]
fn roundtrip_numeric_tagged_enum() {
	let connected = NumericallyTaggedEnum::Connected;
	let connected_json = serde_json::to_value(&connected).unwrap();
	let connected_back: NumericallyTaggedEnum =
		serde_json::from_value(connected_json.clone()).unwrap();
	assert_eq!(connected_json, serde_json::json!([0]));
	assert_eq!(connected_back, connected);

	let disconnected = NumericallyTaggedEnum::Disconnected(7);
	let disconnected_json = serde_json::to_value(&disconnected).unwrap();
	let disconnected_back: NumericallyTaggedEnum =
		serde_json::from_value(disconnected_json.clone()).unwrap();
	assert_eq!(disconnected_json, serde_json::json!([1, 7]));
	assert_eq!(disconnected_back, disconnected);
}

#[test]
fn serialize_map_forces_object_shape() {
	let value = ForcedMapStruct { alpha: 1, beta: String::from("two") };
	let serialized = serde_json::to_value(&value).unwrap();

	assert_eq!(serialized, serde_json::json!({ "alpha": 1, "beta": "two" }));
}

#[test]
fn deserialize_rejects_unknown_variant_tag() {
	let error = serde_json::from_value::<RandomEnum>(serde_json::json!(["unknown"])).unwrap_err();
	let msg = error.to_string();

	assert!(msg.contains("unknown variant"));
	assert!(msg.contains("first"));
}

#[test]
fn deserialize_rejects_extra_elements() {
	let error = serde_json::from_value::<RandomEnum>(serde_json::json!(["first", 10])).unwrap_err();
	assert!(error.to_string().contains("unexpected extra tuple elements"));
}

#[test]
fn trait_deserialize_tuple_elements_can_be_called_directly() {
	let mut deserializer = serde_json::Deserializer::from_str("[\"first\"]");

	let parsed = deserializer.deserialize_seq(DirectSeqVisitor).unwrap();

	assert_eq!(parsed, RandomEnum::First);
}

struct DirectSeqVisitor;

impl<'de> serde::de::Visitor<'de> for DirectSeqVisitor {
	type Value = RandomEnum;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a tuple enum")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: serde::de::SeqAccess<'de>,
	{
		<RandomEnum as DeserializeTupleElements<'de>>::deserialize_tuple_elements(&mut seq)
	}
}
