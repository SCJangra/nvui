use nvui_serde::DeserializeTuple;

use crate::CursorMode;

#[derive(Debug, DeserializeTuple)]
#[cfg_attr(test, derive(PartialEq))]
pub enum RedrawNotification {
	GridResize(#[tuple(flatten)] GridResizeEvent),

	// [`ui-global`](https://neovim.io/doc/user/api-ui-events/#ui-global) events
	SetTitle(String),
	SetIcon(String),
	ModeInfoSet(#[tuple(flatten)] ModeInfoSetEvent),
}

#[derive(Debug, DeserializeTuple)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GridResizeEvent {
	pub grid: u64,
	pub width: u64,
	pub height: u64,
}

#[derive(Debug, DeserializeTuple)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ModeInfoSetEvent {
	pub cursor_style_enabled: bool,
	pub mode_info: CursorMode,
}
