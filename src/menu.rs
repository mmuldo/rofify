use core::fmt;
use std::{process::{Command, Stdio}, str::FromStr, num::ParseIntError};

use async_trait::async_trait;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::Device
};
use strum::{IntoEnumIterator, EnumIter};
use std::env;

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

#[async_trait]
pub trait Menu: Send + Sync {
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

    async fn select(&self, program: MenuProgram) -> Option<Box<&dyn Menu>>;
}

#[derive(Debug, EnumIter)]
pub enum Mode {
    TrackSearch,
    AlbumSearch,
    Device,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::TrackSearch => "Track Search",
            Self::AlbumSearch => "Album Search",
            Self::Device => "Device",
        };
        write!(f, "{text}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseModeError;

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Track Search" => Ok(Self::TrackSearch),
            "Album Search" => Ok(Self::AlbumSearch),
            "Device" => Ok(Self::Device),
            _ => Err(ParseModeError)
        }
    }
}

pub struct ModeMenu<'a> {
    client: &'a AuthCodePkceSpotify
}

impl<'a> ModeMenu<'a> {
    pub async fn new(client: &'a AuthCodePkceSpotify) -> ModeMenu<'a> {
        Self {
            client
        }
    }
}

#[async_trait]
impl Menu for ModeMenu<'_> {
    fn items(&self) -> Vec<String> {
        Mode::iter().map(|mode| mode.to_string()).collect()
    }

    async fn select(&self, program: MenuProgram) -> Option<Box<&dyn Menu>> {
        let selection = self.prompt(program);
        let parsed_mode = Mode::from_str(selection.as_str());

        match parsed_mode {
            Ok(mode) => match mode {
                Mode::TrackSearch => None,
                Mode::AlbumSearch => None,
                Mode::Device => {
                    let caller: Option<Box<&dyn Menu>> = Some(Box::new(self));
                    let device_menu: Box<&dyn Menu> = Box::new(
                        &DeviceMenu::new(
                            caller,
                            self.client
                        ).await
                    );
                    Some(device_menu)
                }
            }
            Err(_) => {
                println!("{selection:#?} is not a valid mode");
                None
            }
        }
    }
}

fn selection_index(selection: String) -> Result<usize, ParseIntError> {
    selection
        .chars()
        .take_while(|&ch| ch != ':')
        .collect::<String>()
        .parse()
}

pub struct DeviceMenu<'a, 'b> {
    caller: Option<Box<&'a dyn Menu>>,
    devices: Vec<Device>,
    client: &'b AuthCodePkceSpotify
}

impl<'a, 'b> DeviceMenu<'a, 'b> {
    pub async fn new(caller: Option<Box<&'a dyn Menu>>, client: &'b AuthCodePkceSpotify) -> DeviceMenu<'a, 'b> {
        let devices = client.device().await.unwrap();
        Self {
            caller,
            devices,
            client
        }
    }
}

#[async_trait]
impl Menu for DeviceMenu<'_, '_> {
    fn items(&self) -> Vec<String> {
        self.devices
            .iter()
            .enumerate()
            .map(|(i, device)| {
                format!("{}: {}", i, device.name)
            })
            .collect()
    }

    async fn select(&self, program: MenuProgram) -> Option<Box<&dyn Menu>> {
        let selection = self.prompt(program);
        let parsed_index = selection_index(selection);

        match parsed_index {
            Ok(index) => {
                let device = &self.devices[index];
                let device_id = device.id.clone().unwrap();
                env::set_var("DEVICE_ID", device_id.as_str());
                println!("{}", env::var("DEVICE_ID").unwrap());
                None
            }
            Err(_) => {
                println!("failed to get index of selected item");
                match &self.caller {
                    Some(caller) => {
                        let caller_copy: Option<Box<&dyn Menu>> = Some(Box::new(**caller));
                        caller_copy
                    },
                    None => None
                }
            }
        }
    }
}
