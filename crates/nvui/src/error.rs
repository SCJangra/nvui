#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Renderer({0})")]
	Renderer(#[from] renderer::RendererError),

	#[error("Window({0})")]
	SendCommand(#[from] flume::SendError<win::WindowCommand>),
}
