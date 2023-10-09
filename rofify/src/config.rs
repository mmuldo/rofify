use serde::{Serialize, Deserialize};

const APP_NAME: &str = "rofify";
const CONFIG_NAME: &str = "config";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub device_id: Option<String>,
    pub redirect_uri_port: Option<u16>
}

impl Config {
    pub fn load() -> Result<Config, confy::ConfyError> {
        confy::load(APP_NAME, CONFIG_NAME)
    }

    pub fn store(self) -> Result<(), confy::ConfyError> {
        confy::store(APP_NAME, CONFIG_NAME, self)
    }
}
