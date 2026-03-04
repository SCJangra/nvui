use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::{WindowConfig, WindowError};

pub fn run(config: WindowConfig) -> Result<(), WindowError> {
	let event_loop = EventLoop::new()?;
	let mut app = WindowApp::new(config);

	event_loop.run_app(&mut app)?;

	if let Some(error) = app.error {
		return Err(WindowError::CreateWindow(error));
	}

	Ok(())
}

struct WindowApp {
	config: WindowConfig,
	window: Option<Window>,
	window_id: Option<WindowId>,
	error: Option<winit::error::OsError>,
}

impl WindowApp {
	fn new(config: WindowConfig) -> Self {
		Self { config, window: None, window_id: None, error: None }
	}

	fn window_attributes(config: &WindowConfig) -> WindowAttributes {
		Window::default_attributes()
			.with_title(config.title.clone())
			.with_inner_size(LogicalSize::new(f64::from(config.width), f64::from(config.height)))
			.with_resizable(config.resizable)
	}
}

impl ApplicationHandler for WindowApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_some() {
			return;
		}

		let attributes = Self::window_attributes(&self.config);

		match event_loop.create_window(attributes) {
			Ok(window) => {
				self.window_id = Some(window.id());
				self.window = Some(window);
			},
			Err(error) => {
				self.error = Some(error);
				event_loop.exit();
			},
		}
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		if Some(window_id) != self.window_id {
			return;
		}

		if let WindowEvent::CloseRequested = event {
			event_loop.exit();
		}
	}
}
