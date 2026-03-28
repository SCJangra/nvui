#[derive(Debug, Clone)]
pub struct WindowConfig {
	pub title: String,
	pub width: u32,
	pub height: u32,
	pub resizable: bool,
}

impl Default for WindowConfig {
	fn default() -> Self {
		Self { title: String::from("NvUI"), width: 1280, height: 720, resizable: true }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_values() {
		let config = WindowConfig::default();

		assert_eq!(config.title, "NvUI");
		assert_eq!(config.width, 1280);
		assert_eq!(config.height, 720);
		assert!(config.resizable);
	}
}
