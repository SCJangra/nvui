#[derive(Debug, thiserror::Error)]
pub enum WindowError {
	#[error("EventLoop({0})")]
	EventLoop(#[from] winit::error::EventLoopError),

	#[error("CreateWindow({0})")]
	Os(#[from] winit::error::OsError),
}
