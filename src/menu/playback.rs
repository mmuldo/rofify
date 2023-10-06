use notify_rust::Notification;
use thiserror;
use std::{
    sync::Arc,
    result,
    num::IntErrorKind,
};

use async_trait::async_trait;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::{
        SimplifiedAlbum,
        FullTrack,
        SimplifiedPlaylist, FullArtist,
    },
};

use crate::config;

use super::{Menu, MenuProgram, MenuResult, selection_index};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("spotify client error: {0}")]
    Client(#[from] rspotify::ClientError),
    #[error("no id found for {0}")]
    NoId(String),
    #[error("error with config file: {0}")]
    Config(#[from] config::Error),
}

pub type Result<T> = result::Result<T, Error>;

pub struct PlaybackMenu<T> {
    client: Arc<AuthCodePkceSpotify>,
    items: Vec<T>
}

impl<T> PlaybackMenu<T> {
    pub async fn new(client: Arc<AuthCodePkceSpotify>, items: Vec<T>) -> PlaybackMenu<T> {
        Self {
            client,
            items
        }
    }
}

#[async_trait]
impl<T: ListItem + StartPlayback + Send + Sync> Menu for PlaybackMenu<T> {
    fn items(&self) -> Vec<String> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, item)| item.list_item(i))
            .collect()
    }

    async fn select(&self, program: MenuProgram) -> MenuResult {
        let selection = self.prompt(program);
        let parsed_index = selection_index(&selection);
        let mut notification = Notification::new();

        match parsed_index {
            Ok(index) => {
                let item = &self.items[index];

                match item.start_playback(Arc::clone(&self.client)).await {
                    Ok(_) => {
                        MenuResult::Exit(None)
                    },
                    Err(error) => {
                        notification.summary("Error");
                        notification.body(format!("Failed to start playback: {error}").as_str());
                        MenuResult::Back(Some(notification))
                    }
                }
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

pub trait ListItem {
    fn list_item(&self, index: usize) -> String;
}

#[async_trait]
pub trait StartPlayback {
    async fn start_playback(&self, client: Arc<AuthCodePkceSpotify>) -> Result<()>;
}

impl ListItem for SimplifiedAlbum {
    fn list_item(&self, index: usize) -> String {
        let artist_names: Vec<&str> = self.artists
            .iter()
            .map(|artist| artist.name.as_str())
            .collect();
        format!("{}: {} | {}", index, self.name, artist_names.join(" "))
    }
}

#[async_trait]
impl StartPlayback for SimplifiedAlbum {
    async fn start_playback(&self, client: Arc<AuthCodePkceSpotify>) -> Result<()> {

        let id = match &self.id {
            Some(id) => Ok(id.clone()),
            None => Err(Error::NoId(self.name.to_owned()))
        }?;

        let config = config::Config::load()?;

        client.start_context_playback(
            PlayContextId::Album(id),
            Some(&config.device_id),
            None,
            None
        ).await?;

        Ok(())
    }
}

impl ListItem for FullTrack {
    fn list_item(&self, index: usize) -> String {
        let artist_names: Vec<&str> = self.artists
            .iter()
            .map(|artist| artist.name.as_str())
            .collect();
        format!("{}: {} | {} | {}", index, self.name, self.album.name, artist_names.join(" "))
    }
}

#[async_trait]
impl StartPlayback for FullTrack {
    async fn start_playback(&self, client: Arc<AuthCodePkceSpotify>) -> Result<()> {

        let id = match &self.id {
            Some(id) => Ok(id.clone()),
            None => Err(Error::NoId(self.name.to_owned()))
        }?;

        let config = config::Config::load()?;

        client.start_uris_playback(
            [PlayableId::Track(id)].iter().map(PlayableId::as_ref),
            Some(&config.device_id),
            None,
            None
        ).await?;

        Ok(())
    }
}

impl ListItem for SimplifiedPlaylist {
    fn list_item(&self, index: usize) -> String {
        let owner_name = match &self.owner.display_name {
            Some(name) => name.clone(),
            None => String::new()
        };

        format!("{}: {} | {}", index, self.name, owner_name)
    }
}

#[async_trait]
impl StartPlayback for SimplifiedPlaylist {
    async fn start_playback(&self, client: Arc<AuthCodePkceSpotify>) -> Result<()> {

        let config = config::Config::load()?;

        client.start_context_playback(
            PlayContextId::Playlist(self.id.clone()),
            Some(&config.device_id),
            None,
            None
        ).await?;

        Ok(())
    }
}

impl ListItem for FullArtist {
    fn list_item(&self, index: usize) -> String {
        format!("{}: {}", index, self.name)
    }
}

#[async_trait]
impl StartPlayback for FullArtist {
    async fn start_playback(&self, client: Arc<AuthCodePkceSpotify>) -> Result<()> {

        let config = config::Config::load()?;

        client.start_context_playback(
            PlayContextId::Artist(self.id.clone()),
            Some(&config.device_id),
            None,
            None
        ).await?;

        Ok(())
    }
}

