use crate::RpcMethod;

pub struct NvimUiAttach;

impl RpcMethod for NvimUiAttach {
	const METHOD: &'static str = "nvim_ui_attach";

	type Params = (u32, u32, rmpv::Value);
	type Response = rmpv::Value;
}
