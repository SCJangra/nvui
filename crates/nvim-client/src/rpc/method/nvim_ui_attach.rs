use serde::Serialize;

use crate::RpcMethod;

/// Neovim's [`nvim_ui_attach`](https://neovim.io/doc/user/api/#nvim_ui_attach()) function
pub struct NvimUiAttach;

impl RpcMethod for NvimUiAttach {
	const METHOD: &'static str = "nvim_ui_attach";

	type Params = NvimUiAttachParams;
	type Response = rmpv::Value;
}

#[derive(Debug, Serialize)]
pub struct NvimUiAttachParams {
	pub width: u32,
	pub height: u32,
	pub options: NvimUiOptions,
}

/// Neovim's external UI [`options`](https://neovim.io/doc/user/api-ui-events/#ui-option)
#[derive(Debug, Default)]
pub struct NvimUiOptions {
	pub ext_multigrid: bool,
}

impl NvimUiOptions {
	pub fn all() -> Self {
		Self { ext_multigrid: true }
	}
}

mod ser_de {
	use serde::ser::SerializeMap;

	use super::*;

	impl Serialize for NvimUiOptions {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			let mut map = serializer.serialize_map(Some(1))?;
			map.serialize_entry("ext_multigrid", &self.ext_multigrid)?;
			map.end()
		}
	}
}
