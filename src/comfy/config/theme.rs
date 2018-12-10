use common::colors::*;
use std::fs::File;
use std::io::prelude::*;
use toml;

/// An intermediate struct used to parse a Toml file
#[derive(Deserialize, Debug)]
struct TomlTheme {
	border_size: Option<u8>,
	active_color: Option<String>,
	inactive_color: Option<String>,
	cursor_indicator_color: Option<String>,
	wallpaper_path: Option<String>,
}

pub struct Theme {
	pub border_size: u8,
	pub active_color: RgbaColor,
	pub inactive_color: RgbaColor,
	pub cursor_indicator_color: RgbaColor,
	pub wallpaper_path: String,
}

impl Theme {
	pub fn new() -> Self {
		Theme {
			border_size: 7,
			active_color: RgbaColor::new(245.0 / 255.0, 147.0 / 255.0, 17.0 / 255.0, 1.0),
			inactive_color: RgbaColor::new(41.0 / 255.0, 49.0 / 255.0, 46.0 / 255.0, 1.0),
			cursor_indicator_color: RgbaColor::new(230.0 / 255.0, 52.0 / 255.0, 42.0 / 255.0, 1.0),
			wallpaper_path: "/usr/share/comfywm/wallpaper.jpg".to_string(),
		}
	}

	/// Read the content of a file and then returns the parsed content.
	pub fn load(mut config_file: File) -> Result<Self, String> {
		let mut file_content = String::new();

		if let Err(e) = config_file.read_to_string(&mut file_content) {
			Err(format!("Could not read the contents of the theme file: {}", e))
		} else {
			Theme::parse_theme_from_toml(&file_content)
		}
	}

	pub fn parse_theme_from_toml(file_content: &str) -> Result<Self, String> {
		let mut theme = Theme::new();

		let parsed_content = match toml::from_str::<TomlTheme>(file_content) {
			Err(e) => return Err(format!("Error parsing the toml content: {}", e)),
			Ok(content) => content,
		};

		if let Some(border_size) = parsed_content.border_size {
			theme.border_size = border_size;
		}

		if let Some(active_color_str) = parsed_content.active_color {
			match HexColor::from_str(active_color_str.as_str()) {
				Ok(active_color) => theme.active_color = active_color.to_rgba(),
				Err(e) => warn!("Invalid active color {}", e),
			}
		}

		if let Some(inactive_color_str) = parsed_content.inactive_color {
			match HexColor::from_str(inactive_color_str.as_str()) {
				Ok(inactive_color) => theme.inactive_color = inactive_color.to_rgba(),
				Err(e) => warn!("Invalid inactive color {}", e),
			}
		}

		if let Some(cursor_indicator_color_str) = parsed_content.cursor_indicator_color {
			match HexColor::from_str(cursor_indicator_color_str.as_str()) {
				Ok(cursor_indicator_color) => theme.cursor_indicator_color = cursor_indicator_color.to_rgba(),
				Err(e) => warn!("Invalid cursor indicator color {}", e),
			}
		}

		if let Some(wallpaper_path) = parsed_content.wallpaper_path {
			theme.wallpaper_path = wallpaper_path;
		}

		Ok(theme)
	}
}
