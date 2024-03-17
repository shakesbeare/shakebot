use crate::dota::patch_notes::PatchNotes;
#[test]
fn test_patch_notes_deserialize() {
    let json = r#"{
    "7.35c" : {
        "general" : [
            "Fixed a bug with the new hero"
        ],
        "items" : {
            "item_name" : [
                "Fixed a bug with the item"
            ]
        },
        "heroes" : {
            "misc" : [
                "Fixed a bug with the hero"
            ],
            "hero_name" : {
                "hero_detail" : [
                    "Fixed a bug with the hero"
                ]
            }
        }
    }
}
"#;

    match serde_json::from_str::<PatchNotes>(json) {
        Ok(_) => {}
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}

#[test]
fn latest_version() {
    let json = r#"{
    "7.24c" : {
        "general" : [
            "Fixed a bug with the new hero"
        ],
        "items" : {
            "item_name" : [
                "Fixed a bug with the item"
            ]
        },
        "heroes" : {
            "misc" : [
                "Fixed a bug with the hero"
            ],
            "hero_name" : {
                "hero_detail" : [
                    "Fixed a bug with the hero"
                ]
            }
        }
    },
    "7.35c" : {
        "general" : [
            "Fixed a bug with the new hero"
        ],
        "items" : {
            "item_name" : [
                "Fixed a bug with the item"
            ]
        },
        "heroes" : {
            "misc" : [
                "Fixed a bug with the hero"
            ],
            "hero_name" : {
                "hero_detail" : [
                    "Fixed a bug with the hero"
                ]
            }
        }
    },
    "7.15c" : {
        "general" : [
            "Fixed a bug with the new hero"
        ],
        "items" : {
            "item_name" : [
                "Fixed a bug with the item"
            ]
        },
        "heroes" : {
            "misc" : [
                "Fixed a bug with the hero"
            ],
            "hero_name" : {
                "hero_detail" : [
                    "Fixed a bug with the hero"
                ]
            }
        }
    }
}
"#;

    let patch_notes: PatchNotes = serde_json::from_str(json).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn_blocking(|| async move {
        let latest_version = patch_notes.get_latest_version().await.unwrap();
        assert_eq!(latest_version, "7.35c");
    });
}
