use serde::Deserialize;

use crate::RpcNotification;

#[derive(Debug, Deserialize)]
pub enum NvimNotification {
	Unknown,
}

impl<'a> TryFrom<RpcNotification<'a, rmpv::Value>> for NvimNotification {
	type Error = rmp_serde::decode::Error;

	fn try_from(value: RpcNotification<'a, rmpv::Value>) -> Result<Self, Self::Error> {
		match value.method {
			_ => Ok(Self::Unknown),
		}
	}
}
