use dashmap::DashMap;
use nvim::{DefaultColorsSetEvent, HlAttrDefineEvent, HlAttrs, HlGroupSetEvent};

const DEFAULT_BACKGROUND: u32 = 0x000000;

#[derive(Debug)]
pub struct HighlightState {
	attrs: DashMap<u32, HlAttrs>,
	default_background: u32,
	groups: DashMap<String, u32>,
}

impl HighlightState {
	pub fn new() -> Self {
		Self {
			attrs: DashMap::new(),
			default_background: DEFAULT_BACKGROUND,
			groups: DashMap::new(),
		}
	}

	pub fn set_attrs(&self, attrs: Vec<HlAttrDefineEvent>) {
		attrs.into_iter().for_each(|attr| {
			self.attrs.insert(attr.id, attr.rgb_attr);
		});
	}

	pub fn set_default_colors(&mut self, colors: Vec<DefaultColorsSetEvent>) {
		colors.into_iter().for_each(|color| {
			self.default_background = color.rgb_bg;
		});
	}

	pub fn set_hl_groups(&self, groups: Vec<HlGroupSetEvent>) {
		groups.into_iter().for_each(|group| {
			self.groups.insert(group.name, group.hl_id);
		});
	}
}
