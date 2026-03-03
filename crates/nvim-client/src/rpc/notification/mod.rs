mod redraw;

use nvui_derive::DeserializeTaggedEnum;

pub use redraw::*;

#[derive(Debug, DeserializeTaggedEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum NvimNotification {
	Redraw(Vec<RedrawNotification>),

	Other { method: String, value: rmpv::Value },
}
