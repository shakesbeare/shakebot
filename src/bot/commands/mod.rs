use std::collections::HashMap;

use anyhow::Error;
use poise::samples::HelpConfiguration;

use super::Data;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug, Clone, serde::Deserialize)]
struct Copypastas(HashMap<String, Copypasta>);

#[derive(Debug, Clone, serde::Deserialize)]
struct Copypasta {
    content: String,
    guild: String,
}

async fn meme_helper(ctx: Context<'_>, text: &str) -> Result<(), Error> {
    let mut first_msg = true;

    if text.len() > 2000 {
        let chunks = super::split_large_message(text);
        for chunk in chunks {
            if first_msg {
                if let Err(e) = ctx.say(chunk).await {
                    tracing::error!("{:?}", e);
                }
                first_msg = false;
            } else if let Err(e) = ctx.channel_id().say(&ctx.http(), chunk).await {
                tracing::error!("{:?}", e);
            }
        }
        return Ok(());
    } else if let Err(e) = ctx.say(text).await {
        tracing::error!("{:?}", e);
    }

    Ok(())
}

/// send a copypasta
#[poise::command(slash_command)]
pub async fn copypasta(
    ctx: Context<'_>,
    #[description = "The name of the copypasta"] name: String,
) -> Result<(), Error> {
    let copypastas = serde_json::from_str::<Copypastas>(&std::fs::read_to_string(
        "copypastas.json",
    )?)?;
    if let Some(copypasta) = copypastas.0.get(&name) {
        meme_helper(ctx, &copypasta.content).await?;
    } else {
        ctx.say("No copypasta found with that name").await?;
    }

    Ok(())
}

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

/// Show help message
#[poise::command(slash_command, track_edits, category = "Utility")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Command to get help for"]
    #[rest]
    mut command: Option<String>,
) -> Result<(), Error> {
    // This makes it possible to just make `help` a subcommand of any command
    // `/fruit help` turns into `/help fruit`
    // `/fruit help apple` turns into `/help fruit apple`
    if ctx.invoked_command_name() != "help" {
        command = match command {
            Some(c) => Some(format!("{} {}", ctx.invoked_command_name(), c)),
            None => Some(ctx.invoked_command_name().to_string()),
        };
    }
    let extra_text_at_bottom = "\
Type `/help command` for more info on a command.
You can edit your `/help` message to the bot and the bot will edit its response.";

    let config = HelpConfiguration {
        show_subcommands: true,
        show_context_menu_commands: true,
        ephemeral: true,
        extra_text_at_bottom,

        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}
