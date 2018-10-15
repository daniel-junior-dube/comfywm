use common::command_type::CommandType;
use compositor::commands::Command;
use input::keyboard::XkbKeySet;
use serde_derive;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use toml;

#[derive(Deserialize, Debug)]
struct TomlKeybindings {
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

		for (keys, command) in parsed_content.keybindings.iter() {
			let xkb_keyset = XkbKeySet::from_str(keys)?;
			let command = Command::from_str(command)?;

			println!("{:?}", xkb_keyset);
			keybindings.bindings.insert(xkb_keyset, command);
		}


		Ok(keybindings)
	}
}
