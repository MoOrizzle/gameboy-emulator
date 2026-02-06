use std::{fs, path::Path};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GameboyConfig {
    window_size: u8,
    use_sound: bool,
}

impl GameboyConfig {
    fn new() -> Self {
        Self {
            window_size: 4,
            use_sound: true
        }
    }

    pub fn load_or_initialize() -> GameboyConfig {
        let config_path = Path::new("Config.toml");

        if config_path.exists() {
            let content = fs::read_to_string(config_path)
                .expect("An error occurred while loading the config");

            let config = toml::from_str(&content)
                .expect("An error occurred while parsing the config");

            return config;
        }

        let config = GameboyConfig::new();
        let toml = toml::to_string(&config).unwrap();

        fs::write(config_path, toml)
            .expect("An error occurred while creating default config");

        config
    }
}