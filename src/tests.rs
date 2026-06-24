use super::*;
use clap::CommandFactory;

fn sample_config() -> config::Config {
    let mut cfg = config::Config::default();
    cfg.webhooks
        .insert("alerts".into(), "https://discord/webhook/alerts".into());
    cfg.webhooks
        .insert("ops".into(), "https://discord/webhook/ops".into());
    cfg.default_webhook = Some("alerts".into());
    cfg
}

#[test]
fn webhook_url_uses_named_or_default_webhook() {
    let cfg = sample_config();

    assert_eq!(
        get_webhook_url_from_config(&cfg, Some("ops")).expect("named webhook"),
        "https://discord/webhook/ops"
    );
    assert_eq!(
        get_webhook_url_from_config(&cfg, None).expect("default webhook"),
        "https://discord/webhook/alerts"
    );
}

#[test]
fn webhook_url_errors_without_default_or_missing_name() {
    let mut cfg = sample_config();
    cfg.default_webhook = None;

    assert!(
        get_webhook_url_from_config(&cfg, None)
            .expect_err("default missing")
            .to_string()
            .contains("No default webhook")
    );
    assert!(
        get_webhook_url_from_config(&cfg, Some("missing"))
            .expect_err("webhook missing")
            .to_string()
            .contains("Webhook 'missing' not found")
    );
}

#[test]
fn add_webhook_sets_first_webhook_as_default_only_once() {
    let mut cfg = config::Config::default();

    add_webhook_to_config(&mut cfg, "first", "url1".into());
    add_webhook_to_config(&mut cfg, "second", "url2".into());

    assert_eq!(cfg.webhooks["first"], "url1");
    assert_eq!(cfg.webhooks["second"], "url2");
    assert_eq!(cfg.default_webhook.as_deref(), Some("first"));
}

#[test]
fn remove_webhook_clears_default_and_errors_when_missing() {
    let mut cfg = sample_config();

    remove_webhook_from_config(&mut cfg, "alerts").expect("remove default");

    assert!(!cfg.webhooks.contains_key("alerts"));
    assert_eq!(cfg.default_webhook, None);
    assert!(remove_webhook_from_config(&mut cfg, "missing").is_err());
}

#[test]
fn set_default_webhook_requires_existing_name() {
    let mut cfg = sample_config();

    set_default_webhook_in_config(&mut cfg, "ops").expect("set default");

    assert_eq!(cfg.default_webhook.as_deref(), Some("ops"));
    assert!(set_default_webhook_in_config(&mut cfg, "missing").is_err());
}

#[test]
fn set_bot_token_replaces_token() {
    let mut cfg = config::Config::default();

    set_bot_token_in_config(&mut cfg, "secret".into());

    assert_eq!(cfg.bot_token.as_deref(), Some("secret"));
}

#[test]
fn mask_webhook_keeps_short_urls_and_masks_long_urls() {
    assert_eq!(mask_webhook("short-url"), "short-url");

    let long = "https://discord.com/api/webhooks/1234567890/abcdefghijklmnopqrstuvwxyz";
    let masked = mask_webhook(long);

    assert!(masked.starts_with("https://discord.com/api/webhooks/1234567"));
    assert!(masked.ends_with("qrstuvwxyz"));
    assert!(masked.contains("..."));
}

#[test]
fn parse_embed_color_accepts_hash_hex_prefix_and_none() {
    const HASH_COLOR: u32 = 0x00ff10;
    const PREFIX_COLOR: u32 = 0xabcdef;
    const PREFIX_COLOR_INPUT: &str = "0xabcdef";

    assert_eq!(
        parse_embed_color(Some("#00ff10")).expect("hash"),
        Some(HASH_COLOR)
    );
    assert_eq!(
        parse_embed_color(Some(PREFIX_COLOR_INPUT)).expect("prefix"),
        Some(PREFIX_COLOR)
    );
    assert_eq!(parse_embed_color(None).expect("none"), None);
    assert!(parse_embed_color(Some("not-hex")).is_err());
}

#[test]
fn clap_definition_is_valid() {
    Cli::command().debug_assert();
}
