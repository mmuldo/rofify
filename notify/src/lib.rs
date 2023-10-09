use notify_rust;
use std::env;

const ICON_PATH: &str = "spotify.png";

pub fn notify(summary: &str, body: &str) {
    let mut notification = notify_rust::Notification::new();
    let icon_path = env::current_dir()
        .unwrap()
        .join(ICON_PATH)
        .into_os_string()
        .into_string()
        .unwrap();

    notification.summary(summary);
    notification.body(body);
    notification.icon(&icon_path);

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
    notify("Error", body);
}
