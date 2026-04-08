mod api;
mod config;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "discord", about = "Discord CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure webhooks and bot token
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Send a message via webhook
    Send {
        /// Message content
        content: String,
        /// Webhook name (uses default if not specified)
        #[arg(short, long)]
        webhook: Option<String>,
    },
    /// Send an embed via webhook
    Embed {
        /// Embed title
        #[arg(short, long)]
        title: Option<String>,
        /// Embed description
        #[arg(short, long)]
        description: Option<String>,
        /// Embed color (hex, e.g., 0x00ff00)
        #[arg(short, long)]
        color: Option<String>,
        /// Webhook name (uses default if not specified)
        #[arg(short, long)]
        webhook: Option<String>,
    },
    /// List guilds (bot only)
    Guilds,
    /// List channels in a guild (bot only)
    Channels {
        /// Guild ID
        guild_id: String,
    },
    /// Read messages from a channel (bot only)
    Messages {
        /// Channel ID
        channel_id: String,
        /// Number of messages to fetch
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },
    /// Send a message to a channel (bot only)
    Message {
        /// Channel ID
        channel_id: String,
        /// Message content
        content: String,
    },
    /// Create a webhook in a channel (bot only)
    CreateWebhook {
        /// Channel ID
        channel_id: String,
        /// Webhook name
        name: String,
        /// Save to config with this alias
        #[arg(short, long)]
        save_as: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Add a webhook
    AddWebhook {
        /// Name for this webhook
        name: String,
        /// Webhook URL
        url: String,
    },
    /// Remove a webhook
    RemoveWebhook {
        /// Webhook name to remove
        name: String,
    },
    /// Set the default webhook
    DefaultWebhook {
        /// Webhook name to use as default
        name: String,
    },
    /// Set bot token
    BotToken {
        /// Bot token
        token: String,
    },
    /// Show current config
    Show,
}

fn get_webhook_url(name: Option<&str>) -> Result<String> {
    let config = config::load();

    let webhook_name = match name {
        Some(n) => n.to_string(),
        None => config.default_webhook.ok_or_else(|| {
            anyhow::anyhow!("No default webhook set. Use 'discord config default-webhook <name>'")
        })?,
    };

    config
        .webhooks
        .get(&webhook_name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Webhook '{}' not found", webhook_name))
}

fn get_bot_client() -> Result<api::BotClient> {
    let config = config::load();
    let token = config.bot_token.ok_or_else(|| {
        anyhow::anyhow!("No bot token configured. Use 'discord config bot-token <token>'")
    })?;
    Ok(api::BotClient::new(&token))
}

fn handle_config(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::AddWebhook { name, url } => {
            let mut config = config::load();
            config.webhooks.insert(name.clone(), url);
            if config.default_webhook.is_none() {
                config.default_webhook = Some(name.clone());
            }
            config::save(&config)?;
            println!("Added webhook '{}'", name);
        }
        ConfigCommands::RemoveWebhook { name } => {
            let mut config = config::load();
            if config.webhooks.remove(&name).is_some() {
                if config.default_webhook.as_ref() == Some(&name) {
                    config.default_webhook = None;
                }
                config::save(&config)?;
                println!("Removed webhook '{}'", name);
            } else {
                bail!("Webhook '{}' not found", name);
            }
        }
        ConfigCommands::DefaultWebhook { name } => {
            let mut config = config::load();
            if !config.webhooks.contains_key(&name) {
                bail!("Webhook '{}' not found", name);
            }
            config.default_webhook = Some(name.clone());
            config::save(&config)?;
            println!("Default webhook set to '{}'", name);
        }
        ConfigCommands::BotToken { token } => {
            let mut config = config::load();
            config.bot_token = Some(token);
            config::save(&config)?;
            println!("Bot token saved");
        }
        ConfigCommands::Show => {
            let config = config::load();
            println!("Webhooks:");
            for (name, url) in &config.webhooks {
                let default_marker = if config.default_webhook.as_ref() == Some(name) {
                    " (default)"
                } else {
                    ""
                };
                // Mask URL for security
                let masked = if url.len() > 50 {
                    format!("{}...{}", &url[..40], &url[url.len() - 10..])
                } else {
                    url.clone()
                };
                println!("  {}{}: {}", name, default_marker, masked);
            }
            println!(
                "Bot token: {}",
                if config.bot_token.is_some() {
                    "configured"
                } else {
                    "not set"
                }
            );
        }
    }
    Ok(())
}

async fn send_webhook_message(content: String, webhook: Option<String>) -> Result<()> {
    let url = get_webhook_url(webhook.as_deref())?;
    let client = api::WebhookClient::new(&url);
    client.send(&content).await?;
    println!("Message sent");
    Ok(())
}

async fn send_webhook_embed(
    title: Option<String>,
    description: Option<String>,
    color: Option<String>,
    webhook: Option<String>,
) -> Result<()> {
    let url = get_webhook_url(webhook.as_deref())?;
    let client = api::WebhookClient::new(&url);
    let color_val = color
        .map(|c| {
            let c = c.trim_start_matches("0x").trim_start_matches('#');
            u32::from_str_radix(c, 16)
        })
        .transpose()?;
    client
        .send_embed(title.as_deref(), description.as_deref(), color_val)
        .await?;
    println!("Embed sent");
    Ok(())
}

async fn list_guilds() -> Result<()> {
    let client = get_bot_client()?;
    let guilds = client.guilds().await?;
    println!("{}", serde_json::to_string_pretty(&guilds)?);
    Ok(())
}

async fn list_channels(guild_id: String) -> Result<()> {
    let client = get_bot_client()?;
    let channels = client.channels(&guild_id).await?;
    println!("{}", serde_json::to_string_pretty(&channels)?);
    Ok(())
}

async fn fetch_messages(channel_id: String, limit: u32) -> Result<()> {
    let client = get_bot_client()?;
    let messages = client.messages(&channel_id, limit).await?;
    println!("{}", serde_json::to_string_pretty(&messages)?);
    Ok(())
}

async fn send_channel_message(channel_id: String, content: String) -> Result<()> {
    let client = get_bot_client()?;
    let result = client.send_message(&channel_id, &content).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn create_webhook(channel_id: String, name: String, save_as: Option<String>) -> Result<()> {
    let client = get_bot_client()?;
    let result = client.create_webhook(&channel_id, &name).await?;
    let url = result["url"].as_str().unwrap_or("");
    println!("Created webhook: {}", url);

    if let Some(alias) = save_as {
        let mut cfg = config::load();
        cfg.webhooks.insert(alias.clone(), url.to_string());
        if cfg.default_webhook.is_none() {
            cfg.default_webhook = Some(alias.clone());
        }
        config::save(&cfg)?;
        println!("Saved as '{}'", alias);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { command } => handle_config(command)?,
        Commands::Send { content, webhook } => send_webhook_message(content, webhook).await?,
        Commands::Embed { title, description, color, webhook } => {
            send_webhook_embed(title, description, color, webhook).await?
        }
        Commands::Guilds => list_guilds().await?,
        Commands::Channels { guild_id } => list_channels(guild_id).await?,
        Commands::Messages { channel_id, limit } => fetch_messages(channel_id, limit).await?,
        Commands::Message { channel_id, content } => {
            send_channel_message(channel_id, content).await?
        }
        Commands::CreateWebhook { channel_id, name, save_as } => {
            create_webhook(channel_id, name, save_as).await?
        }
    }

    Ok(())
}
