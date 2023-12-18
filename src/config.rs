use log::info;
use serde::{Deserialize, Serialize};
use crate::tube;
use std::{fs::OpenOptions, path::PathBuf};

const CONFIG: &str = ".sonotube.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    api_key: Option<String>,
    create_sonotube_playlist: Option<bool>,
    send_previous_tracks: Option<bool>,
    create_toptastic_playlist: Option<bool>,
}

impl Config {
    pub fn new() -> Self {
        let config = match Config::load(CONFIG) {
            Some(config) => {
                if let Some(value) = &config.api_key {
                    std::env::set_var(tube::API_KEY_VAR, value)
                }
                config
            }
            None => Config {
                api_key: None,
                create_sonotube_playlist: None,
                send_previous_tracks: None,
                create_toptastic_playlist: None,
            },
        };
        config
    }

    pub fn create_play_list(&self) -> bool {
        match self.create_sonotube_playlist {
            Some(val) => val,
            None => false,
        }
    }

    pub fn send_previous_tracks(&self) -> bool {
        match self.send_previous_tracks {
            Some(val) => val,
            None => false,
        }
    }

    fn load(file_name: &str) -> Option<Self> {
        use std::fs;
        let config_path = Config::get_config_path(file_name);

        if !config_path.exists() {
            info!("Config file {:?} does not exist. Using defaults", &config_path);
            return None;
        }

        let serialized = fs::read_to_string(config_path).expect("Unable to load config");
        serde_json::from_str(&serialized).unwrap()
    }

    pub fn _save(&self, file_name: &str) {
        let config_path = Config::get_config_path(file_name);

        let log = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&config_path)
            .expect(format!("Unable to open {:?} for writing", &config_path).as_str());

        let mut serializer = serde_json::Serializer::new(log);
        self.serialize(&mut serializer)
            .expect("Unable to save config");
    }

    fn get_config_path(file_name: &str) -> PathBuf {
        let mut config_path = dirs::home_dir().expect("The home directory was not found.");
        config_path.push(file_name);
        config_path
    }
}

