#[cfg(test)]
mod tests;

use async_trait::async_trait;
use daily_briefing_core::input::BoxError;
use daily_briefing_core::output::Output;
use webserver::{ContentEntry, WebServer};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TTS synthesis failed: {0}")]
    Tts(String),
}

pub struct SpeechConfig {
    pub name: String,
    /// Endpoint path this output is served at, e.g. `"/briefing.wav"`.
    pub endpoint: String,
    /// Path to the Piper `.onnx` voice model file.
    pub model_path: String,
    /// Speaker ID for multi-speaker models. `None` for single-speaker models.
    pub speaker: Option<u32>,
}

pub struct SpeechOutput {
    name: String,
    endpoint: String,
    model_path: String,
    speaker: Option<u32>,
    server: Arc<WebServer>,
}

impl SpeechOutput {
    pub fn new(config: SpeechConfig, server: Arc<WebServer>) -> Self {
        Self {
            name: config.name,
            endpoint: config.endpoint,
            model_path: config.model_path,
            speaker: config.speaker,
            server,
        }
    }
}

#[async_trait]
impl Output for SpeechOutput {
    fn name(&self) -> &str {
        &self.name
    }

    async fn write(&self, content: &str) -> Result<(), BoxError> {
        let wav = synthesize(content, &self.model_path, self.speaker)?;
        info!(
            output = self.name,
            bytes = wav.len(),
            "TTS synthesis complete"
        );

        let audio_path = format!("{}/audio.wav", self.endpoint.trim_end_matches('/'));
        self.server.update(
            &audio_path,
            ContentEntry {
                content_type: "audio/wav".to_string(),
                body: wav,
            },
        );

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Daily Briefing</title>
<style>body{{font-family:sans-serif;display:flex;justify-content:center;align-items:center;height:100vh;margin:0;background:#1a1a1a;color:#eee}}audio{{width:400px}}</style>
</head>
<body><audio controls autoplay src="{audio_path}?t={ts}"></audio></body>
</html>"#,
        );
        self.server.update(
            &self.endpoint,
            ContentEntry {
                content_type: "text/html; charset=utf-8".to_string(),
                body: html.into_bytes(),
            },
        );

        Ok(())
    }
}

/// Synthesise `text` to WAV bytes using Piper.
pub fn synthesize(text: &str, model_path: &str, speaker: Option<u32>) -> Result<Vec<u8>, Error> {
    let speaker_str;
    let mut args = vec!["--model", model_path, "--output_file", "/dev/stdout"];
    if let Some(id) = speaker {
        speaker_str = id.to_string();
        args.extend_from_slice(&["--speaker", &speaker_str]);
    }
    let mut child = Command::new("piper")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| Error::Tts(format!("Failed to spawn piper: {e}")))?;

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(text.as_bytes())
        .map_err(|e| Error::Tts(format!("Failed to write to espeak-ng stdin: {e}")))?;

    let out = child
        .wait_with_output()
        .map_err(|e| Error::Tts(format!("espeak-ng wait failed: {e}")))?;

    if !out.status.success() {
        return Err(Error::Tts(format!(
            "piper exited with status: {}",
            out.status
        )));
    }

    Ok(out.stdout)
}
