use std::{collections::HashMap, sync::Mutex};

pub mod bot;
pub mod response;
pub mod parsing;
pub mod serde_response;
pub mod tests;

use rand::seq::IteratorRandom;

use crate::response::{Response, ResponseDatabase};

pub static DATA: std::sync::OnceLock<Mutex<Data>> = std::sync::OnceLock::new();

const BOT_NAMES: [&str; 2] = ["ShakeBot", "ShakeBotDev"];

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Data {
    pub response_database: ResponseDatabase,
    pub disabled_users: Vec<String>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            response_database: ResponseDatabase {
                responses: vec![],
                heroes: HashMap::new(),
                icons: HashMap::new(),
            },
            disabled_users: vec![],
        }
    }
}

impl Data {
    pub async fn update(&mut self) {
        tracing::info!("Updating database");
        self.response_database.responses.clear();
        self.response_database.heroes.clear();
        tracing::info!("Populating responses");
        self.response_database.populate_responses().await;
    }
    pub fn get_response(&self, processed_text: &str, hero_id: Option<i32>) -> Option<&Response> {
        match hero_id {
            Some(id) => self
                .response_database
                .responses
                .iter()
                .filter(|r| r.processed_text == processed_text && r.hero_id == id)
                .choose(&mut rand::thread_rng()),
            None => self
                .response_database
                .responses
                .iter()
                .filter(|r| r.processed_text == processed_text)
                .choose(&mut rand::thread_rng()),
        }
    }
}

/// Function for pre-processing the given response text.
/// It:
/// * converts all unicode characters to their nearest ASCII equivalent
/// * replaces all punctuations with spaces
/// * replaces all whitespace characters (tab, newline etc) with spaces
/// * removes trailing and leading spaces
/// * removes double spaces
/// * changes to lowercase
pub fn process_text(text: &str) -> String {
    let text = text
        .chars()
        .map(|c| match c {
            '’' => '\'',
            '“' | '”' => '"',
            '–' => '-',
            // '…' => '',
            '—' => '-',
            '‘' => '\'',
            '•' => '-',
            _ => c,
        })
        .collect::<String>()
        .replace('…', "...")
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string();

    return text;
}
