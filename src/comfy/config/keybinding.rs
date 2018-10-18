use compositor::commands::Command;
use config::parser::convert_to_xkb_string;
use input::keyboard::XkbKeySet;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use toml;

/// An intermediate struct used to parse a Toml file
#[derive(Deserialize, Debug)]
struct TomlKeybindings {
	modkey: String,
	keybindings: HashMap<String, String>,
}

pub struct Keybindings {
	pub modkey: Vec<XkbKeySet>,
	pub bindings: HashMap<XkbKeySet, Command>,
}

impl Keybindings {
	pub fn new() -> Self {
		Keybindings {
			modkey: Vec::new(),
			bindings: HashMap::new(),
		}
	}

	/// Read the content of a file and then returns the parsed content.
	pub fn load(mut config_file: File) -> Result<Self, String> {
		let mut file_content = String::new();

		config_file
			.read_to_string(&mut file_content)
			.expect("Error reading file keybindings config file");
		Keybindings::parse_config_from_toml(&file_content)
	}

	/// Loads the config file from a specific path and converts it to Comfy's `Keybindings` object.
	///
	/// # Errors
	///
	/// Will crash if no keybindings are in the file or the section `keybindings` is missing.
	///
	/// Will crash if no modkey are in the file or is invalid (not a valid `XkbKeySet` or is two keys).
	///
	/// Will crash if a `Keybinding` is the same as the `Modkey`.
	///
	/// Will crash if a `Keybinding` is not a valid `XkbKeySet`.
	pub fn parse_config_from_toml(file_content: &str) -> Result<Self, String> {
		let mut keybindings = Keybindings::new();

		let parsed_content = match toml::from_str::<TomlKeybindings>(file_content) {
			Ok(ref content) if content.keybindings.is_empty() => {
				return Err("No bindings specified for the keybindings file".to_string());
			}
			Ok(ref content) if content.modkey.is_empty() => {
				return Err("No modkey specified for the keybindings file".to_string());
			}
			Ok(ref content) if content.modkey.contains("+") => {
				return Err("The modkey needs to be a single key.".to_string());
			}
			Ok(content) => content,
			Err(e) => return Err(format!("Error parsing the toml content: {}", e)),
		};

		let modkey_str = &parsed_content.modkey;
		let modkey_keyset_strs = convert_to_xkb_string(modkey_str, modkey_str)?;

		for modkey_keyset_str in modkey_keyset_strs.iter() {
			keybindings.modkey.push(XkbKeySet::from_str(modkey_keyset_str)?);
		}

		for (keys_str, command_str) in parsed_content.keybindings.iter() {
			let xkb_keysets_strs = convert_to_xkb_string(modkey_str, keys_str)?;

			for xkb_keyset_str in xkb_keysets_strs.iter() {
				if modkey_keyset_strs.contains(xkb_keyset_str) {
					return Err(format!("Command set to modkey! {} = {}", keys_str, command_str));
				} else {
					let xkb_keyset = XkbKeySet::from_str(xkb_keyset_str).unwrap();
					if command_str.is_empty() {
						return Err(format!("The command associated with {} is empty", &keys_str));
					}
					let command = Command::from_str(command_str)?;
					keybindings.bindings.insert(xkb_keyset, command);
				}
			}
		}

		Ok(keybindings)
	}
}
