# output-speech

Synthesises the briefing to audio using [Piper](https://github.com/rhasspy/piper) neural TTS and serves it via the shared web server.

- Navigating to `endpoint` opens an in-browser audio player.
- The raw WAV file is available at `{endpoint}/audio.wav`.

Requires a `[webserver]` section in the config.

## Setup

Download a voice model (`.onnx` + `.onnx.json` pair) from the [piper-voices repository](https://huggingface.co/rhasspy/piper-voices) and mount it into the container. Both files must be present alongside each other.

## Configuration

```toml
[[outputs]]
name = "speech"
type = "speech"
endpoint = "/speech"
model = "/etc/daily-briefing/voices/en_US-lessac-medium.onnx"
# speaker = 0   # required for multi-speaker models only
```

| Field | Default | Description |
|---|---|---|
| `name` | — | Unique name for this output, used in logs. |
| `endpoint` | — | URL path for the audio player page (e.g. `"/speech"`). The raw WAV is served at `{endpoint}/audio.wav`. |
| `model` | — | Path to the Piper `.onnx` voice model file. The `.onnx.json` config must exist alongside it with the same base name. |
| `speaker` | — | Speaker ID for multi-speaker voice models. Omit for single-speaker models. Check the model's `.onnx.json` for available speaker IDs. |

## Single-speaker vs multi-speaker models

Most models are single-speaker — no `speaker` field needed. Multi-speaker models (e.g. `en_GB-vctk-medium`) contain many voices and require `speaker = <id>`. The available IDs are listed in the `speaker_id_map` of the `.onnx.json` file.

If using a multi-speaker model, set `"num_speakers": 1` in the `.onnx.json` if you downloaded a single-speaker export, or specify a valid `speaker` ID if you have the full multi-speaker model.
