mod auth;
mod menu;

use menu::{ModeMenu, Menu, MenuProgram};
use rspotify::{
    prelude::*,
    model::{PlayableItem, AdditionalType, Country, Market, CurrentlyPlayingContext, SearchType, SearchResult},
    AuthCodeSpotify,
    Credentials,
    OAuth,
    ClientError
};
use std::{process::{Command, Stdio}, io::Write, str};

#[tokio::main]
async fn main() {
    let client = auth::get_token().await;

    let mode_menu = ModeMenu::new(&client).await;
    let mut maybe_menu: Option<Box<&dyn Menu>> = Some(Box::new(&mode_menu));

    while let Some(menu) = maybe_menu {
        maybe_menu = menu.select(MenuProgram::Rofi).await;
    }
}
