# input-fever

Fetches unread articles from a [Fever-compatible](https://github.com/DigitalDJ/tinytinyrss-fever-plugin/blob/master/fever-api.md) RSS aggregator such as [yarr](https://github.com/nkanaev/yarr).

## Configuration

```toml
[[inputs]]
name = "rss_feed"
type = "fever"
url = "http://yarr.local:7070/fever/"
username_env = "FEVER_USERNAME"
password_env = "FEVER_PASSWORD"
hours = 24
mark_as_read = true
```

| Field | Default | Description |
|---|---|---|
| `name` | — | Unique name for this input, used in logs. |
| `url` | — | Base URL of the Fever endpoint. |
| `username_env` | — | Name of the env var holding the Fever username. |
| `password_env` | — | Name of the env var holding the Fever password. Authentication uses an MD5 hash of `username:password` as required by the Fever API. |
| `hours` | `24` | How far back to fetch items. Items older than `now - hours` are ignored. |
| `mark_as_read` | `false` | If `true`, fetched items are marked as read in the aggregator after collection. |
