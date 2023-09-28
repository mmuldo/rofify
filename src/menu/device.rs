use core::fmt;
use std::{process::{Command, Stdio}, str::FromStr, num::ParseIntError, sync::Arc};

use async_trait::async_trait;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::Device
};
use strum::{IntoEnumIterator, EnumIter};
use std::env;

use crate::config::Config;

use super::{Menu, MenuProgram, MenuResult, selection_index};

pub struct DeviceMenu {
    client: Arc<AuthCodePkceSpotify>,
    devices: Vec<Device>,
}

impl DeviceMenu {
    pub async fn new(client: Arc<AuthCodePkceSpotify>) -> DeviceMenu {
        let devices = client.device().await.unwrap();
        Self {
            client,
            devices,
        }
    }
}

#[async_trait]
impl Menu for DeviceMenu {
    fn items(&self) -> Vec<String> {
        self.devices
            .iter()
            .enumerate()
            .map(|(i, device)| {
                format!("{}: {}", i, device.name)
            })
            .collect()
    }

    async fn select(&self, program: MenuProgram) -> MenuResult {
        let selection = self.prompt(program);
        let parsed_index = selection_index(selection);

        match parsed_index {
            Ok(index) => {
                let device = &self.devices[index];
                let device_id = device.id.clone().unwrap();
                let mut config = match Config::load() {
                    Ok(config) => config,
                    Err(_) => Config { device_id: String::new() }
                };
                config.device_id = device_id;
                config.save();
                MenuResult::Exit
            }
            Err(_) => {
                println!("failed to get index of selected item");
                MenuResult::Back
            }
        }
    }
}
