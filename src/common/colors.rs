/*
..####....####...##.......####...#####....####..
.##..##..##..##..##......##..##..##..##..##.....
.##......##..##..##......##..##..#####....####..
.##..##..##..##..##......##..##..##..##......##.
..####....####...######...####...##..##...####..
................................................
*/

use regex::Regex;

pub struct RgbaColor {
	r: f32,
	g: f32,
	b: f32,
	a: f32,
}

impl RgbaColor {
	pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
		RgbaColor { r, g, b, a }
	}

	pub fn as_slice(&self) -> [f32; 4] {
		[self.r, self.g, self.b, self.a]
	}
}

pub struct HexColor {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

impl HexColor {
	pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
		HexColor { r, g, b, a }
	}

	pub fn as_slice(&self) -> [u8; 4] {
		[self.r, self.g, self.b, self.a]
	}

	pub fn to_rgba(&self) -> RgbaColor {
		RgbaColor {
			r: self.r as f32 / 255.0,
			g: self.g as f32 / 255.0,
			b: self.b as f32 / 255.0,
			a: self.a as f32 / 255.0,
		}
	}

	pub fn from_str(hexcode_str: &str) -> Result<HexColor, String> {
		lazy_static! {
			static ref re: Regex = Regex::new(r"#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})?").unwrap();
		}
		if re.is_match(hexcode_str) {
			let groups = re.captures(hexcode_str).unwrap();
			let r = u8::from_str_radix(groups.get(1).unwrap().as_str(), 16).unwrap();
			let g = u8::from_str_radix(groups.get(2).unwrap().as_str(), 16).unwrap();
			let b = u8::from_str_radix(groups.get(3).unwrap().as_str(), 16).unwrap();
			let a = if let Some(a_str) = groups.get(4) {
				u8::from_str_radix(a_str.as_str(), 16).unwrap()
			} else {
				255
			};
			Ok(HexColor::new(r, g, b, a))
		} else {
			Err(format!("Invalid hexadecimal format {}", hexcode_str))
		}
	}
}

pub enum Color {
	Rgba(RgbaColor),
	Hex(HexColor),
}

impl Color {
	pub fn black() -> Self {
		return Color::Rgba(RgbaColor::new(0.0, 0.0, 0.0, 1.0));
	}

	pub fn bunker() -> Self {
		return Color::Rgba(RgbaColor::new(44.0 / 255.0, 49.0 / 255.0, 55.0 / 255.0, 1.0));
	}

	pub fn burgundy() -> Self {
		return Color::Rgba(RgbaColor::new(135.0 / 255.0, 7.0 / 255.0, 52.0 / 255.0, 1.0));
	}

	pub fn as_rgba_slice(&self) -> [f32; 4] {
		match self {
			Color::Rgba(rgba_color) => rgba_color.as_slice(),
			Color::Hex(hex_color) => hex_color.to_rgba().as_slice(),
		}
	}
}
