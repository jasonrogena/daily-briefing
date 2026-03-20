use crate::config::{Config, InputConfig, OutputConfig, ProcessorConfig};
use chrono::Utc;
use cron::Schedule;
use daily_briefing_core::input::{BoxError, Input, InputData};
use daily_briefing_core::output::Output;
use daily_briefing_core::processor::Processor;
use futures::future::join_all;
use input_fever::{FeverConfig, FeverInput};
use input_home_assistant_metrics::{EntityConfig as HaMetricsEntityConfig, EntityKind as HaMetricsEntityKind, HaMetricsConfig, HaMetricsInput};
use output_home_assistant::{HomeAssistantConfig, HomeAssistantOutput};
use output_speech::{SpeechConfig, SpeechOutput};
use output_webpage::{WebpageConfig, WebpageOutput};
use processor_anthropic::{AnthropicConfig, AnthropicProcessor};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use webserver::{WebServer, WebServerConfig};

pub async fn run(config: Config) -> Result<(), BoxError> {
    let schedule = config
        .schedule
        .as_ref()
        .map(|s| parse_cron(&s.cron))
        .transpose()?;

    // Create shared web server if configured. It starts immediately and serves
    // whatever content outputs register with it.
    let web_server: Option<Arc<WebServer>> = if let Some(wsc) = config.webserver {
        let username = resolve_env(&wsc.username_env, "webserver", "webserver")?;
        let password = resolve_env(&wsc.password_env, "webserver", "webserver")?;
        Some(Arc::new(WebServer::new(WebServerConfig {
            host: wsc.host,
            port: wsc.port,
            username,
            password,
        })))
    } else {
        None
    };

    let has_webserver = web_server.is_some();

    let mut inputs: Vec<Box<dyn Input>> = Vec::new();
    for c in config.inputs {
        let input: Box<dyn Input> = match c {
            InputConfig::Fever(fc) => {
                let username = resolve_env(&fc.username_env, "Fever input", &fc.name)?;
                let password = resolve_env(&fc.password_env, "Fever input", &fc.name)?;
                Box::new(FeverInput::new(FeverConfig {
                    name: fc.name,
                    url: fc.url,
                    username,
                    password,
                    hours: fc.hours,
                    mark_as_read: fc.mark_as_read,
                }))
            }
            InputConfig::HomeAssistantMetrics(hae) => {
                let token = resolve_env(&hae.token_env, "Home Assistant metrics input", &hae.name)?;
                Box::new(HaMetricsInput::new(HaMetricsConfig {
                    name: hae.name,
                    url: hae.url,
                    token,
                    hours: hae.hours,
                    entities: hae
                        .entities
                        .into_iter()
                        .map(|e| HaMetricsEntityConfig {
                            id: e.id,
                            label: e.label,
                            kind: match e.kind {
                                crate::config::EntityKind::Cumulative => HaMetricsEntityKind::Cumulative,
                                crate::config::EntityKind::Gauge => HaMetricsEntityKind::Gauge,
                            },
                        })
                        .collect(),
                }))
            }
        };
        inputs.push(input);
    }

    let processor: Box<dyn Processor> = match config.processor {
        ProcessorConfig::Anthropic(ac) => {
            let api_key = resolve_env(&ac.api_key_env, "Anthropic processor", "anthropic")?;
            Box::new(AnthropicProcessor::new(AnthropicConfig {
                api_key,
                model: ac.model,
                max_tokens: ac.max_tokens,
                prompt: ac.prompt,
            }))
        }
    };

    // Outputs are built once and live for the duration of the process.
    let mut outputs: Vec<Box<dyn Output>> = Vec::new();
    for c in config.outputs {
        let output: Box<dyn Output> = match c {
            OutputConfig::HomeAssistant(hac) => {
                let token = resolve_env(&hac.token_env, "Home Assistant output", &hac.name)?;
                Box::new(HomeAssistantOutput::new(HomeAssistantConfig {
                    name: hac.name,
                    url: hac.url,
                    token,
                    title: hac.title,
                    notification_id: hac.notification_id,
                }))
            }
            OutputConfig::Webpage(wc) => {
                let server = web_server
                    .as_ref()
                    .expect("validated: [webserver] required for webpage output");
                Box::new(WebpageOutput::new(
                    WebpageConfig {
                        name: wc.name,
                        endpoint: wc.endpoint,
                        title: wc.title,
                    },
                    Arc::clone(server),
                ))
            }
            OutputConfig::Speech(sc) => {
                let server = web_server
                    .as_ref()
                    .expect("validated: [webserver] required for speech output");
                Box::new(SpeechOutput::new(
                    SpeechConfig {
                        name: sc.name,
                        endpoint: sc.endpoint,
                        model_path: sc.model,
                        speaker: sc.speaker,
                    },
                    Arc::clone(server),
                ))
            }
        };
        outputs.push(output);
    }

    loop {
        run_pipeline(&inputs, &processor, &outputs).await?;

        match &schedule {
            Some(sched) => {
                let sleep = next_sleep(sched)?;
                tokio::select! {
                    _ = tokio::time::sleep(sleep) => {}
                    _ = tokio::signal::ctrl_c() => {
                        info!("Received Ctrl-C, shutting down");
                        break;
                    }
                }
            }
            None => {
                if has_webserver {
                    info!("Pipeline complete. Web server is running. Press Ctrl-C to stop.");
                    tokio::signal::ctrl_c().await.ok();
                }
                break;
            }
        }
    }

    Ok(())
}

async fn run_pipeline(
    inputs: &[Box<dyn Input>],
    processor: &Box<dyn Processor>,
    outputs: &[Box<dyn Output>],
) -> Result<(), BoxError> {
    info!("Collecting {} input(s)...", inputs.len());
    let collect_results = join_all(inputs.iter().map(|i| i.collect())).await;

    let mut input_data: Vec<InputData> = Vec::new();
    let mut collect_errors: Vec<String> = Vec::new();

    for (input, result) in inputs.iter().zip(collect_results) {
        match result {
            Ok(data) => {
                info!(
                    input = input.name(),
                    chars = data.content.len(),
                    "Collected input"
                );
                input_data.push(data);
            }
            Err(e) => {
                error!(input = input.name(), "Failed to collect input: {}", e);
                collect_errors.push(format!("{}: {}", input.name(), e));
            }
        }
    }

    if !collect_errors.is_empty() && input_data.is_empty() {
        return Err(format!("All inputs failed: {}", collect_errors.join("; ")).into());
    } else if !collect_errors.is_empty() {
        warn!(
            "{} input(s) failed, continuing with partial data",
            collect_errors.len()
        );
    }

    info!("Sending {} input(s) to processor...", input_data.len());
    let result = processor.process(&input_data).await?;
    info!(chars = result.len(), "Processor returned result");

    info!("Writing to {} output(s)...", outputs.len());
    let write_results = join_all(outputs.iter().map(|o| o.write(&result))).await;

    let mut write_errors: Vec<String> = Vec::new();
    for (output, res) in outputs.iter().zip(write_results) {
        match res {
            Ok(()) => info!(output = output.name(), "Output written successfully"),
            Err(e) => {
                error!(output = output.name(), "Failed to write output: {}", e);
                write_errors.push(format!("{}: {}", output.name(), e));
            }
        }
    }

    if !write_errors.is_empty() {
        return Err(format!("Some outputs failed: {}", write_errors.join("; ")).into());
    }

    info!("Pipeline run complete");
    Ok(())
}

fn resolve_env(var: &str, kind: &str, name: &str) -> Result<String, BoxError> {
    env::var(var).map_err(|_| {
        format!(
            "{} '{}': environment variable '{}' is not set",
            kind, name, var
        )
        .into()
    })
}

fn parse_cron(expr: &str) -> Result<Schedule, BoxError> {
    let extended = format!("0 {} *", expr);
    Schedule::from_str(&extended)
        .map_err(|e| format!("Invalid cron expression '{}': {}", expr, e).into())
}

fn next_sleep(schedule: &Schedule) -> Result<Duration, BoxError> {
    let next = schedule
        .upcoming(Utc)
        .next()
        .ok_or("Cron schedule produced no future times")?;
    let now = Utc::now();
    let duration = (next - now).to_std().unwrap_or_default();
    info!(
        "Next run at {} (in {})",
        next.format("%Y-%m-%d %H:%M:%S UTC"),
        format_duration(duration)
    );
    Ok(duration)
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}
