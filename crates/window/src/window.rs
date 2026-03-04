use std::sync::Arc;

use flume::{Receiver, Sender};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::{WinEvent, WindowCommand, WindowConfig, WindowError};

pub fn start(
	config: WindowConfig,
	commands: Receiver<WindowCommand>,
	events: Sender<WinEvent>,
) -> Result<(), WindowError> {
	let event_loop = EventLoop::new()?;
	let mut app = WindowApp::new(config, commands, events);

	event_loop.run_app(&mut app)?;

	if let Some(error) = app.error {
		return Err(error);
	}

	Ok(())
}

struct WindowApp {
	config: WindowConfig,
	commands: Receiver<WindowCommand>,
	events: Sender<WinEvent>,
	window: Option<Arc<Window>>,
	error: Option<WindowError>,
}

impl WindowApp {
	fn new(
		config: WindowConfig,
		commands: Receiver<WindowCommand>,
		events: Sender<WinEvent>,
	) -> Self {
		Self { config, commands, events, window: None, error: None }
	}

	fn window_attributes(config: &WindowConfig) -> WindowAttributes {
		Window::default_attributes()
			.with_title(config.title.clone())
			.with_inner_size(LogicalSize::new(f64::from(config.width), f64::from(config.height)))
			.with_resizable(config.resizable)
	}

	fn send_event(&self, event: WinEvent) {
		let Err(_err) = self.events.send(event) else { return };
		// TODO: Log error
	}

	fn emit_created(&mut self, window: Arc<Window>) {
		self.send_event(WinEvent::Created(window));
	}

	fn handle_command(&self, event_loop: &ActiveEventLoop, command: WindowCommand) {
		match command {
			WindowCommand::Close => event_loop.exit(),
			WindowCommand::RequestRedraw => {
				let Some(window) = self.window.as_ref() else { return };
				window.request_redraw();
			},
			WindowCommand::SetTitle(title) => {
				let Some(window) = self.window.as_ref() else { return };
				window.set_title(&title);
			},
		}
	}

	fn drain_commands(&self, event_loop: &ActiveEventLoop) {
		while let Ok(command) = self.commands.try_recv() {
			self.handle_command(event_loop, command);
		}
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
				let window = Arc::new(window);
				self.emit_created(window.clone());
				window.request_redraw();
				self.window = Some(window);
			},
			Err(error) => {
				self.error = Some(WindowError::Os(error));
				event_loop.exit();
			},
		}
	}

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		self.drain_commands(event_loop);
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let Some(window) = self.window.as_ref().cloned() else { return };

		if window.id() != window_id {
			return;
		}

		match event {
			WindowEvent::CloseRequested => {
				self.send_event(WinEvent::CloseRequested);
			},
			WindowEvent::Resized(_) => {
				self.send_event(WinEvent::Resized(window));
			},
			WindowEvent::ScaleFactorChanged { .. } => {
				self.send_event(WinEvent::ScaleFactorChanged(window));
			},
			WindowEvent::RedrawRequested => {
				self.send_event(WinEvent::RedrawRequested(window));
			},
			_ => (),
		}

		self.drain_commands(event_loop);
	}
}
