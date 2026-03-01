use nvui_serde::SerializeTuple;
use serde::Serialize;

use crate::RpcMethod;

pub struct NvimUiAttach;

impl RpcMethod for NvimUiAttach {
	const METHOD: &'static str = "nvim_ui_attach";

	type Params = NvimUiAttachParams;
	type Response = rmpv::Value;
}

#[derive(Debug, SerializeTuple)]
pub struct NvimUiAttachParams {
	pub width: u32,
	pub height: u32,
	pub options: NvimUiOptions,
}

#[derive(Debug, Serialize)]
pub struct NvimUiOptions {
	#[serde(default = "default_true")]
	pub ext_multigrid: bool,
}
