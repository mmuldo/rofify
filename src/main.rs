use rofify::{
    auth,
    menu::{
        mode::ModeMenu,
        Menu,
        MenuProgram, MenuResult
    }
};
use std::{
    env,
    sync::Arc
};

static ICON_PATH: &str = "spotify.png";

#[tokio::main]
async fn main() {
    let icon_path = env::current_dir().unwrap().join(ICON_PATH).into_os_string().into_string().unwrap();
    let client = Arc::new(auth::get_token().await);

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
                    notification.show().unwrap();
                };
                continue
            },
            MenuResult::Exit(maybe_notification) => {
                if let Some(mut notification) = maybe_notification {
                    notification.icon(&icon_path);
                    notification.show().unwrap();
                };
                break
            }
        }
    }
}
