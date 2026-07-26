// src/stream.rs
use crate::config::{AppConfig, ParsedDurations};
use crate::notify::Notifier;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use tokio::time::Instant;

pub enum LogEvent {
    Line(String),
    FlushTrigger,
    SystemNotice(String),
}

pub async fn message_consumer(
    mut rx: mpsc::Receiver<LogEvent>,
    config: AppConfig,
    durations: ParsedDurations,
    notifier: &Notifier,
) -> Result<()> {
    let mut context_buffer = String::new();
    let mut last_flush_time = Instant::now();
    let mut is_window_active = false;

    let mut msg_timestamps: Vec<DateTime<Utc>> = Vec::new();
    let mut cooldown_until: Option<Instant> = None;
    let mut initial_message_sent = false;

    while let Some(event) = rx.recv().await {
        if let Some(until) = cooldown_until {
            if Instant::now() < until {
                if let LogEvent::Line(_) = event {
                    continue;
                }
            } else {
                cooldown_until = None;
            }
        }

        match event {
            LogEvent::Line(line) => {
                if !is_window_active {
                    last_flush_time = Instant::now();
                    is_window_active = true;
                }

                context_buffer.push_str(&line);
                context_buffer.push('\n');

                if context_buffer.len() >= config.context_length {
                    flush_buffer(
                        &mut context_buffer,
                        notifier,
                        &mut msg_timestamps,
                        &config,
                        &durations,
                        &mut cooldown_until,
                        &mut initial_message_sent,
                    )
                    .await?;
                    is_window_active = false;
                }
            }
            LogEvent::FlushTrigger => {
                if is_window_active && last_flush_time.elapsed() >= durations.context_window {
                    if !context_buffer.is_empty() {
                        flush_buffer(
                            &mut context_buffer,
                            notifier,
                            &mut msg_timestamps,
                            &config,
                            &durations,
                            &mut cooldown_until,
                            &mut initial_message_sent,
                        )
                        .await?;
                    }
                    is_window_active = false;
                }
            }
            LogEvent::SystemNotice(notice) => {
                // 스트림 연결 상태가 변경된 경우 기존 버퍼가 있다면 우선 플러시
                if !context_buffer.is_empty() {
                    let _ = notifier.send(&format!("```\n{}\n```", context_buffer)).await;
                    context_buffer.clear();
                    is_window_active = false;
                }
                let _ = notifier.send(&notice).await;
            }
        }
    }

    // 루프가 종료될 때 잔여 버퍼 플러시
    if !context_buffer.is_empty() {
        let _ = notifier.send(&format!("```\n{}\n```", context_buffer)).await;
    }

    Ok(())
}

async fn flush_buffer(
    buffer: &mut String,
    notifier: &Notifier,
    timestamps: &mut Vec<DateTime<Utc>>,
    config: &AppConfig,
    durations: &ParsedDurations,
    cooldown_until: &mut Option<Instant>,
    initial_message_sent: &mut bool,
) -> Result<()> {
    let now = Utc::now();
    timestamps.retain(|&ts| now.signed_duration_since(ts).num_seconds() < 60);

    if *initial_message_sent && timestamps.len() >= config.rate_limit_msg_per_minute {
        let warning = "```diff\n- [WARNING]: Too many texts. Cooldown activated.\n```";
        let _ = notifier.send(warning).await;

        *cooldown_until = Some(Instant::now() + durations.cool_down);
        buffer.clear();
        return Ok(());
    }

    let payload = format!("```\n{}\n```", buffer);
    notifier.send(&payload).await.context("outrrr failed to dispatch shoutrrr notification")?;

    timestamps.push(now);
    *initial_message_sent = true;
    buffer.clear();

    Ok(())
}
