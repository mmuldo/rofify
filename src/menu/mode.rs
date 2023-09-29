use core::fmt;
use std::{process::{Command, Stdio}, str::FromStr, num::ParseIntError, sync::Arc};

use async_trait::async_trait;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::{Device, SearchType}
};
use strum::{IntoEnumIterator, EnumIter};
use std::env;

use super::{Menu, MenuProgram, MenuResult, device::DeviceMenu, search::SearchMenu};

#[derive(Debug, EnumIter)]
pub enum Mode {
    TrackSearch,
    AlbumSearch,
    PlaylistSearch,
    Device,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::TrackSearch => "Track Search",
            Self::AlbumSearch => "Album Search",
            Self::PlaylistSearch => "Playlist Search",
            Self::Device => "Device",
        };
        write!(f, "{text}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseModeError;

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Track Search" => Ok(Self::TrackSearch),
            "Album Search" => Ok(Self::AlbumSearch),
            "Playlist Search" => Ok(Self::PlaylistSearch),
            "Device" => Ok(Self::Device),
            _ => Err(ParseModeError)
        }
    }
}

pub struct ModeMenu {
    client: Arc<AuthCodePkceSpotify>
}

impl ModeMenu {
    pub fn new(client: Arc<AuthCodePkceSpotify>) -> ModeMenu {
        Self {
            client
        }
    }
}

#[async_trait]
impl Menu for ModeMenu {
    fn items(&self) -> Vec<String> {
        Mode::iter().map(|mode| mode.to_string()).collect()
    }

    async fn select(&self, program: MenuProgram) -> MenuResult {
        let selection = self.prompt(program);
        let parsed_mode = Mode::from_str(selection.as_str());

        match parsed_mode {
            Ok(mode) => match mode {
                Mode::TrackSearch => MenuResult::Menu(Box::new(
                    SearchMenu::new(Arc::clone(&self.client), SearchType::Track).await
                )),
                Mode::AlbumSearch => MenuResult::Menu(Box::new(
                    SearchMenu::new(Arc::clone(&self.client), SearchType::Album).await
                )),
                Mode::PlaylistSearch => MenuResult::Menu(Box::new(
                    SearchMenu::new(Arc::clone(&self.client), SearchType::Playlist).await
                )),
                Mode::Device => MenuResult::Menu(Box::new(
                    DeviceMenu::new(Arc::clone(&self.client)).await
                )),
            }
            Err(_) => {
                println!("{selection:#?} is not a valid mode");
                MenuResult::Back
            }
        }
    }
}
