mod redraw;

pub use redraw::*;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum NvimNotification {
	Redraw(Vec<RedrawNotification>),

	Other(rmpv::Value),
}

mod ser_de {
	use serde::de::{Error as DeError, SeqAccess, Visitor};
	use serde::{Deserialize, Deserializer};

	use super::*;

	struct NvimNotificationVisitor;

	impl<'de> Visitor<'de> for NvimNotificationVisitor {
		type Value = NvimNotification;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("a neovim notification tuple")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: SeqAccess<'de>,
		{
			let method =
				seq.next_element::<String>()?.ok_or_else(|| DeError::invalid_length(0, &self))?;

			match method.as_str() {
				"redraw" => {
					let events = seq
						.next_element::<Vec<RedrawNotification>>()?
						.ok_or_else(|| DeError::invalid_length(1, &self))?;

					Ok(NvimNotification::Redraw(events))
				},
				_ => {
					let other = seq
						.next_element::<rmpv::Value>()?
						.ok_or_else(|| DeError::invalid_length(1, &self))?;

					Ok(NvimNotification::Other(other))
				},
			}
		}
	}

	impl<'de> Deserialize<'de> for NvimNotification {
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: Deserializer<'de>,
		{
			deserializer.deserialize_seq(NvimNotificationVisitor)
		}
	}
}
