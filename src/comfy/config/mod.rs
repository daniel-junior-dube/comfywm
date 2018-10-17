pub mod keybinding;
pub mod parser;

use self::keybinding::Keybindings;
use std::fs::File;
use std::env::var;
use std::path::Path;

const SYSTEM_DEFAULT_KEYBINDINGS: &str = "/etc/comfywm/keybindings.toml";
const USER_KEYBINDINGS: &str = "/.config/comfywm/keybindings.toml";
// TODO: import theme config
// const SYSTEM_DEFAULT_THEME: &str = "/etc/comfywm/theme.toml";
// const USER__THEME: &str = "/.config/comfywm/theme.toml";

pub struct Config {
	pub keybindings: Keybindings,
}

impl Config {
	pub fn load() -> Result<Config, String> {
		let keybindings = if var("HOME").is_ok() {
			match File::open(format!("{}{}", var("HOME").unwrap(), USER_KEYBINDINGS)) {
						Ok(user_config_file) => {
							match Keybindings::load(user_config_file) {
								Ok(keybinding) => keybinding,
								Err(_) => {
									// TODO: log warn error in user config
									load_system_default_keybindings()
								}
							}
						},
						Err(_) => {
							// TODO: log info using system's defaults
							load_system_default_keybindings()
						}
			}
		} else {
			load_system_default_keybindings()
		};

		Ok(Config {
			keybindings: keybindings
		})
	}
}

fn load_system_default_keybindings() -> Keybindings {
	let system_default_keybindings = File::open(SYSTEM_DEFAULT_KEYBINDINGS)
		.expect("Fatal error: Could open keybindings config!");
	Keybindings::load(system_default_keybindings)
		.expect("Fatal error: could not load any keybindings config files!")
}


#[cfg(test)]
mod tests {
	use super::*;
	use common::command_type::CommandType;
	use compositor::commands::Command;
	use input::keyboard::XkbKeySet;

	#[test]
	fn generate_valid_config() {
		let config = r#"modkey = "Control"

			[keybindings]
			"$mod+Shift+Up" = "exec weston-terminal"
			"#;
		match Keybindings::parse_config_from_toml(config) {
			Ok(keybinding) => {
				let expected_bindings: Vec<&str> = vec![
					"Control_R+Shift_R+Up",
					"Control_R+Shift_L+Up",
					"Control_L+Shift_R+Up",
					"Control_L+Shift_L+Up"];
				let expected_command_type = CommandType::Exec;
				let expected_command_args = vec!["weston-terminal".to_string()];
				for binding in expected_bindings.iter() {
					let xkb_keyset = XkbKeySet::from_str(binding).unwrap();
					let command: &Command = keybinding.bindings.get(&xkb_keyset)
						.expect(&format!("The command {} should exist", binding));
					assert_eq!(command.command_type, expected_command_type);
					assert_eq!(command.args, expected_command_args);
				}
			},
			Err(e) => {
				panic!(e);
			}
		};
	}

	#[test]
	fn generate_config_with_invalid_modkey() {
		let empty_modkey = r#"modkey = ""
			[keybindings]
			"$mod+Shift+Up" = "exec weston-terminal"
			"#;
		let no_modkey = r#"[keybindings]
			"$mod+Shift+Up" = "exec weston-terminal"
			"#;
		let invalid_modkey_keyset = r#"modkey = "Heck"
			[keybindings]
			"$mod+Shift+Up" = "exec weston-terminal"
			"#;
		assert!(Keybindings::parse_config_from_toml(empty_modkey).is_err());
		assert!(Keybindings::parse_config_from_toml(no_modkey).is_err());
		assert!(Keybindings::parse_config_from_toml(invalid_modkey_keyset).is_err());
	}

	#[test]
	fn generate_config_with_invalid_keybindings() {
		let modkey_as_binding_using_mod = r#"modkey = "Control"
			[keybindings]
			"$mod" = "exec weston-terminal"
			"#;
		let modkey_as_binding = r#"modkey = "Control"
			[keybindings]
			"Control" = "exec weston-terminal"
			"#;
		let invalid_keyset = r#"modkey = "Control"
			[keybindings]
			"Heck" = "exec weston-terminal"
			"#;
		let invalid_command = r#"modkey = "Control"
			[keybindings]
			"$mod+Shift+Up" = "heck weston-terminal"
			"#;
		let empty_command = r#"modkey = "Control"
			[keybindings]
			"$mod+Shift+Up" = ""
			"#;
		let no_keybindings = r#"modkey = "Control"
			[keybindings]
			"#;
		let no_keybindings_section = r#"modkey = "Control"
		"$mod+Shift+Up" = "exec weston-terminal"
		"#;
		assert!(Keybindings::parse_config_from_toml(modkey_as_binding_using_mod).is_err());
		assert!(Keybindings::parse_config_from_toml(modkey_as_binding).is_err());
		assert!(Keybindings::parse_config_from_toml(invalid_keyset).is_err());
		assert!(Keybindings::parse_config_from_toml(invalid_command).is_err());
		assert!(Keybindings::parse_config_from_toml(empty_command).is_err());
		assert!(Keybindings::parse_config_from_toml(no_keybindings).is_err());
		assert!(Keybindings::parse_config_from_toml(no_keybindings_section).is_err());
	}
}