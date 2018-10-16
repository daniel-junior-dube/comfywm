use common::command_type::CommandType;
use compositor::commands::Command;
use input::keyboard::XkbKeySet;
use serde_derive;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use toml;
use config::parser::convert_to_xkb_string;

#[derive(Deserialize, Debug)]
struct TomlKeybindings {
	modkey: String,
	keybindings: HashMap<String, String>,
}

pub struct Keybindings {
	bindings: HashMap<XkbKeySet, Command>,
}

impl Keybindings {
	pub fn new() -> Self {
		Keybindings {
			bindings: HashMap::new(),
		}
	}

	pub fn load(path: String) -> Result<Self, String> {
		let mut keybindings = Keybindings::new();
		let mut config_file = File::open(path.clone()).unwrap();
		let mut file_content = String::new();

		config_file.read_to_string(&mut file_content).unwrap();

		let parsed_content: TomlKeybindings = toml::from_str(&file_content).expect(&format!("Error in the file {}", path));

		let modkey_str = &parsed_content.modkey;
		let modkey_keyset_strs = convert_to_xkb_string(modkey_str, modkey_str)?;

		for (keys_str, command_str) in parsed_content.keybindings.iter() {
			let xkb_keysets_strs = convert_to_xkb_string(modkey_str, keys_str)?;

			for xkb_keyset_str in xkb_keysets_strs.iter() {
				if modkey_keyset_strs.contains(xkb_keyset_str) {
					return Err(format!("Command set to modkey! \"{} = {}\"", keys_str, command_str));
				} else {
					let xkb_keyset = XkbKeySet::from_str(xkb_keyset_str).unwrap();
					let command = Command::from_str(command_str)?;
					keybindings.bindings.insert(xkb_keyset, command);
				}
			}
		}

		Ok(keybindings)
	}
}
