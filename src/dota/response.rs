/// This module contains relevant methods for the responses database
/// Original Author: Jonarzz
/// Maintainer at time of access: MePsyDuck
/// Transcribed to Rust by: shakesbeare
use anyhow::Result;
use futures::{future::join_all, FutureExt};
use rand::seq::IteratorRandom;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use super::parsing::parse_all_response_lines;
use crate::dota::serde_response::*;
use crate::process_text;

const URL_BASE: &str = "http://dota2.gamepedia.com";
const API_PATH: &str = "http://dota2.gamepedia.com/api.php";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Hero {
    id: i32,
    hero_name: String,
    img_path: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Response {
    pub id: i32,
    pub processed_text: String,
    pub original_text: String,
    pub response_link: String,
    pub hero_id: i32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ResponseDatabase {
    pub responses: Vec<Response>,
    pub heroes: HashMap<i32, Hero>,
    pub icons: HashMap<String, String>,
}

static HERO_ID: OnceLock<Mutex<i32>> = OnceLock::new();
static RESPONSE_ID: OnceLock<Mutex<i32>> = OnceLock::new();

impl ResponseDatabase {
    pub fn add_hero_and_responses(
        &mut self,
        hero_name: String,
        responses: Vec<(String, String, String)>,
    ) {
        let mut hero_id = HERO_ID.get().unwrap().lock().unwrap();
        self.heroes.insert(
            *hero_id,
            Hero {
                id: *hero_id,
                hero_name: hero_name.clone(),
                img_path: format!("/media/dota2/images/{}.png", hero_name),
            },
        );

        for (original_text, processed_text, response_link) in responses {
            let mut response_id = RESPONSE_ID.get().unwrap().lock().unwrap();
            self.responses.push(Response {
                id: *response_id,
                processed_text,
                original_text,
                response_link,
                hero_id: *hero_id,
            });
            *response_id += 1;
        }
        *hero_id += 1;
    }

    pub fn get_hero_id(&self, name: &str) -> Option<i32> {
        self.heroes
            .iter()
            .find(|(_, h)| h.hero_name == name)
            .map(|(id, _)| *id)
    }

    pub fn get_hero_name(&self, id: i32) -> Option<&str> {
        self.heroes.get(&id).map(|h| h.hero_name.as_str())
    }

    pub fn get_img_dir(&self, id: i32) -> Option<&str> {
        self.heroes.get(&id).map(|h| h.img_path.as_str())
    }

    pub fn get_all_hero_names(&self) -> Vec<&str> {
        self.heroes.values().map(|h| h.hero_name.as_str()).collect()
    }

    pub fn get_response(
        &self,
        processed_text: &str,
        hero_id: Option<i32>,
    ) -> Option<&Response> {
        match hero_id {
            Some(id) => self
                .responses
                .iter()
                .filter(|r| r.processed_text == processed_text && r.hero_id == id)
                .choose(&mut rand::thread_rng()),
            None => self
                .responses
                .iter()
                .filter(|r| r.processed_text == processed_text)
                .choose(&mut rand::thread_rng()),
        }
    }

    pub fn get_icon_url(&self, name: &str) -> Option<&str> {
        self.icons.get(name).map(|s| s.as_str())
    }

    pub fn is_hero_response(&self, processed_text: &str) -> bool {
        self.responses
            .iter()
            .any(|r| r.processed_text == processed_text)
    }

    pub async fn populate_responses(&mut self) {
        HERO_ID.get_or_init(|| Mutex::new(0));
        RESPONSE_ID.get_or_init(|| Mutex::new(0));

        let _ = self.populate_hero_responses().await;
        self.populate_chat_wheel().await;
        self.populate_urls().await;
    }

    async fn populate_hero_responses(&mut self) -> Result<()> {
        let pages = {
            let params = {
                let mut category_params = HashMap::new();
                category_params.insert("action", "query");
                category_params.insert("list", "categorymembers");
                category_params.insert("cmlimit", "max");
                category_params.insert("cmprop", "title");
                category_params.insert("format", "json");
                category_params.insert("cmtitle", "Category: Responses");
                category_params
            };
            let url = reqwest::Url::parse_with_params(API_PATH, params)?;
            let json_response = reqwest::get(url).await?;
            let pages_response = match json_response.json::<PagesResponse>().await {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            };

            let mut pages = vec![];

            for category_members in pages_response.query.categorymembers {
                pages.push(category_members.title);
            }

            pages
        };

        // TODO: handle errors rather than simply throwing them away

        let get_fut_and_names = pages.iter().map(|page| {
            let hero_name = if is_hero_type(page) {
                get_hero_name(page)
            } else {
                page.clone()
            };

            tracing::info!("Fetching responses for {}", hero_name);

            let params = HashMap::from([("action", "raw")]);
            let url = reqwest::Url::parse_with_params(
                &format!("{}/{}", URL_BASE, page),
                params,
            )
            .unwrap();
            (reqwest::get(url).fuse(), hero_name)
        });

        let (get_fut, hero_names): (Vec<_>, Vec<_>) =
            get_fut_and_names.into_iter().unzip();

        // await all the futures, filter out the errors
        // note: we also have to filter out the errors in the hero names
        let text_fut_and_names = futures::future::join_all(get_fut)
            .await
            .into_iter()
            .zip(hero_names.into_iter())
            .filter_map(|(r, name)| match r {
                Ok(v) => Some((v, name)),
                Err(_) => None, // TODO here
            })
            .map(|(r, name)| (r.text().fuse(), name))
            .collect::<Vec<_>>();

        // we need the vec of futures separately for the next step
        let (text_fut, hero_names): (Vec<_>, Vec<_>) =
            text_fut_and_names.into_iter().unzip();

        // await all the futures, filter out the errors
        // note: we also have to filter out the errors in the hero names
        let text_and_names = futures::future::join_all(text_fut)
            .await
            .into_iter()
            .zip(hero_names)
            .filter_map(|(r, name)| match r {
                Ok(v) => Some((v, name)),
                Err(_) => None, // TODO here
            })
            .collect::<Vec<_>>();

        let mut futures = vec![];
        let mut hero_names = vec![];
        for (responses_source, hero_name) in text_and_names {
            tracing::info!("Creating response list for {}", hero_name);
            let response_link_list_fut =
                create_responses_text_and_link_list(responses_source);
            futures.push(response_link_list_fut);
            hero_names.push(hero_name);
        }

        let responses = join_all(futures).await;
        for (response, hero_name) in responses.into_iter().zip(hero_names) {
            tracing::info!("Adding responses for {}", hero_name);
            self.add_hero_and_responses(hero_name, response);
        }

        Ok(())
    }

    async fn populate_chat_wheel(&mut self) {
        tracing::warn!(
            "populate_chat_wheel not implemented, some responses may be missing"
        );
        // TODO
    }

    async fn populate_urls(&mut self) {
        // TODO parse urls automatically
        // for now, just read them from the file
        let json_blob = std::fs::read_to_string("urls.json").unwrap();
        let url: IconUrls = serde_json::from_str(&json_blob).unwrap();
        self.icons = url.0;
    }
}

fn is_hero_type(page: &str) -> bool {
    page.ends_with("/Responses")
}

fn get_hero_name(page: &str) -> String {
    page.split('/').next().unwrap().to_string()
}

async fn create_responses_text_and_link_list(
    responses_source: String,
) -> Vec<(String, String, String)> {
    let mut responses: Vec<(String, String, String)> = vec![];
    let Ok(file_and_text_list) =
        parse_all_response_lines(&mut responses_source.as_str())
    else {
        return responses;
    };

    let files_list = file_and_text_list
        .iter()
        .map(|response| &response.file)
        .collect::<Vec<&String>>();
    let file_and_link_map = links_for_files(&files_list).await;

    for response in file_and_text_list.into_iter() {
        let processed_text = process_text(&response.response);
        if !processed_text.is_empty() {
            let link = file_and_link_map.get(&response.file);
            if let Some(v) = link {
                responses.push((response.response, processed_text, v.clone()))
            } else {
                tracing::warn!("No link found for {}", response.file);
            }
        }
    }

    responses
}

async fn links_for_files(files: &[&String]) -> HashMap<String, String> {
    fn get_params_for_files_api(files: Option<&[String]>) -> HashMap<String, String> {
        let titles = if files.is_some() {
            format!("File:{}", files.unwrap().join("|File:"))
        } else {
            String::new()
        };

        HashMap::from([
            ("action".to_string(), "query".to_string()),
            ("titles".to_string(), titles),
            ("prop".to_string(), "imageinfo".to_string()),
            ("iiprop".to_string(), "url".to_string()),
            ("format".to_string(), "json".to_string()),
        ])
    }

    let max_title_list_length = 50;
    let file_title_prefix_length = "%7CFile%3A".len();
    let max_header_length = 1960;

    let mut files_link_mapping: HashMap<String, String> = HashMap::new();
    let empty_api_length =
        reqwest::Url::parse_with_params(API_PATH, get_params_for_files_api(None))
            .unwrap()
            .to_string()
            .len();

    let mut futures = vec![];
    let mut files_batch_list = vec![];
    let mut current_title_length = 0;
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    for file in files {
        let file_name_len = file_title_prefix_length + file.len();
        if file_name_len + current_title_length >= max_header_length - empty_api_length
            || files_batch_list.len() >= max_title_list_length
        {
            let url = reqwest::Url::parse_with_params(
                API_PATH,
                get_params_for_files_api(Some(&files_batch_list)),
            )
            .unwrap();
            futures.push(client.get(url).send());

            files_batch_list.clear();
            current_title_length = 0;
        }

        files_batch_list.push(file.to_string());
        current_title_length += file_name_len;
    }

    if !files_batch_list.is_empty() {
        let url = reqwest::Url::parse_with_params(
            API_PATH,
            get_params_for_files_api(Some(&files_batch_list)),
        )
        .unwrap();
        futures.push(client.get(url).send());
    }

    let responses = join_all(futures).await;
    for res in responses.into_iter() {
        match res {
            Ok(res) => match res.json::<BatchResponse>().await {
                Ok(json_response) => {
                    let pages = json_response.query.pages;

                    for page in pages.values() {
                        let title = page.title.clone();
                        let image_info = &page.imageinfo;
                        let url = image_info[0].url.clone();
                        let url =
                            format!("{}{}", url.split_once(".mp3").unwrap().0, ".mp3");
                        files_link_mapping
                            .insert(title.chars().skip(5).collect::<String>(), url);
                    }
                }
                Err(e) => {
                    tracing::warn!("{:?}", e);
                }
            },
            Err(e) => {
                tracing::error!("{:?}", e);
            }
        }
    }

    files_link_mapping
}

