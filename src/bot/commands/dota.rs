use crate::bot::dota::dota_response_embed;

use super::Context;
use anyhow::Error;
use fuzzy_matcher::FuzzyMatcher;
use poise::CreateReply;
use serenity::builder::{CreateAttachment, CreateMessage, CreateThread, EditThread};

/// Don't allow the bot to send dota responses to your messages
#[poise::command(slash_command)]
pub async fn disable(ctx: Context<'_>) -> Result<(), Error> {
    {
        let mut data = crate::DATA.get().unwrap().lock().unwrap();
        data.disabled_users.push(ctx.author().id.to_string());
    }
    ctx.say("You have disabled dota responses").await?;
    Ok(())
}

/// Allow the bot to send dota responses to your messages
#[poise::command(slash_command)]
pub async fn enable(ctx: Context<'_>) -> Result<(), Error> {
    {
        let mut data = crate::DATA.get().unwrap().lock().unwrap();
        data.disabled_users
            .retain(|id| id != &ctx.author().id.to_string());
    }
    ctx.say("You have enabled dota responses").await?;
    Ok(())
}

/// Query for a Dota response
#[poise::command(slash_command)]
pub async fn dota(
    ctx: Context<'_>,
    #[description = "The phrase to match against"] phrase: String,
) -> Result<(), Error> {
    let res = {
        let data = crate::DATA.get().unwrap().lock().unwrap();
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let highest_scorer = data
            .dota
            .responses
            .responses
            .iter()
            .map(|r| matcher.fuzzy_match(&r.original_text, &phrase))
            .enumerate()
            .max_by_key(|(_, score)| *score);
        data.dota
            .responses
            .responses
            .get(highest_scorer.unwrap().0)
            .cloned()
            .unwrap()
    };
    let embed = dota_response_embed(res.hero_id);
    let message = CreateReply::default().embed(embed);
    let bytes: Vec<u8> = reqwest::get(&res.response_link)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .to_vec();
    if let Ok(msg) = ctx.send(message).await {
        let attachment =
            CreateAttachment::bytes(bytes, format!("{}.mp3", &res.original_text));
        let message = CreateMessage::new().add_file(attachment);
        let thread_builder = CreateThread::new(res.original_text);
        let msg = msg.into_message().await.unwrap();
        let ctx = &ctx;
        let thread = msg
            .channel(&ctx.http())
            .await
            .unwrap()
            .id()
            .create_thread_from_message(ctx.http(), msg, thread_builder);
        if let Ok(mut t) = thread.await {
            t.say(&ctx.http(), &res.response_link).await.unwrap();
            t.send_message(&ctx.http(), message).await.unwrap();
            let edit_thread = EditThread::new().archived(true);
            t.edit_thread(&ctx.http(), edit_thread).await.unwrap();
        } else {
            tracing::error!("Error creating thread");
        }
    } else {
        tracing::error!("Error sending message");
    }
    Ok(())
}
