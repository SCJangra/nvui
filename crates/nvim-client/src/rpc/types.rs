use serde::ser::SerializeTuple;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub enum RpcMessage<'a, T> {
	Request(RpcRequest<'a, T>),
	Response(RpcResponse<T>),
	Notification(RpcNotification<'a, T>),
}

#[derive(Debug, PartialEq)]
pub struct RpcRequest<'a, P> {
	pub id: u32,
	pub method: Cow<'a, str>,
	pub params: P,
}

#[derive(Debug, PartialEq)]
pub struct RpcResponse<R> {
	pub id: u32,
	pub result: Result<R, rmpv::Value>,
}

#[derive(Debug, PartialEq)]
pub struct RpcNotification<'a, P> {
	pub method: Cow<'a, str>,
	pub params: P,
}

mod ser_de {
	use std::marker::PhantomData;

	use serde::de::{self, DeserializeOwned, SeqAccess, Visitor};
	use serde::{Deserialize, Deserializer, Serialize, Serializer};

	use super::*;

	impl<'a, T> Serialize for RpcMessage<'a, T>
	where
		T: Serialize,
	{
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: Serializer,
		{
			match self {
				RpcMessage::Request(request) => {
					let mut s = serializer.serialize_tuple(4)?;
					s.serialize_element(&0)?;
					s.serialize_element(&request.id)?;
					s.serialize_element(&request.method)?;
					s.serialize_element(&request.params)?;
					s.end()
				},
				RpcMessage::Response(response) => {
					let mut s = serializer.serialize_tuple(4)?;
					s.serialize_element(&1)?;
					s.serialize_element(&response.id)?;

					match &response.result {
						Ok(r) => {
							s.serialize_element(&())?;
							s.serialize_element(r)?;
						},
						Err(e) => {
							s.serialize_element(e)?;
							s.serialize_element(&())?;
						},
					}

					s.end()
				},
				RpcMessage::Notification(notification) => {
					let mut s = serializer.serialize_tuple(3)?;
					s.serialize_element(&2)?;
					s.serialize_element(&notification.method)?;
					s.serialize_element(&notification.params)?;
					s.end()
				},
			}
		}
	}

	impl<'de, T> Deserialize<'de> for RpcMessage<'static, T>
	where
		T: DeserializeOwned,
	{
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: Deserializer<'de>,
		{
			deserializer.deserialize_seq(RpcMessageVisitor(PhantomData))
		}
	}

	struct RpcMessageVisitor<T>(PhantomData<T>);

	impl<'de, T> Visitor<'de> for RpcMessageVisitor<T>
	where
		T: DeserializeOwned,
	{
		type Value = RpcMessage<'static, T>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("an RPC message tuple")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: SeqAccess<'de>,
		{
			let msg_type: u8 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;

			match msg_type {
				0 => {
					let id = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
					let method: String = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
					let params = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;

					Ok(RpcMessage::Request(RpcRequest { id, method: Cow::Owned(method), params }))
				},
				1 => {
					let id = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
					let error: rmpv::Value = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;

					if matches!(error, rmpv::Value::Nil) {
						let result: T = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;
						Ok(RpcMessage::Response(RpcResponse { id, result: Ok(result) }))
					} else {
						let _: de::IgnoredAny =
							seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;
						Ok(RpcMessage::Response(RpcResponse { id, result: Err(error) }))
					}
				},
				2 => {
					let method: String = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
					let params = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;

					Ok(RpcMessage::Notification(RpcNotification { method: Cow::Owned(method), params }))
				},
				_ => Err(de::Error::custom("unknown RPC message type")),
			}
		}
	}
}
