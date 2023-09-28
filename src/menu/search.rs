use core::fmt;
use std::{process::{Command, Stdio}, str::FromStr, num::ParseIntError, sync::Arc};

use async_trait::async_trait;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::{Device, SearchType, SearchResult},
};
use strum::{IntoEnumIterator, EnumIter};
use std::env;

use crate::config::Config;

use super::{Menu, MenuProgram, MenuResult, selection_index, album::AlbumMenu, track::TrackMenu};

pub struct SearchMenu {
    client: Arc<AuthCodePkceSpotify>,
    search_type: SearchType,
}

impl SearchMenu {
    pub async fn new(client: Arc<AuthCodePkceSpotify>, search_type: SearchType) -> SearchMenu {
        Self {
            client,
            search_type
        }
    }
}

#[async_trait]
impl Menu for SearchMenu {
    fn items(&self) -> Vec<String> {
        Vec::new()
    }

    async fn select(&self, program: MenuProgram) -> MenuResult {
        let query = self.prompt(program);
        let result = self.client.search(
            &query,
            self.search_type,
            None,
            None,
            Some(10),
            Some(0),
        ).await;

        match result {
            Ok(result) => match result {
                SearchResult::Albums(page) => MenuResult::Menu(Box::new(
                    AlbumMenu::new(Arc::clone(&self.client), page.items).await
                )),
                SearchResult::Tracks(page) => MenuResult::Menu(Box::new(
                    TrackMenu::new(Arc::clone(&self.client), page.items).await
                )),
                _ => MenuResult::Exit
            }
            Err(_) => {
                MenuResult::Back
            }
        }
    }
}
