use rofify::config::Config;
use rspotify::model::{AdditionalType, PlayableItem};
use rspotify::{AuthCodePkceSpotify, ClientError};
use rspotify::prelude::OAuthClient;
use std::result;
use std::sync::Arc;
use clap::Subcommand;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error from spotify client: {0}")]
    Client(#[from] ClientError),
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Subcommand)]
pub enum Action {
    PlayPause,
    Next,
    Previous,
    ToggleLike,
}

async fn play_pause(client: Arc<AuthCodePkceSpotify>) -> Result<()> {
    let config = Config::load().unwrap();
    let maybe_playback = client.current_playback(
        None,
        Some([
            &AdditionalType::Track,
            &AdditionalType::Episode
        ])
    ).await?;

    if let Some(playback) = maybe_playback {
        if playback.is_playing {
            client.pause_playback(None).await?;
            return Ok(());
        }
    };

    client.resume_playback(Some(&config.device_id), None).await?;

    Ok(())
}

async fn like(client: Arc<AuthCodePkceSpotify>) -> Result<()> {
    if let PlayableItem::Track(track) = client.current_playing(
        None,
        Some([
            &AdditionalType::Track,
        ])
    ).await?
        .unwrap()
        .item
        .unwrap() {
        
        if !client.current_user_saved_tracks_contains([track.id.clone().unwrap()]).await?[0] {
            client.current_user_saved_tracks_add([track.id.clone().unwrap()]).await?
        };

    }
    Ok(())


}

pub async fn control(client: Arc<AuthCodePkceSpotify>, action: Action) -> Result<()> {
    match action {
        Action::PlayPause => {
            play_pause(client).await?;
        },
        Action::Next => {
            client.next_track(None).await?;
        },
        Action::Previous => {
            client.previous_track(None).await?;
        },
        Action::ToggleLike => {
            like(client).await?;
        }
    };

    Ok(())
}
