mod commands;
mod dota;

use std::sync::{Mutex, OnceLock};

use poise::serenity_prelude as serenity;

use serenity::{
    all::{GatewayIntents, Message},
    async_trait,
    builder::CreateMessage,
    client::{Context, EventHandler},
    Client,
};

use crate::bot::{commands::*, dota::dota_response_thread};
use commands::dota::*;

use crate::response::Response;
use crate::BOT_NAMES;
use crate::{process_text, DATA};

static DOTA_COOLDOWN: OnceLock<Mutex<i32>> = OnceLock::new();
const MIN_MESSAGES: i32 = 5;

pub struct Bot {}
struct Data {}

impl Bot {
    pub fn new() -> Bot {
        Bot {}
    }

    pub async fn start(&mut self) {
        let _ = DOTA_COOLDOWN.set(Mutex::new(0));
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

        if msg.content.split(' ').count() < 3 {
            return;
        }

        if let Some(mutex) = DOTA_COOLDOWN.get() {
            let mut guard = mutex.lock().unwrap(); // just panic if poisoned
            if *guard > 0 {
                tracing::info!("Dota response is on cooldown...");
                *guard -= 1;
                return;
            } else {
                *guard = MIN_MESSAGES;
            }
        }
        tracing::info!("Received message: {}", &msg.content);

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
            let embed = dota::character_response_embed(res.hero_id);
            let message = CreateMessage::new().add_embed(embed);
            if let Ok(msg) = msg.channel_id.send_message(&ctx.http, message).await {
                dota_response_thread(bytes, &res, &msg, &ctx.http).await;
            } else {
                tracing::error!("Error sending message");
            }
        } else {
            tracing::info!("No response found for: {}", msg.content);
        }
    }
}

impl Handler {
    pub fn get_response(&self, text: &str) -> Option<Response> {
        let data = DATA.get().unwrap().lock().unwrap();
        let processed_text = process_text(text);
        data
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
