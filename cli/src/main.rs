use clap::{Parser, Subcommand};
use notify::enotify;
use std::{sync::Arc, process::exit};
use rofify::{auth, menu::MenuProgram};

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

const PROGRAM: MenuProgram = MenuProgram::Rofi;

#[tokio::main]
async fn main() {
    let mut exit_code = 0;
    let cli = Cli::parse();

    match auth::auth(PROGRAM).await {
        Ok(client) => {
            let client = Arc::new(client);
            match cli.command {
                Commands::Show => rofify::show(client, PROGRAM).await,
                Commands::Control{ action } => if let Err(error) = controller::control(client, &action, PROGRAM).await {
                    enotify(&format!("Failed to perform {}: {error}", &action));
                    exit_code = 1;
                },
                Commands::Visualize => println!("visualize"),
            }
        },
        Err(error) => {
            enotify(&format!("Failed to authenticate with spotify: {error}"));
            exit_code = 1;
        }
    }

    exit(exit_code);

}
