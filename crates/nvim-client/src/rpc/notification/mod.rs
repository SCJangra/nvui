mod redraw;

pub use redraw::*;

use nvui_serde::DeserializeTuple;

#[derive(Debug, DeserializeTuple)]
#[cfg_attr(test, derive(PartialEq))]
pub enum NvimNotification {
	Redraw(RedrawNotification),
}
