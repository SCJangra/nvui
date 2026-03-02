use nvui_serde::{DeserializeTuple, SerializeTuple};

use super::NvimNotification;

#[derive(Debug, SerializeTuple)]
#[cfg_attr(test, derive(PartialEq))]
pub enum OutgoingRpcMessage<T> {
	#[tuple(rename = 0)]
	Request(#[tuple(flatten)] RpcRequest<T>),
	#[tuple(rename = 1)]
	Response(#[tuple(flatten)] RpcResponse<T>),
}

#[derive(Debug, DeserializeTuple)]
#[cfg_attr(test, derive(PartialEq))]
pub enum IncomingRpcMessage<T> {
	#[tuple(rename = 0)]
	Request(#[tuple(flatten)] RpcRequest<T>),
	#[tuple(rename = 1)]
	Response(#[tuple(flatten)] RpcResponse<T>),
	#[tuple(rename = 2)]
	Notification(#[tuple(flatten)] NvimNotification),
}

#[derive(Debug, SerializeTuple, DeserializeTuple)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RpcRequest<P> {
	pub id: u32,
	pub method: String,
	pub params: P,
}

#[derive(Debug, SerializeTuple, DeserializeTuple)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RpcResponse<R> {
	pub id: u32,
	pub error: rmpv::Value,
	pub result: R,
}

impl<R> RpcResponse<R> {
	pub fn into_result(self) -> Result<R, rmpv::Value> {
		if self.error.is_nil() {
			Ok(self.result)
		} else {
			Err(self.error)
		}
	}
}
