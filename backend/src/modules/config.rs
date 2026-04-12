use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    pub integrations: Integrations,
}

#[derive(Deserialize, Default, Clone)]
pub struct Integrations {
    #[serde(default)]
    pub alacritty: bool,
    #[serde(default)]
    pub fuzzel: bool,
}

pub fn load_config() -> Config {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    path.push("niri-quickshell/config.toml");

    if let Ok(content) = fs::read_to_string(path) {
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}
