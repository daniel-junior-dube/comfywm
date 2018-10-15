use compositor::commands::Command;
use input::keyboard::XkbKeySet;
use std::collections::HashMap;

pub struct Keybindings {
	bindings: HashMap<XkbKeySet, Command>,
}

impl Keybindings {
	pub fn new() -> Self {
		Keybindings {
			bindings: HashMap::new(),
		}
	}

	pub fn load(path: String) -> Self {
		let keybindings = Keybindings::new();
		// TODO: Parse the file

		keybindings
	}
}
