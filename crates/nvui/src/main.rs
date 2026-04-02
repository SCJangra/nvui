mod app;
mod error;
mod grid;
mod highlight;
mod windows;

use app::{App, AppEvent};
use windows::WindowConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let event_loop = winit::event_loop::EventLoop::<AppEvent>::with_user_event().build()?;
	let proxy = event_loop.create_proxy();
	let mut app = App::new(WindowConfig::default(), proxy)?;

	event_loop.run_app(&mut app)?;

	if let Some(error) = app.error.take() {
		return Err(Box::new(error));
	}

	Ok(())
}
