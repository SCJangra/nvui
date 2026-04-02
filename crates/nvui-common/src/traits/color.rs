pub trait Color {
	fn from_nvim_color(color: u32) -> wgpu::Color {
		let red = f64::from((color >> 16) & 0xFF) / 255.0;
		let green = f64::from((color >> 8) & 0xFF) / 255.0;
		let blue = f64::from(color & 0xFF) / 255.0;

		wgpu::Color { r: red, g: green, b: blue, a: 1.0 }
	}
}

impl Color for wgpu::Color {}
