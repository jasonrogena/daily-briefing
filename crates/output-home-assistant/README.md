# output-home-assistant

Posts the briefing as a [persistent notification](https://www.home-assistant.io/integrations/persistent_notification/) in Home Assistant.

## Configuration

```toml
[[outputs]]
name = "home_assistant"
type = "home_assistant"
url = "http://homeassistant.local:8123"
token_env = "HA_TOKEN"
title = "Daily Briefing"
notification_id = "daily_briefing"
```

| Field | Description |
|---|---|
| `name` | Unique name for this output, used in logs. |
| `url` | Home Assistant base URL. |
| `token_env` | Name of the env var holding a HA long-lived access token. |
| `title` | Title shown on the notification card in the HA UI. |
| `notification_id` | Notification ID. Using the same ID across runs updates the existing notification in place rather than creating a new one. |
