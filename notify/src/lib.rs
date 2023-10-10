use notify_rust;
use std::{env, path::{PathBuf, Path}};

const ICONS_DIR: &str = ".local/share/rofify/icons";
const APP_ICON: &str = "rofify.png";
const COVER_ART_ICON: &str = "cover.png";

pub fn notify(summary: &str, body: &str, icon: Option<String>) {
    let mut notification = notify_rust::Notification::new();
    let icon = if let Some(icon) = icon {
        icon
    } else {
        match app_icon_path().into_os_string().into_string() {
            Ok(icon) => icon,
            // any garbage string (including "") works since it will result in
            // the notification just not showing any icon
            Err(_) => String::new()
        }
    };

    notification.summary(summary);
    notification.body(body);
    notification.icon(&icon);

    match notification.show() {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to send notification: {error}");
            eprintln!("The original notification was:");
            eprintln!("\t{summary}");
            eprintln!("\t{body}");
        },
    }
}

pub fn enotify(body: &str) {
    notify("Error", body, None);
}


pub fn icons_dir() -> PathBuf {
    let path = Path::new(env!("HOME"));
    let icons_dir = path.join(ICONS_DIR);
    icons_dir
}

pub fn app_icon_path() -> PathBuf {
    icons_dir().join(APP_ICON)
}

pub fn cover_art_icon_path() -> PathBuf {
    icons_dir().join(COVER_ART_ICON)
}
