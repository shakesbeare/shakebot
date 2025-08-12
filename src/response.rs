use crate::serde_response::*;
use anyhow::Result;
use futures::future::join_all;
use futures::FutureExt as _;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

use rand::seq::IteratorRandom as _;

const DOTA_URL_BASE: &str = "http://dota2.gamepedia.com";
const DOTA_API_PATH: &str = "http://dota2.gamepedia.com/api.php";

const SMITE_URL_BASE: &str = "http://smite.fandom.com/";
const SMITE_API_PATH: &str = "http://smite.fandom.com/api.php";

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Response {
    pub id: i32,
    pub processed_text: String,
    pub original_text: String,
    pub response_link: String,
    pub hero_id: i32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Hero {
    id: i32,
    hero_name: String,
    img_path: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ResponseDatabase {
    pub responses: Vec<Response>,
    pub heroes: HashMap<i32, Hero>,
    pub icons: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum Game {
    Dota,
    Smite,
}

static HERO_ID: OnceLock<Mutex<i32>> = OnceLock::new();
static RESPONSE_ID: OnceLock<Mutex<i32>> = OnceLock::new();

impl ResponseDatabase {
    async fn get_pages_for(&mut self, game: Game) -> Result<Vec<String>> {
        match game {
            Game::Dota => {
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
                let url = reqwest::Url::parse_with_params(DOTA_API_PATH, params)?;
                let json_response = reqwest::get(url).await?;
                let pages_response = match json_response.json::<PagesResponse>().await {
                    Ok(v) => v,
                    Err(e) => return Err(e.into()),
                };

                let mut pages = vec![];

                for category_members in pages_response.query.categorymembers {
                    pages.push(category_members.title);
                }

                Ok(pages)
            }
            Game::Smite => {
                let params = {
                    let mut category_params = HashMap::new();
                    category_params.insert("action", "query");
                    category_params.insert("list", "categorymembers");
                    category_params.insert("cmlimit", "max");
                    category_params.insert("cmprop", "title");
                    category_params.insert("format", "json");
                    category_params.insert("cmtitle", "Category: Voicelines");
                    category_params
                };
                // the smite voicelines page is actually a page full of a categories
                // so, for each of these pages, we have to query again to get the correct
                // pages
                let url = reqwest::Url::parse_with_params(SMITE_API_PATH, params.clone())?;
                tracing::info!("Acquiring voice line categories");
                let json_response = reqwest::get(url).await?;
                let pages_response = match json_response.json::<PagesResponse>().await {
                    Ok(v) => v,
                    Err(e) => return Err(e.into()),
                };
                let mut pages = vec![];

                for category_members in pages_response.query.categorymembers {
                    let mut params = params.clone();

                    // Don't want the pages that lead directly to voicelines
                    if !category_members.title.to_string().starts_with("Category") {
                        continue;
                    }

                    tracing::info!("Acquiring categories in {}", &category_members.title);
                    params.insert("cmtitle", &category_members.title);
                    let url = reqwest::Url::parse_with_params(SMITE_API_PATH, params)?;
                    let json_response = reqwest::get(url).await?;
                    let pages_response = match json_response.json::<PagesResponse>().await {
                        Ok(v) => v,
                        Err(e) => return Err(e.into()),
                    };

                    for members_inner in pages_response.query.categorymembers {
                        pages.push(members_inner.title);
                    }
                }

                Ok(pages)
            }
        }
    }

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

    pub fn get_response(&self, processed_text: &str, hero_id: Option<i32>) -> Option<&Response> {
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

        tracing::info!("Populating hero responses");
        let _ = self.populate_hero_responses(Game::Dota).await;
        let _ = self.populate_hero_responses(Game::Smite).await;
        tracing::info!("Populating chat wheel responses");
        self.populate_chat_wheel().await;
        tracing::info!("Populating urls");
        self.populate_urls().await;
    }

    async fn populate_hero_responses(&mut self, game: Game) -> Result<()> {
        let pages = self.get_pages_for(game).await.unwrap();
        // TODO: handle errors rather than simply throwing them away

        let get_fut_and_names = pages.iter().map(|page| {
            let hero_name = if is_hero_type(page) {
                get_hero_name(page)
            } else {
                page.clone()
            };

            tracing::info!("Fetching responses for {}", hero_name);

            let params = HashMap::from([("action", "raw")]);
            let url = match game {
                Game::Dota => {
                    reqwest::Url::parse_with_params(&format!("{}/{}", DOTA_URL_BASE, page), params)
                        .unwrap()
                }
                Game::Smite => {
                    reqwest::Url::parse_with_params(&format!("{}/{}", SMITE_URL_BASE, page), params)
                        .unwrap()
                }
            };
            (reqwest::get(url).fuse(), hero_name)
        });

        let (get_fut, hero_names): (Vec<_>, Vec<_>) = get_fut_and_names.into_iter().unzip();

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
        let (text_fut, hero_names): (Vec<_>, Vec<_>) = text_fut_and_names.into_iter().unzip();

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
                create_responses_text_and_link_list(responses_source, game);
            futures.push(response_link_list_fut);
            hero_names.push(hero_name);
        }

        let responses = join_all(futures).await;
        for (response, hero_name) in responses.into_iter().zip(hero_names) {
            tracing::info!("Adding responses for {}", hero_name);
            self.add_hero_and_responses(hero_name, response);
        }

        tracing::info!("Hero responses complete");
        Ok(())
    }

    async fn populate_chat_wheel(&mut self) {
        tracing::warn!("populate_chat_wheel not implemented, some responses may be missing");
        // TODO
    }

    async fn populate_urls(&mut self) {
        // TODO parse urls automatically
        // for now, just read them from the file
        let json_blob = std::fs::read_to_string("urls.json").unwrap();
        let url: IconUrls = serde_json::from_str(&json_blob).unwrap();
        self.icons = url.0;
        tracing::info!("Urls complete");
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
    game: Game,
) -> Vec<(String, String, String)> {
    let mut responses: Vec<(String, String, String)> = vec![];
    let Ok(file_and_text_list) =
        crate::parsing::parse_all_response_lines(&mut responses_source.as_str())
    else {
        return responses;
    };

    let files_list = file_and_text_list
        .iter()
        .map(|response| &response.file)
        .collect::<Vec<&String>>();
    let file_and_link_map = links_for_files(&files_list, game).await;

    for response in file_and_text_list.into_iter() {
        let processed_text = crate::process_text(&response.response);
        if !processed_text.is_empty() {
            let link = file_and_link_map.get(&response.file);
            if let Some(v) = link {
                responses.push((response.response, processed_text, v.clone()));
            } else {
                tracing::warn!("No link found for {}", response.file);
            }
        }
    }

    responses
}

async fn links_for_files(files: &[&String], game: Game) -> HashMap<String, String> {
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
    let empty_api_length = match game {
        Game::Dota => {
            reqwest::Url::parse_with_params(DOTA_API_PATH, get_params_for_files_api(None))
                .unwrap()
                .to_string()
                .len()
        }
        Game::Smite => {
            reqwest::Url::parse_with_params(SMITE_API_PATH, get_params_for_files_api(None))
                .unwrap()
                .to_string()
                .len()
        }
    };

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
            let url = match game {
                Game::Dota => reqwest::Url::parse_with_params(
                    DOTA_API_PATH,
                    get_params_for_files_api(Some(&files_batch_list)),
                )
                .unwrap(),
                Game::Smite => reqwest::Url::parse_with_params(
                    SMITE_API_PATH,
                    get_params_for_files_api(Some(&files_batch_list)),
                )
                .unwrap(),
            };
            futures.push(client.get(url).send());

            files_batch_list.clear();
            current_title_length = 0;
        }

        files_batch_list.push(file.to_string());
        current_title_length += file_name_len;
    }

    if !files_batch_list.is_empty() {
        let url = match game {
            Game::Dota => reqwest::Url::parse_with_params(
                DOTA_API_PATH,
                get_params_for_files_api(Some(&files_batch_list)),
            )
            .unwrap(),
            Game::Smite => reqwest::Url::parse_with_params(
                SMITE_API_PATH,
                get_params_for_files_api(Some(&files_batch_list)),
            )
            .unwrap(),
        };
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
                        // let url = format!("{}{}", url.split_once(".ogg").unwrap().0, ".ogg");
                        files_link_mapping.insert(title.chars().skip(5).collect::<String>(), url);
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
