#[derive(Debug, Clone)]
pub enum WindowCommand {
	RequestRedraw,
	SetTitle(String),
	Close,
}
