use std::process;

use flume::Receiver;

use crate::{
	Error, NvimNotification, NvimUiAttach, NvimUiAttachParams, NvimUiTryResize,
	NvimUiTryResizeParams, RpcClient,
};

pub struct Nvim {
	rpc: RpcClient,
}

impl Nvim {
	pub fn start() -> Result<Self, Error> {
		let mut nvim = process::Command::new("nvim")
			.arg("--embed")
			.stdin(process::Stdio::piped())
			.stdout(process::Stdio::piped())
			.spawn()
			.map_err(Error::SpawnNvim)?;

		let stdin = nvim.stdin.take().expect("stdin handle present");
		let stdout = nvim.stdout.take().expect("stdout handle present");

		let rpc = RpcClient::start(stdout, stdin);

		Ok(Self { rpc })
	}

	pub fn notifications(&self) -> Result<Receiver<NvimNotification>, Error> {
		self.rpc.subscribe().map_err(Error::Rpc)
	}

	pub async fn ui_attach(&self, params: NvimUiAttachParams) -> Result<rmpv::Value, Error> {
		self.rpc.call::<NvimUiAttach>(params).await.map_err(Error::Rpc)
	}

	pub async fn ui_try_resize(&self, params: NvimUiTryResizeParams) -> Result<rmpv::Value, Error> {
		self.rpc.call::<NvimUiTryResize>(params).await.map_err(Error::Rpc)
	}
}
