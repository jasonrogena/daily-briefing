# daily-briefing

A Rust CLI tool that collects data from inputs, processes via AI, and writes to outputs. The MVP fetches RSS articles via a yarr (Fever API) instance, summarises them with Anthropic, and writes the result to a Home Assistant `input_text` entity.

## Workspace layout

```
crates/
  daily-briefing/          ← binary + config + runner
  daily-briefing-core/     ← Input/Processor/Output traits, InputData, BoxError
  input-fever/            ← Fever API (yarr) input
  processor-anthropic/    ← Anthropic Messages API processor
  output-home-assistant/  ← Home Assistant input_text output
```

## Plugin pattern

- Traits in `daily-briefing-core`: `Input`, `Processor`, `Output`
- Config uses `#[serde(tag = "type", rename_all = "snake_case")]` on enums — each variant wraps a fully independent struct, so different plugin types have completely different config schemas
- New plugin = new enum variant in `config/mod.rs` + new config struct + new crate

## CLI

```
daily-briefing run      --config <path>
daily-briefing validate --config <path>
```

## Key notes

- HA `input_text` entities cap at 100 chars by default; user must set `max: 255` in HA `configuration.yaml`
- Fever auth: MD5 hash of `username:password`
- All network I/O is async (tokio + reqwest)
