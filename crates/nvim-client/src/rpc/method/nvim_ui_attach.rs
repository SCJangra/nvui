use serde::Serialize;
use serde_tuple::Serialize_tuple;

use crate::RpcMethod;

/// Neovim's [`nvim_ui_attach`](https://neovim.io/doc/user/api/#nvim_ui_attach()) function
pub struct NvimUiAttach;

impl RpcMethod for NvimUiAttach {
	const METHOD: &'static str = "nvim_ui_attach";

	type Params = NvimUiAttachParams;
	type Response = rmpv::Value;
}

#[derive(Debug, Serialize_tuple)]
pub struct NvimUiAttachParams {
	pub width: u32,
	pub height: u32,
	pub options: NvimUiOptions,
}

/// Neovim's external UI [`options`](https://neovim.io/doc/user/api-ui-events/#ui-option)
#[derive(Debug, Default, Serialize)]
pub struct NvimUiOptions {
	pub ext_multigrid: bool,
}

impl NvimUiOptions {
	pub fn all() -> Self {
		Self { ext_multigrid: true }
	}
}
