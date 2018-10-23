/*
..####....####...##.......####...#####....####..
.##..##..##..##..##......##..##..##..##..##.....
.##......##..##..##......##..##..#####....####..
.##..##..##..##..##......##..##..##..##......##.
..####....####...######...####...##..##...####..
................................................
*/

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
}

pub enum Color {
	Rgba(RgbaColor),
	Hex(HexColor),
}

impl Color {
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
