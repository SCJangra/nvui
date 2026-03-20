use serde_tuple::Serialize_tuple;

use crate::RpcMethod;

/// Neovim's [`nvim_ui_try_resize`](https://neovim.io/doc/user/api/#nvim_ui_try_resize()) function
pub struct NvimUiTryResize;

impl RpcMethod for NvimUiTryResize {
	const METHOD: &'static str = "nvim_ui_try_resize";

	type Params = NvimUiTryResizeParams;
	type Response = rmpv::Value;
}

#[derive(Debug, Serialize_tuple)]
pub struct NvimUiTryResizeParams {
	pub width: u32,
	pub height: u32,
}
