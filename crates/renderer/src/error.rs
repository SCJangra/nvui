#[derive(Debug, thiserror::Error)]
pub enum RendererError {
	#[error("Handle({0})")]
	Handle(#[from] raw_window_handle::HandleError),

	#[error("CreateSurface({0})")]
	CreateSurface(#[from] wgpu::CreateSurfaceError),

	#[error("NoAdapter")]
	NoAdapter,

	#[error("RequestDevice({0})")]
	RequestDevice(#[from] wgpu::RequestDeviceError),

	#[error("NoSurfaceFormat")]
	NoSurfaceFormat,

	#[error("NoPresentMode")]
	NoPresentMode,

	#[error("UnsupportedPresentMode({0:?})")]
	UnsupportedPresentMode(wgpu::PresentMode),

	#[error("NoAlphaMode")]
	NoAlphaMode,

	#[error("Surface({0})")]
	Surface(#[from] wgpu::SurfaceError),
}
