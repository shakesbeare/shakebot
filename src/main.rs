use std::sync::Mutex;

use anyhow::{Context, Result};

use shake_bot::Data;
use shake_bot::DATA;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let data: Data = match std::fs::read_to_string("data.ron") {
        Ok(d) => ron::from_str(&d)?,
        Err(_) => {
            tracing::info!("No data file found, creating a new one");
            let mut defaults = Data::default();
            let ron_str = ron::to_string(&defaults)?;
            std::fs::write("data.ron", ron_str)?;
            defaults.dota.check_for_updates().await;
            defaults
        }
    };
    tracing::debug!("{} total responses", data.dota.responses.responses.len());
    let _ = DATA.set(Mutex::new(data));

    ctrlc::set_handler(move || match clean_up() {
        Ok(_) => {
            tracing::info!("\nSaved data successfully, goodbye!");
            std::process::exit(0);
        }
        Err(e) => {
            tracing::error!("\nFailed to save data: {}", e);
            tracing::error!("Data will be lost!");
            tracing::info!("Goodbye!");
            std::process::exit(1);
        }
    })
    .expect("Failed to set ctrl-c handler");

    let mut bot = shake_bot::bot::Bot::new();
    bot.start().await;

    Ok(())
}

fn clean_up() -> Result<()> {
    let data = DATA
        .get()
        .context("OnceLock should be populated")?
        .lock()
        .unwrap();
    let string = ron::to_string(&*data)?;
    std::fs::write("data.ron", string).unwrap();
    Ok(())
}
