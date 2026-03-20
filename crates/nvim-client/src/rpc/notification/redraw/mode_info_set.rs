use serde::Deserialize;

use crate::ModeInfo;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ModeInfoSetEvent {
	pub cursor_style_enabled: bool,
	pub mode_info: Vec<ModeInfo>,
}
