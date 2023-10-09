use std::fs::File;
use std::{io, result};

use serde::{Serialize, Deserialize};

const CONFIG_FILE: &str = "config.yml";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error)
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub device_id: String,
    pub redirect_uri_port: Option<u16>
}

impl Config {
    pub fn load() -> Result<Config> {
        let config_file = File::open(CONFIG_FILE)?;
        let config = serde_yaml::from_reader(config_file)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_file = File::create(CONFIG_FILE)?;
        serde_yaml::to_writer(config_file, self)?;
        Ok(())
    }
}
