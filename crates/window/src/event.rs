use std::sync::Arc;

use winit::window::Window;

#[derive(Debug, Clone)]
pub enum WinEvent {
	Created(Arc<Window>),
	Resized(Arc<Window>),
	ScaleFactorChanged(Arc<Window>),
	RedrawRequested(Arc<Window>),
	CloseRequested,
}
