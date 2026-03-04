use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::{io, thread};

use arc_swap::ArcSwap;
use dashmap::DashMap;
use flume::{Receiver, Sender};
use serde::Deserialize;

use crate::{
	IncomingRpcMessage, NvimNotification, OutgoingRpcMessage, RpcError, RpcMethod, RpcRequest,
	RpcResponse,
};

/// A general msgpack RPC client
pub struct RpcClient {
	inner: Arc<RpcClientInner>,
	request: Sender<Vec<u8>>,

	/// Handle for the thread that reads messages from the server
	#[allow(unused)]
	read_handle: thread::JoinHandle<()>,

	/// Handle for the thread that writes messages to the server
	#[allow(unused)]
	write_handle: thread::JoinHandle<()>,
}

struct RpcClientInner {
	/// Id of next RPC request
	next_req_id: AtomicU32,

	/// Whether the RPC client is running or not
	running: AtomicBool,

	/// Pending ['RpcRequest']s
	pending: DashMap<u32, Sender<RpcResponse<rmpv::Value>>>,

	/// All the subscriptions to [`NvimNotification`]s
	subscription: ArcSwap<Option<Sender<NvimNotification>>>,
}

impl RpcClientInner {
	fn start<R, W>(
		self: Arc<Self>,
		reader: R,
		writer: W,
		request: Receiver<Vec<u8>>,
	) -> (thread::JoinHandle<()>, thread::JoinHandle<()>)
	where
		R: io::Read + Send + Sync + 'static,
		W: io::Write + Send + Sync + 'static,
	{
		self.running.store(true, Ordering::SeqCst);

		let self_clone = self.clone();
		let write_handle = thread::spawn(move || {
			let mut writer = io::BufWriter::new(writer);

			loop {
				if !self_clone.running.load(Ordering::SeqCst) {
					break;
				}

				#[rustfmt::skip]
				let Ok(request) = request.recv()
					// TODO: Log error
					else { continue };

				#[rustfmt::skip]
                let Err(_err) = Self::send_request(request, &mut writer) else { continue; };
				// TODO: Log error
			}
		});

		let read_handle = thread::spawn(move || {
			let mut deserializer = rmp_serde::Deserializer::new(io::BufReader::new(reader));

			loop {
				if !self.running.load(Ordering::SeqCst) {
					break;
				}

				#[rustfmt::skip]
				let Ok(msg) = IncomingRpcMessage::<rmpv::Value>::deserialize(&mut deserializer)
					 // TODO: Log error
					 else { continue };

				#[rustfmt::skip]
				let Err(_err) = self.pricess_msg(msg)
					 // TODO: Log error
					 else { continue; };
			}
		});

		(read_handle, write_handle)
	}

	fn send_request<W>(request: Vec<u8>, writer: &mut W) -> Result<(), RpcError>
	where
		W: io::Write,
	{
		writer.write_all(&request).map_err(RpcError::FlushRequest)?;
		writer.flush().map_err(RpcError::FlushRequest)?;

		Ok(())
	}

	fn pricess_msg(&self, msg: IncomingRpcMessage<rmpv::Value>) -> Result<(), RpcError> {
		match msg {
			IncomingRpcMessage::Request(_) => {
				// TODO: Log this request
				return Ok(());
			},
			IncomingRpcMessage::Notification(notification) => {
				let sub = self.subscription.load();

				#[rustfmt::skip]
				let Some(sub) = sub.as_ref() else { return Ok(()); };
				sub.send(notification).map_err(|_| RpcError::SendNotification)?;
			},
			IncomingRpcMessage::Response(response) => {
				#[rustfmt::skip]
                let Some(req) = self.pending.get(&response.id) else { return Ok(()) };
				req.send(response).map_err(|_| RpcError::SendResponse)?;
			},
		}
		Ok(())
	}
}

impl RpcClient {
	pub fn start<R, W>(reader: R, writer: W) -> Self
	where
		R: io::Read + Send + Sync + 'static,
		W: io::Write + Send + Sync + 'static,
	{
		let (sender, receiver) = flume::unbounded();

		let inner = Arc::new(RpcClientInner {
			next_req_id: AtomicU32::new(0),
			running: AtomicBool::new(false),
			pending: DashMap::new(),
			subscription: ArcSwap::new(Arc::new(None)),
		});

		let (read_handle, write_handle) = inner.clone().start(reader, writer, receiver);

		Self { inner, request: sender, read_handle, write_handle }
	}

	pub async fn call_method<M>(&self, id: u32, params: M::Params) -> Result<M::Response, RpcError>
	where
		M: RpcMethod,
	{
		let (sender, receiver) = flume::bounded(1);

		self.inner.pending.insert(id, sender);

		let req =
			OutgoingRpcMessage::Request(RpcRequest { id, method: M::METHOD.to_string(), params });
		let req = rmp_serde::to_vec_named(&req)?;

		self.request.send(req).map_err(|_| RpcError::SendRequest)?;

		let response = receiver.recv_async().await?.result.map_err(RpcError::Response)?;

		let response: M::Response = rmpv::ext::from_value(response)?;
		Ok(response)
	}

	pub async fn call<M>(&self, params: M::Params) -> Result<M::Response, RpcError>
	where
		M: RpcMethod,
	{
		let id = self.inner.next_req_id.fetch_add(1, Ordering::SeqCst);
		let res = self.call_method::<M>(id, params).await;
		self.inner.pending.remove(&id);
		res
	}

	pub fn subscribe(&self) -> Result<Receiver<NvimNotification>, RpcError> {
		if self.inner.subscription.load().is_some() {
			return Err(RpcError::AlreadySubscribed);
		}
		let (sender, receiver) = flume::unbounded();

		self.inner.subscription.store(Arc::new(Some(sender)));

		Ok(receiver)
	}

	pub fn unsubscribe(&self) {
		self.inner.subscription.store(Arc::new(None));
	}

	pub fn stop(self) {
		let Self { inner, request, read_handle, write_handle } = self;

		inner.running.store(false, Ordering::SeqCst);
		drop(request);
		let _ = read_handle.join();
		let _ = write_handle.join();
	}
}

#[cfg(test)]
mod tests {
	use crate::{NvimEval, NvimUiAttach, NvimUiAttachParams, NvimUiOptions, RpcClient};

	#[test]
	fn call() {
		let mut nvim = std::process::Command::new("nvim")
			.arg("--embed")
			.stdin(std::process::Stdio::piped())
			.stdout(std::process::Stdio::piped())
			.spawn()
			.unwrap();

		let stdin = nvim.stdin.take().unwrap();
		let stdout = nvim.stdout.take().unwrap();

		let client = RpcClient::start(stdout, stdin);

		let future = client.call::<NvimEval>(["1 + 1".into()]);
		let response = smol::block_on(future).unwrap();

		assert_eq!(response, rmpv::Value::from(2));

		client.stop();
		nvim.kill().unwrap();
		nvim.wait().unwrap();
	}

	#[test]
	fn call_nvim_ui_attach() {
		let mut nvim = std::process::Command::new("nvim")
			.arg("--embed")
			.stdin(std::process::Stdio::piped())
			.stdout(std::process::Stdio::piped())
			.spawn()
			.unwrap();

		let stdin = nvim.stdin.take().unwrap();
		let stdout = nvim.stdout.take().unwrap();

		let client = RpcClient::start(stdout, stdin);

		let future = client.call::<NvimUiAttach>(NvimUiAttachParams {
			width: 10,
			height: 10,
			options: NvimUiOptions::all(),
		});
		let response = smol::block_on(future).unwrap();

		assert_eq!(response, rmpv::Value::Nil);

		client.stop();
		nvim.kill().unwrap();
		nvim.wait().unwrap();
	}
}
