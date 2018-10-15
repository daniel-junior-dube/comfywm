pub mod keybinding;
pub mod parser;

use self::keybinding::Keybindings;
use std::env::var;
use std::path::Path;

const SYSTEM_DEFAULT_CONFIG_LOCATION: &str = "/etc/comfywm/";
const USER_DEFAULT_CONFIG_LOCATION: &str = "/.config/confywm/";
const KEYBINDING_FILE_NAME: &str = "keybindings.toml";
const THEME_FILE_NAME: &str = "theme.toml";

pub struct Config {
	pub keybindings: Keybindings,
}

impl Config {
	pub fn load() -> Config {
		let keybindings_file_path = find_config_file(KEYBINDING_FILE_NAME);
		let theme_file_path = find_config_file(THEME_FILE_NAME);

		Config {
			keybindings: Keybindings::new(),
		}
	}
}

fn find_config_file(config_type: &str) -> String {
	let user_home_directory = String::from(var("HOME").unwrap());
	let user_configs_directory = format!("{}{}", user_home_directory, USER_DEFAULT_CONFIG_LOCATION);

	let mut config_file_path = format!("{}{}", user_configs_directory, config_type);

	if cfg!(debug_assertions) {
		format!("./config/{}", config_type)
	} else {
		if Path::new(&config_file_path).exists() {
			println!(
				"The config file \"{}\" was not found in the user's configuration directory (~/.config/confywm/).
                The default system configuration will be used instead.",
				config_type
			);

			format!("{}{}", SYSTEM_DEFAULT_CONFIG_LOCATION, config_type)
		} else {
			config_file_path
		}
	}
}
