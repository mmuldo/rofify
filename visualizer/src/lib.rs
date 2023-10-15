mod spectrum;
use std::{sync::Arc, thread, result, time};
use chrono;

use eframe::{egui::{self, Vec2, FontDefinitions}, run_native, CreationContext, NativeOptions, App, Frame, emath::Numeric, epaint::{Color32, FontFamily, FontId}, Storage};
use rspotify::{AuthCodePkceSpotify, prelude::OAuthClient, model::{AdditionalType, PlayableItem}, ClientError};
use spectrum::Bode;
use tokio::sync::mpsc::{channel, Sender, Receiver};

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
    progress: chrono::Duration,
    duration: chrono::Duration,
    track: String,
    album: String,
    artists: Vec<String>,
    cover_art_url: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            liked: Default::default(),
            shuffled: Default::default(),
            progress: chrono::Duration::seconds(0),
            duration: chrono::Duration::seconds(0),
            track: Default::default(),
            album: Default::default(),
            artists: Default::default(),
            cover_art_url: Default::default(),
        }
    }
}

struct Client {
    client: Arc<AuthCodePkceSpotify>,
    tx: Sender<StateResult<State>>
}

impl Client {
    fn new(client: Arc<AuthCodePkceSpotify>, tx: Sender<StateResult<State>>) -> Self {
        Self {
            client,
            tx
        }
    }

    async fn get_state(&self) -> StateResult<State >{
        if let Some(current_playback_context) = self.client.current_playback(None, Some([
            &AdditionalType::Track,
            &AdditionalType::Episode
        ])).await? {
            if let (Some(progress), Some(PlayableItem::Track(track))) = (current_playback_context.progress, current_playback_context.item) {
                let duration = track.duration;
                let shuffled = current_playback_context.shuffle_state;
                let liked = self.client
                    .current_user_saved_tracks_contains([track.id.clone().unwrap()])
                    .await?
                    .first()
                    .unwrap()
                    .clone();
                let track_name = track.name.clone();
                let album = track.album.name.clone();
                let artists: Vec<String> = track.artists
                    .iter()
                    .map(|artist| artist.name.clone())
                    .collect();
                let cover_art_url = track.album.images.first().unwrap().url.clone();

                Ok(State {
                    liked,
                    shuffled,
                    progress,
                    duration,
                    track: track_name,
                    album,
                    artists,
                    cover_art_url
                })
            } else {
                Err(StateError::MissingState)
            }
        } else {
            Err(StateError::NoContext)
        }
    }

    fn spawn(self) {
        tokio::spawn(async move {
            while let Ok(()) = self.tx.send(self.get_state().await).await {
                tokio::time::sleep(time::Duration::from_millis(50)).await;
            }
        });
    }
}

struct Visualizer {
    state: State,
    bode: Bode,
    rx: Receiver<StateResult<State>>
}

impl Visualizer {
    fn new(rx: Receiver<StateResult<State>>) -> Self {
        Self {
            state: State::default(),
            bode: Bode::new(),
            rx
        }
    }
}

impl App for Visualizer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(Ok(state)) = self.rx.try_recv() {
                self.state = state;
            }

            ui.columns(3, |columns| {
                let info_layout = egui::Layout::top_down(eframe::emath::Align::Center);
                columns[0].with_layout(info_layout, |ui| {
                    let cover_art_height = ui.max_rect().max.x / 3.;
                    let space = cover_art_height / 3.;
                    let font_size = cover_art_height / 3.;

                    ui.add_space(space);
                    let track = egui::RichText::new(format!("{}", self.state.track)).size(font_size);
                    ui.label(track);

                    ui.add_space(space);
                    let album = egui::RichText::new(format!("{}", self.state.album)).size(0.7 * font_size);
                    ui.label(album);

                    ui.add_space(space);
                    let artists = egui::RichText::new(format!("{}", self.state.artists.join(", "))).size(0.8 * font_size);
                    ui.label(artists);
                });

                let image_layout = egui::Layout::top_down(eframe::emath::Align::Center);
                columns[1].with_layout(image_layout, |ui| {
                    ui.add(egui::Image::new(self.state.cover_art_url.clone()))
                });

                let icons_layout = egui::Layout::top_down(eframe::emath::Align::Center);
                columns[2].with_layout(icons_layout, |ui| {
                    let cover_art_height = ui.max_rect().max.x / 3.;
                    let space = cover_art_height / 3.;
                    let font_size = cover_art_height / 3.;
                    let active_color = Color32::from_rgb(196, 39, 39);
                    let inactive_color = Color32::from_rgb(156, 116, 116);

                    ui.add_space(space);
                    let liked = egui::RichText::new("")
                        .font(FontId::new(font_size, FontFamily::Proportional))
                        .color(if self.state.liked {active_color} else {inactive_color});
                    ui.label(liked);

                    ui.add_space(space);
                    let shuffled = egui::RichText::new("")
                        .font(FontId::new(font_size, FontFamily::Proportional))
                        .color(if self.state.shuffled {active_color} else {inactive_color});
                    ui.label(shuffled);
                })
            });

            let progress_bar = egui::ProgressBar::new(
                self.state.progress.num_milliseconds() as f32 / self.state.duration.num_milliseconds() as f32
            )
                .text(format!("{} / {}", format_duration(self.state.progress), format_duration(self.state.duration)))
                .desired_height(10.)
                .fill(Color32::from_rgb(122, 36, 39));
            ui.add(progress_bar);

            self.bode.show(ui);

        });

        ctx.request_repaint();
    }
}

fn format_two_digit_int(number: i64) -> String {
    let tens = number.div_euclid(10);
    let ones = number % 10;

    format!("{}{}", tens.to_string(), ones.to_string())
}

fn format_duration(duration: chrono::Duration) -> String {
    let minutes = duration.num_minutes();
    let seconds = duration.num_seconds() % 60;
    format!("{}:{}",
    format_two_digit_int(minutes),
    format_two_digit_int(seconds)
)
}

pub fn show(client: Arc<AuthCodePkceSpotify>) -> eframe::Result<()> {
    let (tx, rx) = channel(1);
    let client = Client::new(client, tx);
    let visualizer = Visualizer::new(rx);

    client.spawn();

    let mut native_options = NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(540., 540.));
    native_options.max_window_size = Some(Vec2::new(540., 540.));

    run_native(
        "Rofify Visualizer",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "awesome".to_owned(),
                egui::FontData::from_static(include_bytes!("font-awesome-solid.ttf"))
            );
            fonts.families.get_mut(&FontFamily::Proportional)
                .unwrap()
                .push("awesome".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Box::new(visualizer)
        })
    )
}
