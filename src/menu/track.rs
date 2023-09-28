use core::fmt;
use std::{process::{Command, Stdio}, str::FromStr, num::ParseIntError, sync::Arc};

use async_trait::async_trait;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::{Device, SimplifiedTrack, FullTrack}
};
use strum::{IntoEnumIterator, EnumIter};
use std::env;

use crate::config::Config;

use super::{Menu, MenuProgram, MenuResult, selection_index};

pub struct TrackMenu {
    client: Arc<AuthCodePkceSpotify>,
    tracks: Vec<FullTrack>
}

impl TrackMenu {
    pub async fn new(client: Arc<AuthCodePkceSpotify>, tracks: Vec<FullTrack>) -> TrackMenu {
        Self {
            client,
            tracks
        }
    }
}

#[async_trait]
impl Menu for TrackMenu {
    fn items(&self) -> Vec<String> {
        self.tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let artist_names: Vec<&str> = track.artists.iter().map(|artist| artist.name.as_str()).collect();
                format!("{}: {} | {} | {}", i, track.name, track.album.name, artist_names.join(" "))
            })
            .collect()
    }

    async fn select(&self, program: MenuProgram) -> MenuResult {
        let selection = self.prompt(program);
        let parsed_index = selection_index(selection);

        match parsed_index {
            Ok(index) => {
                let track = &self.tracks[index];

                let config = match Config::load() {
                    Ok(config) => config,
                    Err(_) => Config { device_id: String::new() }
                };

                if let Some(id) = &track.id {
                    self.client.start_uris_playback(
                        [PlayableId::Track(id.clone())].iter().map(PlayableId::as_ref),
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
