# daily-briefing

A Rust CLI tool that collects data from multiple inputs, processes it with an AI model, and publishes the result to one or more outputs.

## Features

**Inputs** — [Fever RSS](crates/input-fever/README.md) · [Home Assistant metrics](crates/input-home-assistant-metrics/README.md)

**Processors** — [Anthropic Claude](crates/processor-anthropic/README.md)

**Outputs** — [Home Assistant notification](crates/output-home-assistant/README.md) · [Web page](crates/output-webpage/README.md) · [Speech (Piper TTS)](crates/output-speech/README.md)

- Run once or on a cron schedule
- Multiple inputs collected in parallel
- All secrets passed via environment variables — nothing sensitive in the config file
- Single shared web server with basic auth for all web-based outputs

## Installation

### Build from source

```bash
cargo build --release
cp target/release/daily-briefing /usr/local/bin/
```

### Container

```bash
podman build -t localhost/daily-briefing:latest .
```

## Configuration

Copy `config.example.toml` and edit to suit:

```bash
cp config.example.toml config.toml
$EDITOR config.toml
```

The config file holds only non-secret values. Secrets are read from environment variables whose **names** are specified in the config.

### Example

```toml
[[inputs]]
name = "home_health"
type = "home_assistant_metrics"
url = "http://homeassistant.local:8123"
token_env = "HA_TOKEN"
hours = 24

[[inputs.entities]]
id = "sensor.grid_consumption_energy"
label = "Grid Consumption"

[[inputs.entities]]
id = "sensor.solar_production_energy"
label = "Solar Production"

[[inputs.entities]]
id = "sensor.hue_motion_sensor_1_battery"
label = "Motion Sensor 1 Battery"
kind = "gauge"

[[inputs]]
name = "rss_feed"
type = "fever"
url = "http://yarr.local:7070/fever/"
username_env = "FEVER_USERNAME"
password_env = "FEVER_PASSWORD"
hours = 24
mark_as_read = true

[processor]
type = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-opus-4-6"
max_tokens = 1024
prompt = "Summarize the following into a concise daily briefing, readable as a podcast."

# Remove to run once and exit.
[schedule]
cron = "0 8 * * *"

# Required for webpage and speech outputs.
[webserver]
host = "0.0.0.0"
port = 8080
username_env = "BRIEFING_USERNAME"
password_env = "BRIEFING_PASSWORD"

[[outputs]]
name = "home_assistant"
type = "home_assistant"
url = "http://homeassistant.local:8123"
token_env = "HA_TOKEN"
title = "Daily Briefing"
notification_id = "daily_briefing"

[[outputs]]
name = "webpage"
type = "webpage"
endpoint = "/"
title = "Daily Briefing"

[[outputs]]
name = "speech"
type = "speech"
endpoint = "/speech"
model = "/etc/daily-briefing/voices/en_US-lessac-medium.onnx"
```

See [`config.example.toml`](config.example.toml) for an annotated version.

## Usage

```bash
# Validate config (no network calls)
daily-briefing validate --config config.toml

# Run the pipeline once (or as a daemon if [schedule] is configured)
daily-briefing run --config config.toml
```

Set `RUST_LOG=debug` for detailed logs including HTTP requests.

## Running with Podman

```bash
podman run --env-file daily-briefing.env \
  -v /path/to/config.toml:/etc/daily-briefing/config.toml:ro \
  -v /path/to/voices:/etc/daily-briefing/voices:ro \
  -p 8080:8080 \
  localhost/daily-briefing:latest
```

### Podman Quadlet (systemd)

A ready-made Quadlet unit file is provided at `daily-briefing.container`. Place it in `/etc/containers/systemd/` and run:

```bash
systemctl daemon-reload
systemctl start systemd-daily-briefing
```

## Architecture

```
crates/
  daily-briefing/               ← binary, config parsing, pipeline runner
  daily-briefing-core/          ← Input / Processor / Output traits, InputData
  input-fever/                  ← Fever API input
  input-home-assistant-metrics/ ← Home Assistant sensor metrics input
  processor-anthropic/          ← Anthropic Messages API processor
  output-home-assistant/        ← Home Assistant persistent notification output
  output-webpage/               ← Rendered HTML page output
  output-speech/                ← Piper TTS audio output
  webserver/                    ← Shared Axum web server with basic auth
```

New inputs, processors, or outputs can be added as new crates implementing the relevant trait from `daily-briefing-core`.

## License

MIT
