use serenity::{model::Color, builder::{CreateEmbed, CreateEmbedFooter}};

use crate::DATA;

pub fn dota_response_embed(hero_id: i32) -> CreateEmbed {
    let data = DATA.get().unwrap().lock().unwrap();
    let db = &data.dota.responses;
    let footer_text = "Dota 2 Hero Responses".to_string();
    let Some(hero_name) = db.get_hero_name(hero_id) else {
        tracing::info!("No hero found for id: {}", hero_id);
        unreachable!();
    };
    let hero_name_fmt = hero_name.to_string().replace(' ', "").to_lowercase();
    let mut icon_url = db.get_icon_url(&hero_name_fmt);
    if icon_url.is_none() {
        let tmp = &hero_name_fmt.to_string().replace("announcerpack", "");
        icon_url = db.get_icon_url(tmp);
    }

    match icon_url {
        Some(url) => {
            let embed_footer = CreateEmbedFooter::new(footer_text)
                .icon_url(url);
            
                CreateEmbed::new()
                .description(hero_name)
                .colour(Color::BLUE)
                .footer(embed_footer)
        }
        None => {
            let embed_footer = CreateEmbedFooter::new(footer_text);
            
                CreateEmbed::new()
                .description(hero_name)
                .colour(Color::BLUE)
                .footer(embed_footer)
        }
    }
}
