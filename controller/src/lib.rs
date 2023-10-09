use notify::notify;
use rofify::menu::MenuProgram;
use rofify::menu::device::device_id;
use rspotify::model::{AdditionalType, PlayableItem};
use rspotify::{AuthCodePkceSpotify, ClientError};
use rspotify::prelude::OAuthClient;
use std::{result, fmt};
use std::sync::Arc;
use clap::Subcommand;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error from spotify client: {0}")]
    Client(#[from] ClientError),
    #[error("Nothing is playing right now.")]
    NoContext,
    #[error("Item is not a playable track.")]
    NotTrack,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Subcommand)]
pub enum Action {
    PlayPause,
    Next,
    Previous,
    Like,
    Shuffle,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::PlayPause => "play-pause",
            Self::Next => "next",
            Self::Previous => "previous",
            Self::Like => "like",
            Self::Shuffle => "shuffle",
        };
        write!(f, "{text}")
    }
}

async fn play_pause(client: Arc<AuthCodePkceSpotify>,  device_id: Option<&str>) -> Result<()> {
    let maybe_current_playback_context = client.current_playback(
        None,
        Some([
            &AdditionalType::Track,
            &AdditionalType::Episode
        ])
    ).await?;

    if let Some(current_playback_context) = maybe_current_playback_context {
        if current_playback_context.is_playing {
            client.pause_playback(device_id).await?;
            return Ok(());
        } 
    }

    client.resume_playback(device_id, None).await?;
    Ok(())
}

async fn like(client: Arc<AuthCodePkceSpotify>) -> Result<()> {
    let maybe_currently_playing_context = client.current_playing(
        None,
        Some([
            &AdditionalType::Track,
        ])
    ).await?; 

    Ok(match maybe_currently_playing_context {
        Some(currently_playing_context) => {
            match currently_playing_context.item {
                Some(PlayableItem::Track(track)) => {
                    let artist_names: Vec<&str> = track.artists
                        .iter()
                        .map(|artist| artist.name.as_str())
                        .collect();
                    let formatted_track = format!("{} | {} | {}", track.name, track.album.name, artist_names.join(", "));

                    if !client.current_user_saved_tracks_contains([track.id.clone().unwrap()]).await?[0] {
                        client.current_user_saved_tracks_add([track.id.clone().unwrap()]).await?;

                        notify("Added to liked songs:", &formatted_track);
                        Ok(())
                    } else {
                        notify("Already in liked songs:", &formatted_track);
                        Ok(())
                    }
                },
                _ => Err(Error::NotTrack)
            }

        },
        None => Err(Error::NoContext)
    }?)
}

async fn shuffle(client: Arc<AuthCodePkceSpotify>, device_id: Option<&str>) -> Result<()> {
    let maybe_current_playback_context = client.current_playback(
        None,
        Some([
            &AdditionalType::Track,
            &AdditionalType::Episode
        ])
    ).await?;

    Ok(match maybe_current_playback_context {
        Some(current_playback_context) => {
            let is_shuffled = current_playback_context.shuffle_state;
            client.shuffle(!is_shuffled, device_id).await?;
            notify("Shuffle", if is_shuffled { "disabled" } else { "enabled" });
            Ok(())
        },
        None => Err(Error::NoContext)
    }?)
}

pub async fn control(client: Arc<AuthCodePkceSpotify>, action: &Action, program: MenuProgram) -> Result<()> {
    match action {
        Action::PlayPause => {
            play_pause(Arc::clone(&client), device_id(Arc::clone(&client), program).await.as_deref()).await?;
        },
        Action::Next => {
            client.next_track(device_id(Arc::clone(&client), program).await.as_deref()).await?;
        },
        Action::Previous => {
            client.previous_track(device_id(Arc::clone(&client), program).await.as_deref()).await?;
        },
        Action::Like => {
            like(client).await?;
        },
        Action::Shuffle => {
            shuffle(Arc::clone(&client), device_id(Arc::clone(&client), program).await.as_deref()).await?;
        }
    };

    Ok(())
}
