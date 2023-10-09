use std::sync::Arc;

use async_trait::async_trait;
use notify::enotify;
use rspotify::{
    prelude::*,
    AuthCodePkceSpotify,
    model::{
        SearchType,
        SearchResult
    },
};

use super::{
    Menu,
    MenuProgram,
    MenuResult,
    playback::PlaybackMenu
};

const SEARCH_LIMIT: u32 = 25;

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

        if query.is_empty() {
            // user hit Esc or something
            return MenuResult::Back
        }

        let result = self.client.search(
            &query,
            self.search_type,
            None,
            None,
            Some(SEARCH_LIMIT),
            None,
        ).await;

        match result {
            Ok(result) => match result {
                SearchResult::Artists(page) => MenuResult::Menu(Box::new(
                    PlaybackMenu::new(Arc::clone(&self.client), page.items).await
                )),
                SearchResult::Albums(page) => MenuResult::Menu(Box::new(
                    PlaybackMenu::new(Arc::clone(&self.client), page.items).await
                )),
                SearchResult::Tracks(page) => MenuResult::Menu(Box::new(
                    PlaybackMenu::new(Arc::clone(&self.client), page.items).await
                )),
                SearchResult::Playlists(page) => MenuResult::Menu(Box::new(
                    PlaybackMenu::new(Arc::clone(&self.client), page.items).await
                )),
                _ => MenuResult::Exit
            }
            Err(error) => {
                enotify(&format!("Failed to get results for search {query:#?}: {error}"));
                MenuResult::Back
            }
        }
    }
}
