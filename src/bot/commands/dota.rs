use crate::bot::dota::{dota_response_embed, dota_response_thread};

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
            .response_database
            .responses
            .iter()
            .map(|r| matcher.fuzzy_match(&r.original_text, &phrase))
            .enumerate()
            .max_by_key(|(_, score)| *score);
        data.dota
            .response_database
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
        dota_response_thread(bytes, &res, &msg.into_message().await?, ctx.http()).await;
    } else {
        tracing::error!("Error sending message");
    }
    Ok(())
}
