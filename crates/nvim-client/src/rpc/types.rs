use nvui_serde::{DeserializeTuple, SerializeTuple};

use super::NvimNotification;

#[derive(Debug, PartialEq, SerializeTuple, DeserializeTuple)]
pub enum RpcMessage<T> {
	#[tuple(rename = 0)]
	Request(#[tuple(flatten)] RpcRequest<T>),
	#[tuple(rename = 1)]
	Response(#[tuple(flatten)] RpcResponse<T>),
	#[tuple(rename = 2)]
	Notification(#[tuple(flatten)] NvimNotification),
}

#[derive(Debug, PartialEq, SerializeTuple, DeserializeTuple)]
pub struct RpcRequest<P> {
	pub id: u32,
	pub method: String,
	pub params: P,
}

#[derive(Debug, PartialEq, SerializeTuple, DeserializeTuple)]
pub struct RpcResponse<R> {
	pub id: u32,
	pub error: Option<rmpv::Value>,
	pub result: Option<R>,
}

impl<R> RpcResponse<R> {
	pub fn into_result(self) -> Result<R, rmpv::Value> {
		match (self.error, self.result) {
			(Some(error), _) => Err(error),
			(None, Some(result)) => Ok(result),
			(None, None) => Err(rmpv::Value::Nil),
		}
	}
}
