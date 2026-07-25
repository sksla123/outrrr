// src/main.rs
use anyhow::Result;
use clap::Parser;
use std::io::{self, BufRead};
use tokio::sync::mpsc;
use tokio::time::sleep;

mod cli;
mod config;
mod notify;
mod stream;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();
    
    let (app_config, durations) = config::load_config(&args.config)?;
    let notifier = notify::Notifier::new(&app_config.notify_urls)?;
    
    // 1. 시작 알림 발송
    let _ = notifier.send("🟢 **`outrrr` 로그 스트리밍 프록시가 시작되었습니다.**").await;

    let (tx, rx) = mpsc::channel::<stream::LogEvent>(100);

    let tx_stdin = tx.clone();
    tokio::task::spawn_blocking(move || {
        let stdin = io::stdin();
        let reader = stdin.lock();
        for line in reader.lines().map_while(Result::ok) {
            let _ = tx_stdin.blocking_send(stream::LogEvent::Line(line));
        }
    });

    let tx_timer = tx.clone();
    let timer_duration = durations.context_window;
    tokio::spawn(async move {
        loop {
            sleep(timer_duration).await;
            if tx_timer.send(stream::LogEvent::FlushTrigger).await.is_err() {
                break;
            }
        }
    });

    // 2. OS 시그널 바인딩 및 소비자(Consumer) 태스크 실행
    #[cfg(unix)]
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    #[cfg(unix)]
    tokio::select! {
        res = stream::message_consumer(rx, app_config, durations, &notifier) => {
            if let Err(e) = res {
                eprintln!("Consumer error: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down...");
        }
        _ = sigterm.recv() => {
            println!("Received SIGTERM, shutting down...");
        }
    }

    #[cfg(not(unix))]
    tokio::select! {
        res = stream::message_consumer(rx, app_config, durations, &notifier) => {
            if let Err(e) = res {
                eprintln!("Consumer error: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down...");
        }
    }

    // 3. 종료 알림 발송 (정상 파이프 종료 및 강제 시그널 종료 시 모두 동작)
    let _ = notifier.send("🛑 **`outrrr` 로그 스트리밍 프록시가 종료되었습니다.**").await;

    Ok(())
}
