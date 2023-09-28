use core::fmt;
use std::{process::{Command, Stdio}, str::FromStr, num::ParseIntError, sync::Arc};

use async_trait::async_trait;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::{Device, SimplifiedAlbum}
};
use strum::{IntoEnumIterator, EnumIter};
use std::env;

use crate::config::Config;

use super::{Menu, MenuProgram, MenuResult, selection_index};

pub struct AlbumMenu {
    client: Arc<AuthCodePkceSpotify>,
    albums: Vec<SimplifiedAlbum>
}

impl AlbumMenu {
    pub async fn new(client: Arc<AuthCodePkceSpotify>, albums: Vec<SimplifiedAlbum>) -> AlbumMenu {
        Self {
            client,
            albums
        }
    }
}

#[async_trait]
impl Menu for AlbumMenu {
    fn items(&self) -> Vec<String> {
        self.albums
            .iter()
            .enumerate()
            .map(|(i, album)| {
                let artist_names: Vec<&str> = album.artists.iter().map(|artist| artist.name.as_str()).collect();
                format!("{}: {} | {}", i, album.name, artist_names.join(" "))
            })
            .collect()
    }

    async fn select(&self, program: MenuProgram) -> MenuResult {
        let selection = self.prompt(program);
        let parsed_index = selection_index(selection);

        match parsed_index {
            Ok(index) => {
                let album = &self.albums[index];

                let config = match Config::load() {
                    Ok(config) => config,
                    Err(_) => Config { device_id: String::new() }
                };

                if let Some(id) = &album.id {
                    self.client.start_context_playback(
                        PlayContextId::Album(id.clone()),
                        Some(&config.device_id),
                        None,
                        None
                    ).await;
                }

                MenuResult::Exit
            }
            Err(_) => {
                println!("failed to get index of selected item");
                MenuResult::Back
            }
        }
    }
}
