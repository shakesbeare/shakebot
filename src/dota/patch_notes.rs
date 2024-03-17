use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use itertools::Itertools;

/// Patch version i.e. 7_35c
type VersionNumber = String;
/// The internal name of the hero, may require translation
type HeroName = String;
/// The internal name of the item, may require translation
type ItemName = String;
/// The title of the ability or trait that has been changed
type HeroDetail = String;
/// A list of the changes made to the specified detail
type Changes = Vec<String>;

/// This doesn't work without serde(flatten) support from ron
/// Not using this makes things a bit more cumbersome
/// TODO: fix this when ron fixes it
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PatchNotes {
    #[serde(flatten)]
    pub versions: HashMap<VersionNumber, Version>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Version {
    pub general: Changes,
    pub items: HashMap<ItemName, Changes>,
    pub heroes: HashMap<HeroName, Hero>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Hero {
    Hero(HashMap<HeroDetail, Changes>),
    Misc(Changes),
}

impl PatchNotes {
    pub async fn get() -> Result<PatchNotes> {
        // retrieve the major patch version of the dota game client
        // assuming that new voicelines are only added in major patches
        // https://raw.githubusercontent.com/odota/dotaconstants/master/build/patchnotes.json
        // should have the most recent patch information
        let res = reqwest::get("https://raw.githubusercontent.com/odota/dotaconstants/master/build/patchnotes.json").await?;
        Ok(res.json::<PatchNotes>().await?)
    }

    /// Returns the latest version of the patch notes currently downloaded
    pub async fn get_latest_version(&self) -> Result<VersionNumber> {
        // keys are unsorted
        let version_numbers = self
            .versions
            .keys()
            .sorted()
            .collect::<Vec<&VersionNumber>>();
        let latest_version = version_numbers.last().context("Patch notes are empty")?;
        Ok(latest_version.to_string())
    }
}
