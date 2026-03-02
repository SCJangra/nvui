use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
pub enum CursorShape {
	Block,
	Horizontal,
	Vertical,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct CursorMode {
	pub cursor_shape: Option<CursorShape>,
}
