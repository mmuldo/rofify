use notify::{notify, cover_art_icon_path, icons_dir};
use rofify::menu::MenuProgram;
use rofify::menu::device::device_id;
use rspotify::model::{AdditionalType, PlayableItem, CurrentPlaybackContext, RepeatState};
use rspotify::{AuthCodePkceSpotify, ClientError};
use rspotify::prelude::OAuthClient;
use std::future::Future;
use std::{result, fmt, fs, io};
use std::sync::Arc;
use clap::Subcommand;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    #[error("Error from spotify client: {0}")]
    Client(#[from] ClientError),
    #[error("Http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Failed to convert os path to string.")]
    PathToString,
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
    Repeat,
    OnChange,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::PlayPause => "play-pause",
            Self::Next => "next",
            Self::Previous => "previous",
            Self::Like => "like",
            Self::Shuffle => "shuffle",
            Self::Repeat => "repeat",
            Self::OnChange => "on-change",
        };
        write!(f, "{text}")
    }
}

struct Controller {
    client: Arc<AuthCodePkceSpotify>,
    device_id: Option<String>,
}

impl Controller {
    async fn new(client: Arc<AuthCodePkceSpotify>, program: MenuProgram) -> Self {
        let device_id = device_id(
            Arc::clone(&client),
            program.clone()
        ).await;

        Self { client, device_id }
    }

    async fn control<F, Fut>(&self, with_context: F) -> Result<()>
    where
        F: FnOnce(Arc<AuthCodePkceSpotify>, CurrentPlaybackContext, Option<String>) -> Fut,
        Fut: Future<Output = Result<()>>
    {
        let maybe_current_playback_context = self.client.current_playback(
            None,
            Some([
                &AdditionalType::Track,
                &AdditionalType::Episode
            ])
        ).await?;

        Ok(match maybe_current_playback_context {
            Some(current_playback_context) => {
                with_context(
                    Arc::clone(&self.client),
                    current_playback_context,
                    self.device_id.clone()
                ).await?;
                Ok(())
            },
            None => Err(Error::NoContext)
        }?)
    }

    async fn next(&self) -> Result<()> {
        self.client.next_track(self.device_id.as_deref()).await?;
        Ok(())
    }

    async fn previous(&self) -> Result<()> {
        self.client.previous_track(self.device_id.as_deref()).await?;
        Ok(())
    }

    async fn play_pause(&self) -> Result<()> {
        self.control(|client, context, device_id| async move {
            play_pause(client, context, device_id).await
        }).await
    }

    async fn shuffle(&self) -> Result<()> {
        self.control(|client, context, device_id| async move {
            shuffle(client, context, device_id).await
        }).await
    }

    async fn repeat(&self) -> Result<()> {
        self.control(|client, context, device_id| async move {
            repeat(client, context, device_id).await
        }).await
    }


    async fn like(&self) -> Result<()> {
        self.control(|client, context, _| async move {
            like(client, context).await
        }).await
    }

    async fn on_change(&self) -> Result<()> {
        self.control(|_, context, _| async move {
            on_change(context).await
        }).await
    }
}

async fn play_pause(
    client: Arc<AuthCodePkceSpotify>,
    context: CurrentPlaybackContext,
    device_id: Option<String>
) -> Result<()> {
    if context.is_playing {
        client.pause_playback(device_id.as_deref()).await?;
    } else {
        client.resume_playback(device_id.as_deref(), None).await?;
    }

    Ok(())
}

async fn shuffle(
    client: Arc<AuthCodePkceSpotify>,
    context: CurrentPlaybackContext,
    device_id: Option<String>
) -> Result<()> {
    let is_shuffled = context.shuffle_state;
    client.shuffle(!is_shuffled, device_id.as_deref()).await?;
    notify("Shuffle", if is_shuffled { "disabled" } else { "enabled" }, None);
    Ok(())
}

async fn repeat(
    client: Arc<AuthCodePkceSpotify>,
    context: CurrentPlaybackContext,
    device_id: Option<String>
) -> Result<()> {
    let repeat_state = context.repeat_state;
    let (new_repeat_state, name) = match repeat_state {
        RepeatState::Off => (RepeatState::Context, "context"),
        RepeatState::Context => (RepeatState::Track, "track"),
        RepeatState::Track => (RepeatState::Off, "off"),
    };
    client.repeat(new_repeat_state.clone(), device_id.as_deref()).await?;
    notify("Repeat", name, None);
    Ok(())
}

async fn like(
    client: Arc<AuthCodePkceSpotify>,
    context: CurrentPlaybackContext,
) -> Result<()> {
    match context.item {
        Some(PlayableItem::Track(track)) => {
            let artist_names: Vec<&str> = track.artists
                .iter()
                .map(|artist| artist.name.as_str())
                .collect();
            let formatted_track = format!("{} | {} | {}", track.name, track.album.name, artist_names.join(", "));

            if !client.current_user_saved_tracks_contains([track.id.clone().unwrap()]).await?[0] {
                client.current_user_saved_tracks_add([track.id.clone().unwrap()]).await?;

                notify("Added to liked songs:", &formatted_track, None);
                Ok(())
            } else {
                notify("Already in liked songs:", &formatted_track, None);
                Ok(())
            }
        },
        _ => Err(Error::NotTrack)
    }

}

async fn on_change(
    context: CurrentPlaybackContext,
) -> Result<()> {
    match context.item {
        Some(PlayableItem::Track(track)) => {
            let artist_names: Vec<&str> = track.artists
                .iter()
                .map(|artist| artist.name.as_str())
                .collect();

            let cover_art = track.album.images.last();

            let raw_image = reqwest::get(&cover_art.unwrap().url)
                .await?
                .bytes()
            .await?;
            let cover_art_icon = image::load_from_memory(&raw_image)?;
            fs::create_dir_all(icons_dir())?;
            cover_art_icon.save(cover_art_icon_path())?;

            match cover_art_icon_path().into_os_string().into_string() {
                Ok(icon_path) => Ok(notify(
                    &track.name,
                    &format!("{} - {}", artist_names.join(", "), track.album.name),
                    Some(icon_path)
                )),
                Err(_) => Err(Error::PathToString)
            }
        },
        _ => Err(Error::NotTrack)
    }
}

pub async fn control(client: Arc<AuthCodePkceSpotify>, action: &Action, program: MenuProgram) -> Result<()> {
    let controller = Controller::new(Arc::clone(&client), program).await;
    match action {
        Action::PlayPause => {
            match controller.play_pause().await {
                Err(Error::NoContext) => controller
                    .client
                    .resume_playback(controller.device_id.as_deref(), None)
                    .await?,
                otherwise => otherwise?
            };
        },
        Action::Next => {
            controller.next().await?;
        },
        Action::Previous => {
            controller.previous().await?;
        },
        Action::Like => {
            controller.like().await?;
        },
        Action::Shuffle => {
            controller.shuffle().await?;
        },
        Action::Repeat => {
            controller.repeat().await?;
        },
        Action::OnChange => {
            controller.on_change().await?;
        }
    };

    Ok(())
}
