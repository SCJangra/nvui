#[derive(Debug, Clone)]
pub struct RendererConfig {
	pub clear_color: wgpu::Color,
	pub power_preference: wgpu::PowerPreference,
	pub present_mode: Option<wgpu::PresentMode>,
	pub prefer_srgb_surface_format: bool,
	pub desired_maximum_frame_latency: u32,
}

impl Default for RendererConfig {
	fn default() -> Self {
		Self {
			clear_color: wgpu::Color { r: 0.08, g: 0.08, b: 0.10, a: 1.0 },
			power_preference: wgpu::PowerPreference::default(),
			present_mode: Some(wgpu::PresentMode::Fifo),
			prefer_srgb_surface_format: true,
			desired_maximum_frame_latency: 2,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_values() {
		let config = RendererConfig::default();

		assert_eq!(config.clear_color.r, 0.08);
		assert_eq!(config.clear_color.g, 0.08);
		assert_eq!(config.clear_color.b, 0.10);
		assert_eq!(config.clear_color.a, 1.0);
		assert_eq!(config.power_preference, wgpu::PowerPreference::default());
		assert_eq!(config.present_mode, Some(wgpu::PresentMode::Fifo));
		assert!(config.prefer_srgb_surface_format);
		assert_eq!(config.desired_maximum_frame_latency, 2);
	}
}
