use nvui_derive::DeserializeTaggedEnum;
use serde::Deserialize;

#[derive(Debug, DeserializeTaggedEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum RedrawNotification {
	GridResize(#[tagged_enum(flatten)] Vec<GridResizeEvent>),

	Other {
		method: String,
		#[tagged_enum(flatten)]
		value: Vec<rmpv::Value>,
	},
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GridResizeEvent {
	pub grid: u32,
	pub width: u32,
	pub height: u32,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn de_grid_resize() {
		let expected = RedrawNotification::GridResize(vec![
			GridResizeEvent { grid: 1, width: 20, height: 20 },
			GridResizeEvent { grid: 2, width: 40, height: 40 },
		]);

		let event = vec![
			0x93, 0xAB, 0x67, 0x72, 0x69, 0x64, 0x5F, 0x72, 0x65, 0x73, 0x69, 0x7A, 0x65, 0x93,
			0x01, 0x14, 0x14, 0x93, 0x02, 0x28, 0x28,
		];

		let deserialized: RedrawNotification = rmp_serde::from_slice(&event).unwrap();

		assert_eq!(deserialized, expected)
	}

	#[test]
	fn de_other() {
		let expected = RedrawNotification::Other {
			method: String::from("unknown"),
			value: vec![
				rmpv::Value::from("simple_string"),
				rmpv::Value::from(vec![
					rmpv::Value::from("array_string_1"),
					rmpv::Value::from("array_string_2"),
				]),
			],
		};

		let event = vec![
			0x93, 0xA7, 0x75, 0x6E, 0x6B, 0x6E, 0x6F, 0x77, 0x6E, 0xAD, 0x73, 0x69, 0x6D, 0x70,
			0x6C, 0x65, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x92, 0xAE, 0x61, 0x72, 0x72,
			0x61, 0x79, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x5F, 0x31, 0xAE, 0x61, 0x72,
			0x72, 0x61, 0x79, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x5F, 0x32,
		];

		let deserialized: RedrawNotification = rmp_serde::from_slice(&event).unwrap();

		assert_eq!(deserialized, expected)
	}
}
