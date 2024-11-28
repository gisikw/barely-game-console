use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub rfid_cards: std::collections::HashMap<String, CardInfo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CardInfo {
    pub rom_path: String,
    pub emulator: String,
    pub artwork: String,
}

impl Config {
    pub fn load() -> Self {
        let config = fs::read_to_string("config.toml").expect("Failed to read config.toml");
        toml::from_str(&config).expect("Failed to parse config.toml")
    }
}
