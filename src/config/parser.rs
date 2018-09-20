use toml;
use std::fs::File;

const SYSTEM_DEFAULT_CONFIG_LOCATION: &str = "/etc/comfywm/";
const USER_DEFAULT_CONFIG_LOCATION: &str = "/.config/confywm/";
const CONFIG_FILE_NAME: &str = "config.toml";
const THEME_FILE_NAME: &str = "theme.toml";

pub fn load_config() {
    import_config_files();
}

fn import_config_files() {
    let user_home_directory = String::from(std::env::var("HOME").unwrap());
    let user_configs_directory = format!("{}{}", user_home_directory, USER_DEFAULT_CONFIG_LOCATION);

    let mut config_file_path = format!("{}{}", user_configs_directory, CONFIG_FILE_NAME);

    let config_file = match File::open(&config_file_path) {
        Err(_) => {
            println!("The config file was not found in the user's configuration directory (~/.config/confywm/). The default system configuration will be used instead.");

            config_file_path = format!("{}{}", SYSTEM_DEFAULT_CONFIG_LOCATION, CONFIG_FILE_NAME);
            File::open(&config_file_path).unwrap()
        },
        Ok(file) => file
    };

    println!("{}", config_file_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        load_config();
    }
}