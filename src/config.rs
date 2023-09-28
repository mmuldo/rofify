use std::{fs::File, error::Error};

use serde::{Serialize, Deserialize};

static CONFIG_FILE: &str = "config.yml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub device_id: String
}

impl Config {
    pub fn load() -> Result<Config, Box<dyn Error>> {
        let config_file = File::open(CONFIG_FILE)?;
        let config = serde_yaml::from_reader(config_file)?;
        Ok(config)
    }

    pub fn save(&self) {
        let config_file = File::create(CONFIG_FILE).unwrap();
        serde_yaml::to_writer(config_file, self).unwrap()
    }
}
