use rofify::{
    auth,
    menu::{
        mode::ModeMenu,
        Menu,
        MenuProgram, MenuResult
    }
};
use rspotify::{
    prelude::*,
    model::{PlayableItem, AdditionalType, Country, Market, CurrentlyPlayingContext, SearchType, SearchResult},
    AuthCodeSpotify,
    Credentials,
    OAuth,
    ClientError
};
use std::{process::{Command, Stdio}, io::Write, str, sync::Arc};

#[tokio::main]
async fn main() {
    let client = Arc::new(auth::get_token().await);

    let mode_menu = Box::new(ModeMenu::new(Arc::clone(&client)));
    let mut menu_stack: Vec<Box<dyn Menu>> = vec![mode_menu];

    while let Some(menu) = menu_stack.pop() {
        match menu.select(MenuProgram::Rofi).await {
            MenuResult::Menu(new_menu) => {
                menu_stack.push(menu);
                menu_stack.push(new_menu);
            },
            MenuResult::Back => continue,
            MenuResult::Exit => break
        }
    }
}
