use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

pub struct WebhookClient {
    http: reqwest::Client,
    url: String,
}

impl WebhookClient {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn new(url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            url: url.to_string(),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn send(&self, content: &str) -> Result<()> {
        let response = self
            .http
            .post(&self.url)
            .json(&json!({ "content": content }))
            .send()
            .await
            .context("Failed to send webhook")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("Webhook failed with status {}: {}", status, body);
        }

        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn send_embed(
        &self,
        title: Option<&str>,
        description: Option<&str>,
        color: Option<u32>,
    ) -> Result<()> {
        let mut embed = json!({});
        if let Some(t) = title {
            embed["title"] = json!(t);
        }
        if let Some(d) = description {
            embed["description"] = json!(d);
        }
        if let Some(c) = color {
            embed["color"] = json!(c);
        }

        let response = self
            .http
            .post(&self.url)
            .json(&json!({ "embeds": [embed] }))
            .send()
            .await
            .context("Failed to send webhook")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("Webhook failed with status {}: {}", status, body);
        }

        Ok(())
    }
}

pub struct BotClient {
    http: reqwest::Client,
    token: String,
}

impl BotClient {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn new(token: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            token: token.to_string(),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    async fn request(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<&Value>,
    ) -> Result<Value> {
        let url = format!("https://discord.com/api/v10{}", endpoint);
        let mut req = self
            .http
            .request(method, &url)
            .header("Authorization", format!("Bot {}", self.token));
        if let Some(b) = body {
            req = req.json(b);
        }
        let response = req.send().await.context("API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("API error {}: {}", status, body);
        }

        response.json().await.context("Failed to parse response")
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    async fn get(&self, endpoint: &str) -> Result<Value> {
        self.request(reqwest::Method::GET, endpoint, None).await
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    async fn post(&self, endpoint: &str, body: &Value) -> Result<Value> {
        self.request(reqwest::Method::POST, endpoint, Some(body))
            .await
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn guilds(&self) -> Result<Value> {
        self.get("/users/@me/guilds").await
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn channels(&self, guild_id: &str) -> Result<Value> {
        self.get(&format!("/guilds/{}/channels", guild_id)).await
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn messages(&self, channel_id: &str, limit: u32) -> Result<Value> {
        self.get(&format!(
            "/channels/{}/messages?limit={}",
            channel_id, limit
        ))
        .await
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<Value> {
        self.post(
            &format!("/channels/{}/messages", channel_id),
            &json!({ "content": content }),
        )
        .await
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn create_webhook(&self, channel_id: &str, name: &str) -> Result<Value> {
        self.post(
            &format!("/channels/{}/webhooks", channel_id),
            &json!({ "name": name }),
        )
        .await
    }
}
