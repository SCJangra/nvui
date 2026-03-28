mod app;
mod error;
mod grid;
mod windows;

use app::App;
use windows::WindowConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let event_loop = winit::event_loop::EventLoop::new()?;
	let mut app = App::new(WindowConfig::default());

	event_loop.run_app(&mut app)?;

	if let Some(error) = app.error.take() {
		return Err(Box::new(error));
	}

	Ok(())
}
