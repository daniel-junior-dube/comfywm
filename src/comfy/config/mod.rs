pub mod global;
pub mod keybinding;
pub mod parser;
pub mod theme;

use self::global::Global;
use self::keybinding::Keybindings;
use self::theme::Theme;
use std::env::var;
use std::fs::File;

const SYSTEM_DEFAULT_KEYBINDINGS_PATH: &str = "/etc/comfywm/keybindings.toml";
const USER_KEYBINDINGS_PATH: &str = "/.config/comfywm/keybindings.toml";
const SYSTEM_DEFAULT_CONFIG_PATH: &str = "/etc/comfywm/global.toml";
const USER_GLOBAL_CONFIG_PATH: &str = "/.config/comfywm/global.toml";
const SYSTEM_DEFAULT_THEME_PATH: &str = "/etc/comfywm/theme.toml";
const USER_THEME_PATH: &str = "/.config/comfywm/theme.toml";

pub struct Config {
	pub keybindings: Keybindings,
	pub global: Global,
	pub theme: Theme,
}

impl Config {
	/// Returns a Comfy's config object. At first it tries to load the user's config in `"$HOME/.config/comfywm/"`. If
	/// it fails to parse it or the file is not there, it loads the system defaults situated in `/etc/comfywm/`. If that
	/// fails once again Comfy cannot start because it is not properly installed.
	pub fn load() -> Config {
		let keybindings = match load_user_keybindings() {
			Ok(keybindings) => keybindings,
			Err(e) => {
				warn!("Could not load the user's keybinding config: {}", e);
				load_system_default_keybindings()
			}
		};

		let global = match load_user_global_config() {
			Ok(global) => global,
			Err(e) => {
				warn!("Could not load the user's keybinding config: {}", e);
				load_system_default_global()
			}
		};

		let theme = match load_user_theme_config() {
			Ok(theme) => theme,
			Err(e) => {
				warn!("Could not load the user's theme config: {}", e);
				load_system_default_theme()
			}
		};

		Config {
			keybindings,
			global,
			theme,
		}
	}

	pub fn reload_config() -> Result<Config, String> {
		let keybindings = load_user_keybindings()?;
		let global = load_user_global_config()?;
		let theme = load_user_theme_config()?;
		Ok(Config {
			keybindings,
			global,
			theme,
		})
	}
}

fn load_user_theme_config() -> Result<Theme, String> {
	let user_home_path = var("HOME");
	if !user_home_path.is_ok() {
		return Err("No HOME variable set for the current user.".to_string());
	}

	match File::open(format!("{}{}", user_home_path.unwrap(), USER_THEME_PATH)) {
		Ok(user_config_file) => match Theme::load(user_config_file) {
			Ok(theme) => Ok(theme),
			Err(e) => Err(format!("The user's theme configuration contained error(s): {}", e)),
		},
		Err(e) => Err(format!("Could not open the user's theme file: {}", e)),
	}
}

fn load_system_default_theme() -> Theme {
	info!("Loading system default theme configuration");
	let system_default_theme_config =
		File::open(SYSTEM_DEFAULT_THEME_PATH).expect("Fatal error: Could open theme config!");
	Theme::load(system_default_theme_config).expect("Fatal error: could not load any theme config files!")
}

fn load_user_global_config() -> Result<Global, String> {
	let user_home_path = var("HOME");
	if !user_home_path.is_ok() {
		return Err("No HOME variable set for the current user.".to_string());
	}

	match File::open(format!("{}{}", user_home_path.unwrap(), USER_GLOBAL_CONFIG_PATH)) {
		Ok(user_config_file) => match Global::load(user_config_file) {
			Ok(global) => Ok(global),
			Err(e) => Err(format!("The user's global configuration contained error(s): {}", e)),
		},
		Err(e) => Err(format!("Could not open the user's global file: {}", e)),
	}
}

fn load_system_default_global() -> Global {
	info!("Loading system default global configuration");
	let system_default_global_config =
		File::open(SYSTEM_DEFAULT_CONFIG_PATH).expect("Fatal error: Could open global config!");
	Global::load(system_default_global_config).expect("Fatal error: could not load any global config files!")
}

fn load_user_keybindings() -> Result<Keybindings, String> {
	let user_home_path = var("HOME");
	if !user_home_path.is_ok() {
		return Err("No HOME variable set for the current user.".to_string());
	}

	match File::open(format!("{}{}", user_home_path.unwrap(), USER_KEYBINDINGS_PATH)) {
		Ok(user_config_file) => match Keybindings::load(user_config_file) {
			Ok(keybindings) => Ok(keybindings),
			Err(e) => Err(format!("The user's keybinding configuration contained error(s): {}", e)),
		},
		Err(e) => Err(format!("Could not open the user's keybinding file: {}", e)),
	}
}

fn load_system_default_keybindings() -> Keybindings {
	info!("Loading system default keybinding configuration");
	let system_default_keybindings =
		File::open(SYSTEM_DEFAULT_KEYBINDINGS_PATH).expect("Fatal error: Could open keybindings config!");
	Keybindings::load(system_default_keybindings).expect("Fatal error: could not load any keybindings config files!")
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
					"Control_L+Shift_L+Up",
				];
				let expected_modkeys: Vec<&str> = vec!["Control_R", "Control_L"];
				let expected_command_type = CommandType::Exec;
				let expected_command_args = vec!["weston-terminal".to_string()];

				for modkey_str in expected_modkeys.iter() {
					assert!(keybinding.modkey.contains(&XkbKeySet::from_str(modkey_str).unwrap()))
				}

				for binding in expected_bindings.iter() {
					let xkb_keyset = XkbKeySet::from_str(binding).unwrap();
					let command: &Command = keybinding
						.bindings
						.get(&xkb_keyset)
						.expect(&format!("The command {} should exist", binding));
					assert_eq!(command.command_type, expected_command_type);
					assert_eq!(command.args, expected_command_args);
				}
			}
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
		let invalid_modkey_keyset_2 = r#"modkey = "Control+Down"
			[keybindings]
			"$mod+Shift+Up" = "exec weston-terminal"
		"#;
		assert!(Keybindings::parse_config_from_toml(empty_modkey).is_err());
		assert!(Keybindings::parse_config_from_toml(no_modkey).is_err());
		assert!(Keybindings::parse_config_from_toml(invalid_modkey_keyset).is_err());
		assert!(Keybindings::parse_config_from_toml(invalid_modkey_keyset_2).is_err());
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
