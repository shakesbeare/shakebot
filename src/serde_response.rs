use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
pub struct PagesResponse {
    #[serde(skip)]
    #[allow(dead_code)]
    pub batchcomplete: String,
    #[serde(skip)]
    #[allow(dead_code)]
    pub limits: String,
    pub query: Query,
}

#[derive(Debug, serde::Deserialize)]
pub struct Query {
    pub categorymembers: Vec<ResponseCategoryMember>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ResponseCategoryMember {
    #[serde(skip)]
    #[allow(dead_code)]
    pub ns: i32,
    pub title: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchResponse {
    pub query: BatchQuery,
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchQuery {
    #[serde(skip)]
    #[allow(dead_code)]
    pub normalized: String,
    pub pages: HashMap<String, BatchPage>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchPage {
    #[serde(skip)]
    #[allow(dead_code)]
    pub pageid: i32,
    #[serde(skip)]
    #[allow(dead_code)]
    pub ns: i32,
    pub title: String,
    pub imageinfo: Vec<ImageInfo>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ImageInfo {
    pub url: String,
    #[allow(dead_code)]
    #[serde(alias = "descriptionurl", skip)]
    pub description_url: String,
    #[allow(dead_code)]
    #[serde(alias = "descriptionshorturl", skip)]
    pub description_short_url: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct IconUrls(pub HashMap<String, String>);
