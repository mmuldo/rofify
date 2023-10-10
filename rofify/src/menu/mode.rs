use core::fmt;
use std::{
    str::FromStr,
    sync::Arc
};
use futures::{
    stream::TryStreamExt,
    StreamExt
};
use async_trait::async_trait;
use notify::enotify;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::{
        SearchType,
        FullTrack
    }
};
use strum::{
    IntoEnumIterator,
    EnumIter
};
use super::{
    Menu,
    MenuProgram,
    MenuResult,
    device::DeviceMenu,
    search::SearchMenu,
    playback::PlaybackMenu
};

const LIKED_SONGS_LIMIT: usize = 100;

#[derive(Debug, EnumIter)]
pub enum Mode {
    ArtistSearch,
    AlbumSearch,
    TrackSearch,
    PlaylistSearch,
    MyPlaylists,
    LikedSongs,
    Device,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::ArtistSearch => "Artist Search",
            Self::AlbumSearch => "Album Search",
            Self::TrackSearch => "Track Search",
            Self::PlaylistSearch => "Playlist Search",
            Self::MyPlaylists => "My Playlists",
            Self::LikedSongs => "Liked Songs",
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
            "Artist Search" => Ok(Self::ArtistSearch),
            "Album Search" => Ok(Self::AlbumSearch),
            "Track Search" => Ok(Self::TrackSearch),
            "Playlist Search" => Ok(Self::PlaylistSearch),
            "My Playlists" => Ok(Self::MyPlaylists),
            "Liked Songs" => Ok(Self::LikedSongs),
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
        let selection = self.prompt(program, "Rofify");
        let parsed_mode = Mode::from_str(selection.as_str());

        match parsed_mode {
            Ok(mode) => match mode {
                Mode::ArtistSearch => MenuResult::Menu(Box::new(
                    SearchMenu::new(Arc::clone(&self.client), SearchType::Artist).await
                )),
                Mode::AlbumSearch => MenuResult::Menu(Box::new(
                    SearchMenu::new(Arc::clone(&self.client), SearchType::Album).await
                )),
                Mode::TrackSearch => MenuResult::Menu(Box::new(
                    SearchMenu::new(Arc::clone(&self.client), SearchType::Track).await
                )),
                Mode::PlaylistSearch => MenuResult::Menu(Box::new(
                    SearchMenu::new(Arc::clone(&self.client), SearchType::Playlist).await
                )),
                Mode::MyPlaylists => {
                    match self.client
                        .current_user_playlists()
                        .try_collect::<Vec<_>>()
                        .await {
                            Ok(playlists) => MenuResult::Menu(Box::new(
                                PlaybackMenu::new(Arc::clone(&self.client), playlists).await
                            )),
                            Err(error) => {
                                enotify(&format!("Failed to get playlists: {error}"));
                                MenuResult::Back
                            }
                        }
                },
                Mode::LikedSongs => {
                    let try_liked_songs = self.client
                        .current_user_saved_tracks(None)
                        .take(LIKED_SONGS_LIMIT)
                        .try_filter_map(|saved_track| async {
                            Ok(Some(saved_track.track))
                        })
                        .try_collect::<Vec<FullTrack>>()
                        .await;

                    match try_liked_songs {
                        Ok(liked_songs) => MenuResult::Menu(Box::new(
                            PlaybackMenu::new(Arc::clone(&self.client), liked_songs).await
                        )),
                        Err(error) => {
                            enotify(&format!("Failed to get liked songs: {error}"));
                            MenuResult::Back
                        }
                    }
                }
                Mode::Device => MenuResult::Menu(Box::new(
                    DeviceMenu::new(Arc::clone(&self.client)).await
                )),
            }
            Err(_) => {
                if !selection.is_empty() {
                    enotify(&format!("{selection:#?} is not a valid mode"));
                }
                MenuResult::Back
            }
        }
    }
}
