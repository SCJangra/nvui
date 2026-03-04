mod app;
mod error;

use std::thread;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let (command_tx, command_rx) = flume::unbounded();
	let (event_tx, event_rx) = flume::unbounded();

	let app = App::new(event_rx, command_tx);
	let app_thread = thread::spawn(move || app.start_event_loop());

	let result = win::start(win::WindowConfig::default(), command_rx, event_tx);

	let _ = app_thread.join();

	result.map_err(Into::into)
}
