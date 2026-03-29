use std::sync::Arc;
use std::thread;

use dashmap::DashMap;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoopProxy;
use winit::window::{Window, WindowAttributes, WindowId};

use nvim::{Nvim, NvimNotification, NvimUiAttachParams, NvimUiOptions, RedrawNotification};
use renderer::{Renderer, RendererConfig, RendererError};

use crate::error::Error;
use crate::grid::Grid;
use crate::windows::WindowConfig;

pub struct App {
	pub config: WindowConfig,
	pub windows: DashMap<WindowId, Arc<Window>>,
	pub renderers: DashMap<WindowId, Renderer>,
	pub grids: DashMap<WindowId, Grid>,
	pub nvim: Nvim,
	pub error: Option<Error>,
}

#[derive(Debug)]
pub enum AppEvent {
	Nvim(NvimNotification),
}

impl App {
	pub fn new(config: WindowConfig, proxy: EventLoopProxy<AppEvent>) -> Result<Self, Error> {
		let nvim = Nvim::start()?;
		let notifications = nvim.notifications()?;
		let proxy = proxy.clone();
		thread::spawn(move || {
			for notification in notifications.iter() {
				let _ = proxy.send_event(AppEvent::Nvim(notification));
			}
		});
		Ok(Self {
			config,
			windows: DashMap::new(),
			renderers: DashMap::new(),
			grids: DashMap::new(),
			nvim,
			error: None,
		})
	}

	fn window_attributes(config: &WindowConfig) -> WindowAttributes {
		Window::default_attributes()
			.with_title(config.title.clone())
			.with_inner_size(LogicalSize::new(f64::from(config.width), f64::from(config.height)))
			.with_resizable(config.resizable)
	}

	fn create_window(
		&mut self,
		event_loop: &ActiveEventLoop,
		config: &WindowConfig,
	) -> Result<Arc<Window>, Error> {
		let attributes = Self::window_attributes(config);
		let window = event_loop.create_window(attributes)?;
		let window = Arc::new(window);
		self.windows.insert(window.id(), window.clone());
		self.grids.insert(window.id(), Grid::new(0, 0));
		Ok(window)
	}

	fn create_renderer(&mut self, id: WindowId, window: &Window) -> Result<(), Error> {
		let size = window.inner_size();

		let renderer = smol::block_on(Renderer::with_config(
			window,
			size.width,
			size.height,
			RendererConfig::default(),
		))?;

		self.renderers.insert(id, renderer);

		Ok(())
	}

	fn attach_ui(&self, window: &Window) -> Result<(), Error> {
		let size = window.inner_size();

		let params = NvimUiAttachParams {
			width: size.width,
			height: size.height,
			options: NvimUiOptions::all(),
		};

		smol::block_on(self.nvim.ui_attach(params))?;
		Ok(())
	}

	fn handle_nvim_notification(&mut self, notification: NvimNotification) {
		match notification {
			NvimNotification::Redraw(events) => self.apply_redraw_events(events),
			NvimNotification::Other { .. } => (),
		}
	}

	fn apply_redraw_events(&mut self, events: Vec<RedrawNotification>) {
		for event in events {
			match event {
				RedrawNotification::GridResize(resizes) => {
					for resize in resizes {
						if resize.grid != 1 {
							continue;
						}
						for mut grid in self.grids.iter_mut() {
							grid.resize(resize.width as usize, resize.height as usize);
						}
					}
				},
				RedrawNotification::GridClear(clears) => {
					for clear in clears {
						if clear.grid != 1 {
							continue;
						}
						for mut grid in self.grids.iter_mut() {
							grid.clear();
						}
					}
				},
				RedrawNotification::GridLine(lines) => {
					for line in lines {
						if line.grid != 1 {
							continue;
						}
						for mut grid in self.grids.iter_mut() {
							grid.set_line(line.row as usize, line.col_start as usize, &line.cells);
						}
					}
				},
				_ => (),
			}
		}
	}

	fn resize_renderer(&mut self, id: WindowId, window: &Window) -> Result<(), Error> {
		let Some(mut renderer) = self.renderers.get_mut(&id) else { return Ok(()) };
		let size = window.inner_size();

		renderer.resize(size.width, size.height);
		Ok(())
	}

	fn redraw_renderer(
		&mut self,
		event_loop: &ActiveEventLoop,
		id: WindowId,
		window: &Window,
	) -> Result<(), Error> {
		let Some(mut renderer) = self.renderers.get_mut(&id) else { return Ok(()) };

		let Err(err) = renderer.render_clear() else { return Ok(()) };

		match err {
			RendererError::Surface(wgpu::SurfaceError::Lost)
			| RendererError::Surface(wgpu::SurfaceError::Outdated) => {
				window.request_redraw();
			},
			RendererError::Surface(wgpu::SurfaceError::OutOfMemory) => {
				self.error = Some(err.into());
				event_loop.exit();
			},
			_ => (), // TODO: Log error
		}

		Ok(())
	}

	fn close_window(&mut self, event_loop: &ActiveEventLoop, id: WindowId) {
		self.renderers.remove(&id);
		self.grids.remove(&id);
		self.windows.remove(&id);

		if self.windows.is_empty() {
			event_loop.exit();
		}
	}
}

impl ApplicationHandler<AppEvent> for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if !self.windows.is_empty() {
			return;
		}

		let config = self.config.clone();
		let window = match self.create_window(event_loop, &config) {
			Ok(window) => window,
			Err(error) => {
				self.error = Some(error);
				event_loop.exit();
				return;
			},
		};

		if let Err(error) = self.create_renderer(window.id(), &window) {
			self.error = Some(error);
			event_loop.exit();
			return;
		}

		if let Err(error) = self.attach_ui(&window) {
			self.error = Some(error);
			event_loop.exit();
			return;
		}

		window.request_redraw();
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let Some(window) = self.windows.get(&window_id).map(|entry| entry.value().clone()) else {
			return;
		};

		match event {
			WindowEvent::CloseRequested => {
				self.close_window(event_loop, window_id);
			},
			WindowEvent::Resized(_) => {
				if let Err(error) = self.resize_renderer(window_id, &window) {
					self.error = Some(error);
					event_loop.exit();
				}
			},
			WindowEvent::ScaleFactorChanged { .. } => {
				if let Err(error) = self.resize_renderer(window_id, &window) {
					self.error = Some(error);
					event_loop.exit();
				}
			},
			WindowEvent::RedrawRequested => {
				if let Err(error) = self.redraw_renderer(event_loop, window_id, &window) {
					self.error = Some(error);
					event_loop.exit();
				}
			},
			_ => (),
		}
	}

	fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
		match event {
			AppEvent::Nvim(notification) => self.handle_nvim_notification(notification),
		}
	}
}
