use crate::{RendererConfig, RendererError};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

pub struct Renderer {
	surface: wgpu::Surface<'static>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	clear_color: wgpu::Color,
}

impl Renderer {
	pub async fn new(
		window: &winit::window::Window,
		width: u32,
		height: u32,
	) -> Result<Self, RendererError> {
		Self::with_config(window, width, height, RendererConfig::default()).await
	}

	pub async fn with_config(
		window: &winit::window::Window,
		width: u32,
		height: u32,
		renderer_config: RendererConfig,
	) -> Result<Self, RendererError> {
		let raw_display_handle = window.display_handle()?.as_raw();
		let raw_window_handle = window.window_handle()?.as_raw();

		unsafe {
			Self::with_raw_handles(
				raw_display_handle,
				raw_window_handle,
				width,
				height,
				renderer_config,
			)
			.await
		}
	}

	pub async unsafe fn with_raw_handles(
		raw_display_handle: RawDisplayHandle,
		raw_window_handle: RawWindowHandle,
		width: u32,
		height: u32,
		renderer_config: RendererConfig,
	) -> Result<Self, RendererError> {
		let instance = wgpu::Instance::default();

		let surface = unsafe {
			instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
				raw_display_handle,
				raw_window_handle,
			})?
		};

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: renderer_config.power_preference,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.map_err(|_| RendererError::NoAdapter)?;

		let (device, queue) = adapter
			.request_device(&wgpu::DeviceDescriptor {
				label: Some("nvui-renderer-device"),
				required_features: wgpu::Features::empty(),
				required_limits: wgpu::Limits::default(),
				memory_hints: wgpu::MemoryHints::Performance,
				trace: wgpu::Trace::Off,
				experimental_features: wgpu::ExperimentalFeatures::disabled(),
			})
			.await?;

		let capabilities = surface.get_capabilities(&adapter);

		let format = if renderer_config.prefer_srgb_surface_format {
			capabilities
				.formats
				.iter()
				.copied()
				.find(wgpu::TextureFormat::is_srgb)
				.or_else(|| capabilities.formats.first().copied())
				.ok_or(RendererError::NoSurfaceFormat)?
		} else {
			capabilities.formats.first().copied().ok_or(RendererError::NoSurfaceFormat)?
		};

		let present_mode = match renderer_config.present_mode {
			Some(mode) => {
				if capabilities.present_modes.contains(&mode) {
					mode
				} else {
					return Err(RendererError::UnsupportedPresentMode(mode));
				}
			},
			None => capabilities
				.present_modes
				.first()
				.copied()
				.ok_or(RendererError::NoPresentMode)?,
		};

		let alpha_mode =
			capabilities.alpha_modes.first().copied().ok_or(RendererError::NoAlphaMode)?;

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format,
			width: width.max(1),
			height: height.max(1),
			present_mode,
			alpha_mode,
			view_formats: vec![],
			desired_maximum_frame_latency: renderer_config.desired_maximum_frame_latency.max(1),
		};

		surface.configure(&device, &config);

		Ok(Self { surface, device, queue, config, clear_color: renderer_config.clear_color })
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self.config.width = width.max(1);
		self.config.height = height.max(1);
		self.surface.configure(&self.device, &self.config);
	}

	pub fn set_clear_color(&mut self, clear_color: wgpu::Color) {
		self.clear_color = clear_color;
	}

	pub fn render_clear(&mut self) -> Result<(), RendererError> {
		let frame = self.surface.get_current_texture()?;
		let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("nvui-renderer-clear-encoder"),
		});

		{
			let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("nvui-renderer-clear-pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					depth_slice: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(self.clear_color),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				occlusion_query_set: None,
				timestamp_writes: None,
				multiview_mask: None,
			});
		}

		self.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}
}
