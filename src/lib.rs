use std::{collections::HashMap, sync::Mutex};

use dota::{response::ResponseDatabase, Dota};

pub mod bot;
pub mod dota;
pub mod tests;

pub static DATA: std::sync::OnceLock<Mutex<Data>> = std::sync::OnceLock::new();

const BOT_NAMES: [&str; 2] = ["ShakeBot", "ShakeBotDev"];

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Data {
    pub dota: Dota,
    pub disabled_users: Vec<String>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            dota: Dota {
                version: "0_00a".to_string(),
                response_database: ResponseDatabase {
                    responses: vec![],
                    heroes: HashMap::new(),
                    icons: HashMap::new(),
                },
                patch_notes: HashMap::new(),
            },
            disabled_users: vec![],
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
