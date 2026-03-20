//! Integration tests for config loading from real files.

use std::io::Write;
use tempfile::NamedTempFile;

fn write_temp_config(contents: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create temp file");
    f.write_all(contents.as_bytes()).expect("write temp file");
    f
}

fn valid_toml() -> &'static str {
    r#"
[[inputs]]
name = "rss_feed"
type = "fever"
url = "http://yarr.local:7070/fever/"
username_env = "user@example.com"
password_env = "secret"
hours = 12

[processor]
type = "anthropic"
api_key_env = "test_key"
model = "claude-opus-4-6"
max_tokens = 512
prompt = "Summarize."

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
fn test_config_loads_from_file() {
    let f = write_temp_config(valid_toml());
    let path = f.path().to_str().unwrap().to_string();
    let config = daily_briefing::config::Config::new(&path).expect("config should load");
    assert_eq!(config.inputs.len(), 1);
    assert_eq!(config.outputs.len(), 1);
}

#[test]
fn test_config_missing_file_returns_error() {
    let result = daily_briefing::config::Config::new("/nonexistent/path/config.toml");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Failed to read config file"));
}

#[test]
fn test_config_invalid_toml_returns_error() {
    let f = write_temp_config("this is not valid toml ][[[");
    let path = f.path().to_str().unwrap().to_string();
    let result = daily_briefing::config::Config::new(&path);
    assert!(result.is_err());
}

#[test]
fn test_config_multiple_inputs_and_outputs() {
    let toml = r#"
[[inputs]]
name = "feed1"
type = "fever"
url = "http://yarr.local/fever/"
username_env = "u1"
password_env = "p1"

[[inputs]]
name = "feed2"
type = "fever"
url = "http://yarr2.local/fever/"
username_env = "u2"
password_env = "p2"
hours = 6

[processor]
type = "anthropic"
api_key_env = "key"
model = "claude-opus-4-6"
max_tokens = 256
prompt = "Brief."

[[outputs]]
name = "out1"
type = "home_assistant"
url = "http://ha1.local:8123"
token_env = "tok1"
title = "Daily Summary"
notification_id = "daily_briefing_1"

[[outputs]]
name = "out2"
type = "home_assistant"
url = "http://ha2.local:8123"
token_env = "tok2"
title = "Daily Summary"
notification_id = "daily_briefing_2"
"#;
    let f = write_temp_config(toml);
    let path = f.path().to_str().unwrap().to_string();
    let config = daily_briefing::config::Config::new(&path).expect("config should load");
    assert_eq!(config.inputs.len(), 2);
    assert_eq!(config.outputs.len(), 2);
}

#[test]
fn test_config_empty_inputs_fails_validation() {
    let toml = r#"
inputs = []

[processor]
type = "anthropic"
api_key_env = "key"
model = "claude-opus-4-6"
max_tokens = 256
prompt = "Brief."

[[outputs]]
name = "out1"
type = "home_assistant"
url = "http://ha1.local:8123"
token_env = "tok1"
title = "Daily Summary"
notification_id = "daily_briefing"
"#;
    let f = write_temp_config(toml);
    let path = f.path().to_str().unwrap().to_string();
    let result = daily_briefing::config::Config::new(&path);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("input"),
        "error should mention inputs, got: {}",
        msg
    );
}
