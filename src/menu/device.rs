use std::{
    sync::Arc,
    num::IntErrorKind
};

use async_trait::async_trait;
use notify_rust::Notification;
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
        let mut notification = Notification::new();

        match parsed_index {
            Ok(index) => {
                let device = &self.devices[index];
                let device_id = device.id.clone().unwrap();
                let mut config = match Config::load() {
                    Ok(config) => config,
                    Err(_) => Config::default()
                };

                config.device_id = device_id;
                match config.save() {
                    Ok(_) => {
                        match self.client.transfer_playback(&config.device_id, Some(true)).await {
                            Ok(_) => {
                                notification.summary(format!("Device set to {}", device.name).as_str());
                            },
                            Err(error) => {
                                notification.summary("Error");
                                notification.body(format!("Failed to switch playback to {}: {error}", device.name).as_str());
                            }
                        }
                    },
                    Err(error) => {
                        notification.summary("Error");
                        notification.body(format!("Failed to set device to {}: {error}", device.name).as_str());
                    }
                };

                MenuResult::Exit(Some(notification))
            }
            Err(error) => {
                let maybe_notification = match error.kind() {
                    IntErrorKind::Empty => None,
                    _ => {
                        notification.summary("Error");
                        notification.body(format!("Failed to get index of selected item {selection:#?}: {error}").as_str());
                        Some(notification)
                    }
                };
                MenuResult::Back(maybe_notification)
            }
        }
    }
}
