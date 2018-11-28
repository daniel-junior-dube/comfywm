use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use toml;

/// An intermediate struct used to parse a Toml file
#[derive(Deserialize, Debug)]
struct TomlGlobal {
	pointer_focus_type: Option<String>,
}

pub struct Global {
	pub pointer_focus_type: PointerFocusType,
}

#[derive(PartialEq, Eq, ToString, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PointerFocusType {
	OnHover,
	OnClick,
}

impl Global {
	pub fn new() -> Self {
		Global {
			pointer_focus_type: PointerFocusType::OnHover,
		}
	}

	/// Read the content of a file and then returns the parsed content.
	pub fn load(mut config_file: File) -> Result<Self, String> {
		let mut file_content = String::new();

		if let Err(e) = config_file.read_to_string(&mut file_content) {
			Err(format!("Could not read the contents of the keybindings file: {}", e))
		} else {
			Global::parse_config_from_toml(&file_content)
		}
	}

	pub fn parse_config_from_toml(file_content: &str) -> Result<Self, String> {
		let mut global = Global::new();

		let parsed_content = match toml::from_str::<TomlGlobal>(file_content) {
			Err(e) => return Err(format!("Error parsing the toml content: {}", e)),
			Ok(content) => content,
		};

		if let Some(pointer_focus_type_str) = parsed_content.pointer_focus_type {
			match PointerFocusType::from_str(&pointer_focus_type_str) {
				Ok(pointer_focus_type) => global.pointer_focus_type = pointer_focus_type,
				Err(_) => error!(
					"The pointer focus type you specified is incorrect: {}",
					pointer_focus_type_str
				),
			}
		}

		Ok(global)
	}
}
