use std::collections::HashMap;

use self::{
    patch_notes::{PatchNotes, Version},
    response::ResponseDatabase,
};

pub mod parsing;
pub mod patch_notes;
pub mod response;
pub mod serde_response;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Dota {
    pub version: String,
    pub patch_notes: HashMap<String, Version>,
    pub responses: ResponseDatabase,
}

impl Dota {
    pub async fn check_for_updates(&mut self) {
        // TODO: find a way to check for updates without having to download the entire patch notes
        let patch_notes = PatchNotes::get().await.unwrap();
        let latest_version = patch_notes.get_latest_version().await.unwrap();
        if latest_version != self.version {
            self.patch_notes = patch_notes.versions;
            self.version = latest_version;

            self.responses.responses.clear();
            self.responses.heroes.clear();
            self.responses.populate_responses().await;
        }
    }

    pub async fn possible_next_versions(&self) -> (String, String, String) {
        let Some((major, minor)) = self.version.split_once('_') else {
            panic!("Invalid version number");
        };
        let (minor, patch) = minor.split_at(1);

        let n_major = major.parse::<u32>().unwrap() + 1;
        let n_minor = minor.parse::<u32>().unwrap() + 1;
        let n_patch =
            std::char::from_u32(patch.chars().next().unwrap() as u32 + 1).unwrap();
        (
            format!("{}_{}{}", n_major, minor, patch),
            format!("{}_{}{}", major, n_minor, patch),
            format!("{}_{}{}", major, minor, n_patch),
        )
    }
}
