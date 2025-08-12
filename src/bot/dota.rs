use serenity::{
    builder::{CreateEmbed, CreateEmbedFooter, CreateAttachment, CreateMessage, CreateThread, EditThread},
    model::Color, all::Message, http::Http,
};

use crate::{DATA, response::Response};

pub fn character_response_embed(hero_id: i32) -> CreateEmbed {
    let data = DATA.get().unwrap().lock().unwrap();
    let db = &data.response_database;
    let footer_text = "Hero Responses".to_string();
    let hero_name = db.get_hero_name(hero_id).unwrap_or("Unknown");
    let hero_name_fmt = hero_name.to_string().replace(' ', "").to_lowercase();
    let mut icon_url = db.get_icon_url(&hero_name_fmt);
    if icon_url.is_none() {
        let tmp = &hero_name_fmt.to_string().replace("announcerpack", "");
        icon_url = db.get_icon_url(tmp);
    }

    match icon_url {
        Some(url) => {
            let embed_footer = CreateEmbedFooter::new(footer_text).icon_url(url);

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

pub async fn dota_response_thread(bytes: Vec<u8>, res: &Response, msg: &Message, ctx_http: &Http) {
    let attachment =
        CreateAttachment::bytes(bytes, format!("{}.mp3", &res.original_text));
    let message = CreateMessage::new().add_file(attachment);
    let thread_builder = CreateThread::new(res.original_text.clone());
    let thread =
        msg.channel_id
            .create_thread_from_message(ctx_http, msg, thread_builder);
    if let Ok(mut t) = thread.await {
        t.say(ctx_http, &res.response_link).await.unwrap();
        t.send_message(ctx_http, message).await.unwrap();
        let edit_thread = EditThread::new().archived(true);
        t.edit_thread(ctx_http, edit_thread).await.unwrap();
    } else {
        tracing::error!("Error creating thread");
    }
}
