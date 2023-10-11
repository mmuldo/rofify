use std::{sync::Arc, thread, result};
use chrono::Duration;

use eframe::{egui::{self, Vec2}, run_native, CreationContext, NativeOptions, App, Frame, emath::Numeric, epaint::Color32};
use rspotify::{AuthCodePkceSpotify, prelude::OAuthClient, model::{AdditionalType, PlayableItem}, ClientError};

#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("Error from spotify client: {0}")]
    Client(#[from] ClientError),
    #[error("Nothing is playing at the moment.")]
    NoContext,
    #[error("Could not get some of the required state from the client.")]
    MissingState,
}

pub type StateResult<T> = result::Result<T, StateError>;

struct State {
    liked: bool,
    shuffled: bool,
    progress: Duration,
    duration: Duration,
    cover_art_url: String,
}

impl State {
    fn get(client: Arc<AuthCodePkceSpotify>) -> StateResult<Self> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                if let Some(current_playback_context) = client.current_playback(None, Some([
                    &AdditionalType::Track,
                    &AdditionalType::Episode
                ])).await? {
                    if let (Some(progress), Some(PlayableItem::Track(track))) = (current_playback_context.progress, current_playback_context.item) {
                        let duration = track.duration;
                        let shuffled = current_playback_context.shuffle_state;
                        let liked = client
                            .current_user_saved_tracks_contains([track.id.clone().unwrap()])
                            .await?
                            .first()
                            .unwrap()
                            .clone();
                        let cover_art_url = track.album.images.first().unwrap().url.clone();

                        Ok(Self {
                            liked,
                            shuffled,
                            progress,
                            duration,
                            cover_art_url
                        })
                    } else {
                        return Err(StateError::MissingState);
                    }

                } else {
                    Err(StateError::NoContext)
                }
            })
    }
}

fn format_two_digit_int(number: i64) -> String {
    let tens = number.div_euclid(10);
    let ones = number % 10;

    format!("{}{}", tens.to_string(), ones.to_string())
}

fn format_duration(duration: Duration) -> String {
    let minutes = duration.num_minutes();
    let seconds = duration.num_seconds() % 60;
    format!("{}:{}",
    format_two_digit_int(minutes),
    format_two_digit_int(seconds)
)
}

struct Visualizer {
    client: Arc<AuthCodePkceSpotify>
}

impl Visualizer {
    fn new(cc: &CreationContext<'_>, client: Arc<AuthCodePkceSpotify>) -> Self {
        Self { client }
    }
}

impl App for Visualizer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        let client = Arc::clone(&self.client);
        let try_state = thread::spawn(|| {
            State::get(client)
        }).join().unwrap();

        if let Ok(state) = try_state {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(format!("liked: {}", state.liked));
                        ui.heading(format!("shuffled: {}", state.shuffled));
                    });
                    ui.add(
                        egui::Image::new(state.cover_art_url)
                            .shrink_to_fit()
                    );
                });

                let progress_bar = egui::ProgressBar::new(
                    state.progress.num_milliseconds() as f32 / state.duration.num_milliseconds() as f32
                )
                    .text(format!("{}/{}", format_duration(state.progress), format_duration(state.duration)))
                    .desired_height(10.)
                    .fill(Color32::from_rgb(122, 36, 39));
                ui.add(progress_bar);

            });
            ctx.request_repaint_after(std::time::Duration::from_millis(1));
        }

    }
}

pub fn show(client: Arc<AuthCodePkceSpotify>) -> eframe::Result<()> {
    let mut native_options = NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(540., 540.));
    run_native(
        "Rofify Visualizer",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(
                Visualizer::new(cc, client)
            )
        })
    )
}
