pub mod auth;
pub mod config;
pub mod menu;

use std::sync::Arc;

use rspotify::AuthCodePkceSpotify;

use menu::{Menu, MenuProgram, MenuResult};
use menu::mode::ModeMenu;

pub async fn show(client: Arc<AuthCodePkceSpotify>, program: MenuProgram) {
    let mode_menu = Box::new(ModeMenu::new(Arc::clone(&client)));
    let mut menu_stack: Vec<Box<dyn Menu>> = vec![mode_menu];

    while let Some(menu) = menu_stack.pop() {
        match menu.select(program.clone()).await {
            MenuResult::Menu(new_menu) => {
                menu_stack.push(menu);
                menu_stack.push(new_menu);
            },
            MenuResult::Back => continue,
            MenuResult::Exit => break,
            MenuResult::Input(_) => break,
        }
    }
}
