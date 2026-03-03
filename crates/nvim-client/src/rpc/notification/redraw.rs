use serde::Deserialize;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum RedrawNotification {
	GridResize(Vec<GridResizeEvent>),

	Other { method: String, values: Vec<rmpv::Value> },
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GridResizeEvent {
	pub grid: u32,
	pub width: u32,
	pub height: u32,
}

mod ser_de {
	use serde::de::{Error as DeError, SeqAccess, Visitor};

	use super::*;

	struct RedrawNotificationVisitor;

	impl<'de> Visitor<'de> for RedrawNotificationVisitor {
		type Value = RedrawNotification;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("a redraw notification array")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: SeqAccess<'de>,
		{
			// First element: method name
			let method =
				seq.next_element::<String>()?.ok_or_else(|| DeError::invalid_length(0, &self))?;

			match method.as_str() {
				"grid_resize" => {
					let mut events = Vec::new();

					while let Some(event) = seq.next_element::<GridResizeEvent>()? {
						events.push(event);
					}

					Ok(RedrawNotification::GridResize(events))
				},
				_ => {
					let mut values = Vec::new();

					while let Some(value) = seq.next_element::<rmpv::Value>()? {
						values.push(value);
					}

					Ok(RedrawNotification::Other { method, values })
				},
			}
		}
	}

	impl<'de> Deserialize<'de> for RedrawNotification {
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			deserializer.deserialize_seq(RedrawNotificationVisitor)
		}
	}
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
			values: vec![
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
