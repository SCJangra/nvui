use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GridLineEvent {
	pub grid: u32,
	pub row: u32,
	pub col_start: u32,
	pub cells: Vec<GridCell>,
	pub wrap: bool,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GridCell {
	pub text: char,
	#[serde(default)]
	pub hl_id: Option<u32>,
	#[serde(default)]
	pub repeat: Option<u32>,
}

#[cfg(test)]
mod tests {
	use super::{GridCell, GridLineEvent};
	use crate::rpc::notification::redraw::RedrawNotification;

	#[test]
	fn de_grid_line() {
		let expected = RedrawNotification::GridLine(vec![GridLineEvent {
			grid: 1,
			row: 0,
			col_start: 0,
			cells: vec![
				GridCell { text: 'a', hl_id: Some(1), repeat: Some(2) },
				GridCell { text: 'b', hl_id: None, repeat: None },
				GridCell { text: 'c', hl_id: Some(3), repeat: None },
			],
			wrap: true,
		}]);

		let event = vec![
			0x92, 0xA9, 0x67, 0x72, 0x69, 0x64, 0x5F, 0x6C, 0x69, 0x6E, 0x65, 0x95, 0x01, 0x00,
			0x00, 0x93, 0x93, 0xA1, 0x61, 0x01, 0x02, 0x91, 0xA1, 0x62, 0x92, 0xA1, 0x63, 0x03,
			0xC3,
		];

		let deserialized: RedrawNotification = rmp_serde::from_slice(&event).unwrap();

		assert_eq!(deserialized, expected)
	}
}
