mod commands;
mod dota;

use poise::serenity_prelude as serenity;

use ::serenity::builder::{CreateThread, EditThread};
use serenity::{
    all::{GatewayIntents, Message},
    async_trait,
    builder::{CreateAttachment, CreateMessage},
    client::{Context, EventHandler},
    Client,
};

use crate::bot::commands::*;
use crate::dota::response::Response;
use crate::BOT_NAMES;
use crate::{process_text, DATA};

pub struct Bot {}
struct Data {}

impl Bot {
    pub fn new() -> Bot {
        Bot {}
    }

    pub async fn start(&mut self) {
        dotenv::dotenv().ok();
        let token = std::env::var("DISCORD_TOKEN")
            .expect("Expected a token in the environment");
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let poise_options = poise::FrameworkOptions {
            commands: vec![copypasta(), help(), disable(), enable()],
            ..Default::default()
        };

        let framework = poise::Framework::builder()
            .setup(move |ctx, _ready, framework| {
                Box::pin(async move {
                    println!("Logged in as {}", _ready.user.name);
                    poise::builtins::register_globally(
                        ctx,
                        &framework.options().commands,
                    )
                    .await?;
                    Ok(Data {})
                })
            })
            .options(poise_options)
            .build();

        let mut client = Client::builder(&token, intents)
            .event_handler(Handler)
            .framework(framework)
            .await
            .expect("Error creating client");

        if let Err(e) = client.start().await {
            tracing::error!("Error starting clientr: {:?}", e);
        }
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if BOT_NAMES.contains(&msg.author.name.as_str()) {
            return;
        }

        if crate::DATA
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .disabled_users
            .contains(&msg.author.id.to_string())
        {
            return;
        }

        let res = self.get_response(&msg.content);
        if let Some(res) = res {
            tracing::debug!("Response found: {:?}", res);
            let bytes: Vec<u8> = reqwest::get(&res.response_link)
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec();
            let embed = dota::dota_response_embed(res.hero_id);
            let message = CreateMessage::new().add_embed(embed);
            if let Ok(msg) = msg.channel_id.send_message(&ctx.http, message).await {
                let attachment = CreateAttachment::bytes(
                    bytes,
                    format!("{}.mp3", &res.original_text),
                );
                let message = CreateMessage::new().add_file(attachment);
                let thread_builder = CreateThread::new(res.original_text);
                let thread = msg.channel_id.create_thread_from_message(
                    &ctx.http,
                    msg,
                    thread_builder,
                );
                if let Ok(mut t) = thread.await {
                    t.say(&ctx.http, &res.response_link).await.unwrap();
                    t.send_message(&ctx.http, message).await.unwrap();
                    let edit_thread = EditThread::new().archived(true);
                    t.edit_thread(&ctx.http, edit_thread).await.unwrap();
                } else {
                    tracing::error!("Error creating thread");
                }
            } else {
                tracing::error!("Error sending message");
            }
        } else {
            tracing::debug!("No response found for: {}", msg.content);
        }
    }
}

impl Handler {
    pub fn get_response(&self, text: &str) -> Option<Response> {
        let data = DATA.get().unwrap().lock().unwrap();
        let processed_text = process_text(text);
        data.dota
            .responses
            .get_response(&processed_text, None)
            .cloned()
    }
}

/// Splits the contents of msg into chunks of 2000 characters
pub fn split_large_message(msg: &str) -> Vec<&str> {
    // TODO: ensure that splits happen on spaces
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut end = 2000;
    if end > msg.len() {
        end = msg.len();
    }
    while end < msg.len() {
        chunks.push(&msg[start..end]);
        start = end;
        end += 2000;
        if end > msg.len() {
            end = msg.len();
        }
    }
    chunks.push(&msg[start..end]);

    chunks
}
