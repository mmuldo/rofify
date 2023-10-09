use std::{
    sync::Arc,
    num::IntErrorKind
};

use async_trait::async_trait;
use notify::{
    notify,
    enotify
};
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::Device
};

use crate::config::Config;

use super::{
    Menu,
    MenuProgram,
    MenuResult,
    selection_index
};

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
        let parsed_index = selection_index(&selection);

        match parsed_index {
            Ok(index) => {
                let device = &self.devices[index];

                match device.id.clone() {
                    Some(id) => {
                        match Config::load() {
                            Ok(mut config) => {
                                config.device_id = Some(id.clone());

                                match config.store() {
                                    Ok(_) => {
                                        match self.client.transfer_playback(&id, Some(true)).await {
                                            Ok(_) => {
                                                notify(&format!("Device set to {}", device.name), "")
                                            },
                                            Err(error) => {
                                                enotify(&format!("Failed to switch playback to {}: {error}", device.name))
                                            }
                                        }
                                    },
                                    Err(error) => enotify(&format!("Failed to set device to {}: {error}", device.name))
                                };

                                MenuResult::Exit
                            },
                            Err(error) => {
                                enotify(&format!("Failed to load config: {error}"));
                                MenuResult::Back
                            },
                        }
                    },
                    None => {
                        enotify(&format!("Device {} has no ID", device.name));
                        MenuResult::Back
                    }
                }
            }
            Err(error) => {
                if error.kind().clone() != IntErrorKind::Empty {
                    enotify(&format!("Failed to get index of selected item {selection:#?}: {error}"))
                }
                MenuResult::Back
            }
        }
    }
}

pub async fn device_id(client: Arc<AuthCodePkceSpotify>, program: MenuProgram) -> Option<String> {
    match Config::load() {
        Ok(config) => {
            match config.device_id {
                Some(id) => Some(id),
                None => {
                    let _ = DeviceMenu::new(Arc::clone(&client))
                        .await
                        .select(program)
                        .await;

                    match Config::load() {
                        Ok(config) => config.device_id,
                        Err(error) => {
                            enotify(&format!("Failed to load device id from config: {error}"));
                            None
                        }
                    }
                }
            }
        },
        Err(error) => {
            enotify(&format!("Failed to load device id from config: {error}"));
            None
        }
    }
}
