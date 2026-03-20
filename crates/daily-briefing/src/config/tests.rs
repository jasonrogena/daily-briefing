use super::*;

fn valid_config_toml() -> &'static str {
    r#"
[[inputs]]
name = "rss_feed"
type = "fever"
url = "http://yarr.local:7070/fever/"
username_env = "user@example.com"
password_env = "secret"
hours = 24

[processor]
type = "anthropic"
api_key_env = "test_key"
model = "claude-opus-4-6"
max_tokens = 1024
prompt = "Summarize the articles."

[[outputs]]
name = "ha_summary"
type = "home_assistant"
url = "http://homeassistant.local:8123"
token_env = "test_token"
title = "Daily Summary"
notification_id = "daily_briefing"
"#
}

#[test]
fn test_parse_valid_config() {
    let config: Config = toml::from_str(valid_config_toml()).unwrap();
    assert_eq!(config.inputs.len(), 1);
    assert_eq!(config.outputs.len(), 1);

    match &config.inputs[0] {
        InputConfig::Fever(c) => {
            assert_eq!(c.name, "rss_feed");
            assert_eq!(c.url, "http://yarr.local:7070/fever/");
            assert_eq!(c.username_env, "user@example.com");
            assert_eq!(c.password_env, "secret");
            assert_eq!(c.hours, 24);
        }
        _ => panic!("expected Fever input"),
    }

    match &config.processor {
        ProcessorConfig::Anthropic(c) => {
            assert_eq!(c.model, "claude-opus-4-6");
            assert_eq!(c.max_tokens, 1024);
        }
    }

    match &config.outputs[0] {
        OutputConfig::HomeAssistant(c) => {
            assert_eq!(c.name, "ha_summary");
            assert_eq!(c.title, "Daily Summary");
            assert_eq!(c.notification_id, "daily_briefing");
        }
        _ => panic!("expected HomeAssistant output"),
    }
}

#[test]
fn test_default_hours() {
    let toml_str = r#"
[[inputs]]
name = "rss_feed"
type = "fever"
url = "http://yarr.local:7070/fever/"
username_env = "user@example.com"
password_env = "secret"

[processor]
type = "anthropic"
api_key_env = "key"
model = "claude-opus-4-6"
max_tokens = 1024
prompt = "Summarize."

[[outputs]]
name = "ha"
type = "home_assistant"
url = "http://ha.local:8123"
token_env = "token"
title = "Daily Summary"
notification_id = "daily_briefing"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    match &config.inputs[0] {
        InputConfig::Fever(c) => assert_eq!(c.hours, 24),
        _ => panic!("expected Fever input"),
    }
}

#[test]
fn test_validate_no_inputs() {
    let toml_str = r#"
inputs = []

[processor]
type = "anthropic"
api_key_env = "key"
model = "claude-opus-4-6"
max_tokens = 1024
prompt = "Summarize."

[[outputs]]
name = "ha"
type = "home_assistant"
url = "http://ha.local:8123"
token_env = "token"
title = "Daily Summary"
notification_id = "daily_briefing"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_no_outputs() {
    // outputs = [] must appear before any [section] header to be top-level in TOML
    let toml_str = r#"
outputs = []

[[inputs]]
name = "rss_feed"
type = "fever"
url = "http://yarr.local:7070/fever/"
username_env = "user@example.com"
password_env = "secret"

[processor]
type = "anthropic"
api_key_env = "key"
model = "claude-opus-4-6"
max_tokens = 1024
prompt = "Summarize."
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.validate().is_err());
}

#[test]
fn test_parse_webpage_and_speech_with_webserver() {
    let toml_str = r#"
[[inputs]]
name = "rss_feed"
type = "fever"
url = "http://yarr.local/fever/"
username_env = "u"
password_env = "p"

[processor]
type = "anthropic"
api_key_env = "key"
model = "claude-opus-4-6"
max_tokens = 512
prompt = "Brief."

[webserver]
port = 8080
username_env = "WS_USER"
password_env = "WS_PASS"

[[outputs]]
name = "page"
type = "webpage"
endpoint = "/"
title = "Daily Briefing"

[[outputs]]
name = "audio"
type = "speech"
endpoint = "/briefing.wav"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.validate().is_ok());
    let ws = config.webserver.as_ref().unwrap();
    assert_eq!(ws.port, 8080);
    match &config.outputs[0] {
        OutputConfig::Webpage(c) => {
            assert_eq!(c.endpoint, "/");
            assert_eq!(c.title, "Daily Briefing");
        }
        _ => panic!("expected Webpage output"),
    }
    match &config.outputs[1] {
        OutputConfig::Speech(c) => assert_eq!(c.endpoint, "/briefing.wav"),
        _ => panic!("expected Speech output"),
    }
}

#[test]
fn test_validate_webpage_without_webserver_fails() {
    let toml_str = r#"
[[inputs]]
name = "rss_feed"
type = "fever"
url = "http://yarr.local/fever/"
username_env = "u"
password_env = "p"

[processor]
type = "anthropic"
api_key_env = "key"
model = "claude-opus-4-6"
max_tokens = 512
prompt = "Brief."

[[outputs]]
name = "page"
type = "webpage"
endpoint = "/"
title = "Daily Briefing"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.validate().is_err());
}

#[test]
fn test_parse_unknown_input_type_fails() {
    let toml_str = r#"
[[inputs]]
name = "rss_feed"
type = "unknown_type"
url = "http://yarr.local:7070/fever/"

[processor]
type = "anthropic"
api_key_env = "key"
model = "claude-opus-4-6"
max_tokens = 1024
prompt = "Summarize."

[[outputs]]
name = "ha"
type = "home_assistant"
url = "http://ha.local:8123"
token_env = "token"
title = "Daily Summary"
notification_id = "daily_briefing"
"#;
    assert!(toml::from_str::<Config>(toml_str).is_err());
}
