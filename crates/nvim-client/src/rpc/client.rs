use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::{io, thread};

use arc_swap::ArcSwap;
use dashmap::DashMap;
use flume::{Receiver, Sender};
use serde::{Deserialize, Serialize};

use crate::{NvimNotification, RpcError, RpcMessage, RpcMethod, RpcRequest, RpcResponse};

/// A general msgpack RPC client
pub struct RpcClient {
	inner: Arc<RpcClientInner>,

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

	/// Sends requests to the thread that writes messages to the server
	request: Sender<RpcRequest<rmpv::Value>>,
}

impl RpcClientInner {
	fn start<R, W>(
		self: Arc<Self>,
		reader: R,
		writer: W,
		request: Receiver<RpcRequest<rmpv::Value>>,
	) -> (thread::JoinHandle<()>, thread::JoinHandle<()>)
	where
		R: io::Read + Send + Sync + 'static,
		W: io::Write + Send + Sync + 'static,
	{
		self.running.store(true, Ordering::SeqCst);

		let self_clone = self.clone();
		let write_handle = thread::spawn(move || {
			let mut writer = io::BufWriter::new(writer);

			while let Ok(request) = request.recv() {
				if !self_clone.running.load(Ordering::SeqCst) {
					break;
				}

				#[rustfmt::skip]
                let Err(_err) = Self::send_request(request, &mut writer) else { continue; };
				// TODO: Log error
			}
		});

		let read_handle = thread::spawn(move || {
			let mut deserializer = rmp_serde::Deserializer::new(io::BufReader::new(reader));

			while let Ok(msg) = RpcMessage::<rmpv::Value>::deserialize(&mut deserializer) {
				if !self.running.load(Ordering::SeqCst) {
					break;
				}

				#[rustfmt::skip]
                let Err(_err) = self.pricess_msg(msg) else { continue; };
				// TODO: Log error
			}
		});

		(read_handle, write_handle)
	}

	fn send_request<P, W>(request: RpcRequest<P>, writer: &mut W) -> Result<(), RpcError>
	where
		P: Serialize,
		W: io::Write,
	{
		let msg = RpcMessage::Request(request);
		rmp_serde::encode::write(writer, &msg)?;
		writer.flush().map_err(RpcError::FlushRequest)?;

		Ok(())
	}

	fn pricess_msg(&self, msg: RpcMessage<rmpv::Value>) -> Result<(), RpcError> {
		match msg {
			RpcMessage::Request(_) => {
				// TODO: Log this request
				return Ok(());
			},
			RpcMessage::Notification(notification) => {
				let sub = self.subscription.load();

				#[rustfmt::skip]
                let Some(sub) = sub.as_ref() else { return Ok(()); };
				sub.send(notification).map_err(|_| RpcError::SendNotification)?;
			},
			RpcMessage::Response(response) => {
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
			request: sender,
		});

		let (read_handle, write_handle) = inner.clone().start(reader, writer, receiver);

		Self { inner, read_handle, write_handle }
	}

	pub async fn call_method<M>(&self, id: u32, params: M::Params) -> Result<M::Response, RpcError>
	where
		M: RpcMethod,
	{
		let (sender, receiver) = flume::bounded(1);

		self.inner.pending.insert(id, sender);

		let val = rmpv::ext::to_value(params)?;
		let req = RpcRequest { id, method: M::METHOD.to_string(), params: val };

		self.inner.request.send(req).map_err(|_| RpcError::SendRequest)?;

		let response = receiver.recv_async().await?.into_result().map_err(RpcError::Response)?;

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
}

impl Drop for RpcClient {
	fn drop(&mut self) {
		self.inner.running.store(false, Ordering::SeqCst);
	}
}

#[cfg(test)]
mod tests {
	use crate::NvimEval;

	use super::RpcClient;

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
	}
}
