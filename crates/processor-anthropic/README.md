# processor-anthropic

Summarises collected input using the [Anthropic Messages API](https://docs.anthropic.com/en/api/messages).

## Configuration

```toml
[processor]
type = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-opus-4-6"
max_tokens = 1024
prompt = "Summarize the following into a concise daily briefing, readable as a podcast."
```

| Field | Description |
|---|---|
| `api_key_env` | Name of the env var holding the Anthropic API key. |
| `model` | Model ID to use (e.g. `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-haiku-4-5-20251001`). |
| `max_tokens` | Maximum number of tokens in the response. |
| `prompt` | System prompt prepended to the collected input. Controls tone, format, and length of the output. |
