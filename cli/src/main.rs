use clap::{Parser, Subcommand};
use notify::enotify;
use std::{sync::Arc, process::exit};
use rofify::{auth, config::Config};

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
    let cli = Cli::parse();

    let program = match Config::load() {
        Ok(config) => config.program.unwrap(),
        Err(error) => {
            enotify(&format!("Failed to load program from config: {error}"));
            exit(1)
        },
    };

    let client = match auth::auth(program.clone()).await {
        Ok(client) => Arc::new(client),
        Err(error) => {
            enotify(&format!("Failed to authenticate with spotify: {error}"));
            exit(1);
        }
    };

    match cli.command {
        Commands::Show => rofify::show(client, program).await,
        Commands::Control{ action } => if let Err(error) = controller::control(client, &action, program).await {
            enotify(&format!("Failed to perform \"{}\": {error}", &action));
            exit(1)
        },
        Commands::Visualize => println!("visualize"),

    }
}
