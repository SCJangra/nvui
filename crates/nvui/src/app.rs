use flume::{Receiver, Sender};
use winit::window::Window;

use nvui_renderer::{Renderer, RendererConfig, RendererError};
use win::{WinEvent, WindowCommand};

use crate::error::Error;

pub struct App {
	pub event: Receiver<WinEvent>,
	pub command: Sender<WindowCommand>,
	pub renderer: Option<Renderer>,
}

impl App {
	pub fn new(event: Receiver<WinEvent>, command: Sender<WindowCommand>) -> Self {
		Self { event, command, renderer: None }
	}

	pub fn create_renderer(&mut self, window: &Window) -> Result<(), Error> {
		let size = window.inner_size();

		let renderer = smol::block_on(Renderer::with_config(
			window,
			size.width,
			size.height,
			RendererConfig::default(),
		))?;

		self.renderer = Some(renderer);

		Ok(())
	}

	pub fn resize(&mut self, window: &Window) -> Result<(), Error> {
		let Some(ref mut renderer) = self.renderer else { return Ok(()) };

		let size = window.inner_size();

		renderer.resize(size.width, size.height);
		Ok(())
	}

	pub fn redraw(&mut self, _window: &Window) -> Result<(), Error> {
		let Some(ref mut renderer) = self.renderer else { return Ok(()) };

		let Err(err) = renderer.render_clear() else { return Ok(()) };

		match err {
			RendererError::Surface(wgpu::SurfaceError::Lost)
			| RendererError::Surface(wgpu::SurfaceError::Outdated) => {
				self.command.send(win::WindowCommand::RequestRedraw)?;
			},
			RendererError::Surface(wgpu::SurfaceError::OutOfMemory) => {
				self.command.send(win::WindowCommand::Close)?;
			},
			_ => (), // TODO: Log error
		}

		Ok(())
	}

	pub fn close(&mut self) -> Result<(), Error> {
		self.command.send(WindowCommand::Close)?;
		Ok(())
	}

	pub fn start_event_loop(mut self) -> Result<(), Error> {
		while let Ok(event) = self.event.recv() {
			self.process_event(event)?;
		}

		Ok(())
	}

	pub fn process_event(&mut self, event: WinEvent) -> Result<(), Error> {
		match event {
			WinEvent::Created(window) => self.create_renderer(&window),
			WinEvent::Resized(window) => self.resize(&window),
			WinEvent::ScaleFactorChanged(_) => Ok(()),
			WinEvent::RedrawRequested(window) => self.redraw(&window),
			WinEvent::CloseRequested => self.close(),
		}
	}
}
