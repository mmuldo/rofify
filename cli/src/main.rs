use clap::{Parser, Subcommand};
use notify_rust::Notification;
use std::{sync::Arc, process::exit, env};
use rofify::{auth, ICON_PATH};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Show,
    Control {
    #[command(subcommand)]
        action: controller::Action,
    },
    Visualize
}

#[tokio::main]
async fn main() {
    let mut exit_code = 0;
    let icon_path = env::current_dir().unwrap().join(ICON_PATH).into_os_string().into_string().unwrap();
    let mut notification = Notification::new();
    notification.icon(&icon_path);
    let cli = Cli::parse();

    match auth::auth().await {
        Ok(client) => {
            let client = Arc::new(client);
            match cli.command {
                Commands::Show => rofify::show(client).await,
                Commands::Control{ action } => match controller::control(client, action).await {
                    Ok(_) => (),
                    Err(error) => {
                        notification.summary("Error");
                        notification.body(&format!("Failed to perform action: {error}"));
                        match notification.show() {
                            Ok(_) => (),
                            Err(error) => eprintln!("Failed to send notification: {error}"),
                        }
                    }
                },
                Commands::Visualize => println!("visualize"),
            }
        },
        Err(error) => {
            match error {
                auth::Error::Notification(error) => eprintln!("Failed to send notification: {error}"),
                _ => {
                    notification.summary("Error");
                    notification.body(&format!("Failed to authenticate with spotify: {error}"));
                    match notification.show() {
                        Ok(_) => (),
                        Err(error) => eprintln!("Failed to send notification: {error}"),
                    }
                }
            };
            exit_code = 1;
        }
    }

    exit(exit_code);

}
