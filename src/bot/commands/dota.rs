use super::Context;
use anyhow::Error;

/// Don't allow the bot to send hero responses to your messages
#[poise::command(slash_command)]
pub async fn disable(ctx: Context<'_>) -> Result<(), Error> {
    {
        let mut data = crate::DATA.get().unwrap().lock().unwrap();
        data.disabled_users.push(ctx.author().id.to_string());
    }
    ctx.say("You have disabled hero responses").await?;
    Ok(())
}

/// Allow the bot to send hero responses to your messages
#[poise::command(slash_command)]
pub async fn enable(ctx: Context<'_>) -> Result<(), Error> {
    {
        let mut data = crate::DATA.get().unwrap().lock().unwrap();
        data.disabled_users
            .retain(|id| id != &ctx.author().id.to_string());
    }
    ctx.say("You have enabled hero responses").await?;
    Ok(())
}
