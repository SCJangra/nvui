use std::io;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
	#[error("Rmpv({0})")]
	Rmpv(#[from] rmpv::ext::Error),

	#[error("Rmp({0})")]
	RmpWrite(#[from] rmp_serde::encode::Error),

	#[error("FlumeRecv({0})")]
	FlumeRecv(#[from] flume::RecvError),

	#[error("SendRequest")]
	SendRequest,

	#[error("WriteRequest({0})")]
	FlushRequest(io::Error),

	#[error("SendResponse")]
	SendResponse,

	#[error("SendNotification")]
	SendNotification,

	#[error("AlreadySubscribed")]
	AlreadySubscribed,

	#[error("Response({0})")]
	Response(rmpv::Value),

	#[error("WriterPoisoned")]
	WriterPoisoned,

	#[error("RequestDropped")]
	RequestDropped,

	#[error("DeserializeNotification(${0})")]
	DeserializeNotification(rmp_serde::decode::Error),
}
