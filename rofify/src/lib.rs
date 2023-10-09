pub mod auth;
pub mod config;
pub mod menu;

use std::{
    env,
    sync::Arc
};

use rspotify::AuthCodePkceSpotify;

use menu::{Menu, MenuProgram, MenuResult};
use menu::mode::ModeMenu;

pub const ICON_PATH: &str = "spotify.png";

pub async fn show(client: Arc<AuthCodePkceSpotify>) {
    let icon_path = env::current_dir().unwrap().join(ICON_PATH).into_os_string().into_string().unwrap();

    let mode_menu = Box::new(ModeMenu::new(Arc::clone(&client)));
    let mut menu_stack: Vec<Box<dyn Menu>> = vec![mode_menu];

    while let Some(menu) = menu_stack.pop() {
        match menu.select(MenuProgram::Rofi).await {
            MenuResult::Menu(new_menu) => {
                menu_stack.push(menu);
                menu_stack.push(new_menu);
            },
            MenuResult::Back(maybe_notification) => {
                if let Some(mut notification) = maybe_notification {
                    notification.icon(&icon_path);
                    match notification.show() {
                        Ok(_) => (),
                        Err(error) => eprintln!("Failed to send notification: {error}"),
                    }
                };
                continue
            },
            MenuResult::Exit(maybe_notification) => {
                if let Some(mut notification) = maybe_notification {
                    notification.icon(&icon_path);
                    match notification.show() {
                        Ok(_) => (),
                        Err(error) => eprintln!("Failed to send notification: {error}"),
                    }
                };
                break
            },
            _ => break,
        }
    }
}
