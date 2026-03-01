use nvui_serde::{DeserializeTuple, SerializeTuple};

#[derive(Debug, PartialEq, SerializeTuple, DeserializeTuple)]
pub enum NvimNotification {
	Redraw(RedrawNotification),
}

#[derive(Debug, PartialEq, SerializeTuple, DeserializeTuple)]
pub enum RedrawNotification {
	GridResize(#[tuple(flatten)] GridResizeEvent),
}

#[derive(Debug, PartialEq, SerializeTuple, DeserializeTuple)]
pub struct GridResizeEvent {
	pub grid: u64,
	pub width: u64,
	pub height: u64,
}
