#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Renderer({0})")]
	Renderer(#[from] renderer::RendererError),

	#[error("CreateWindow({0})")]
	Window(#[from] winit::error::OsError),
}
