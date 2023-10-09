pub mod device;
pub mod mode;
pub mod playback;
pub mod search;

use serde::{Serialize, Deserialize};
use std::{
    process::{Command, Stdio},
    num::ParseIntError
};
use async_trait::async_trait;

#[async_trait]
pub trait Menu {
    fn items(&self) -> Vec<String>;

    fn prompt(&self, program: MenuProgram) -> String {
        let input_from_echo = Command::new("echo")
            .arg(self.items().join("\n"))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let selection = program.command()
            .stdin(input_from_echo.stdout.unwrap())
            .output()
            .unwrap();

        String::from_utf8(selection.stdout).unwrap().trim().to_owned()
    }

    async fn select(&self, program: MenuProgram) -> MenuResult;
}

pub enum MenuResult {
    Menu(Box<dyn Menu>),
    Back,
    Exit,
    Input(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MenuProgram {
    Rofi,
    DMenu,
}

impl MenuProgram {
    fn command(&self) -> Command {
        match self {
            MenuProgram::Rofi => {
                let mut cmd = Command::new("rofi");
                cmd.arg("-dmenu");
                cmd
            },
            MenuProgram::DMenu => Command::new("dmenu"),
        }
    }
}


pub fn selection_index(selection: &str) -> Result<usize, ParseIntError> {
    selection
        .chars()
        .take_while(|&ch| ch != ':')
        .collect::<String>()
        .parse()
}
