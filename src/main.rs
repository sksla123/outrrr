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

    // 2. Stdin 수집 스크립트 (재시도 루프 처리)
    let tx_stdin = tx.clone();
    let retry_count = app_config.retry_count;
    let retry_interval = durations.retry_interval;

    tokio::task::spawn_blocking(move || {
        let mut attempts = 0;

        loop {
            let stdin = io::stdin();
            let reader = stdin.lock();
            let mut read_any_line = false;

            for line in reader.lines().map_while(Result::ok) {
                read_any_line = true;
                attempts = 0; // 성공적으로 라인을 읽으면 재시도 카운트 리셋
                if tx_stdin.blocking_send(stream::LogEvent::Line(line)).is_err() {
                    return; // Consumer 채널이 닫힌 경우 스레드 종료
                }
            }

            // EOF 발생 또는 파이프가 끊어진 경우
            attempts += 1;

            if retry_count > 0 && attempts > retry_count {
                eprintln!("Max retry count ({}) reached. Exiting stdin reader...", retry_count);
                break;
            }

            let log_msg = if retry_count == 0 {
                format!("⚠️ **로그 스트림 연결이 끊겼습니다.** ({:?} 후 무한 재시도 중... [시도 횟수: {}])", retry_interval, attempts)
            } else {
                format!("⚠️ **로그 스트림 연결이 끊겼습니다.** ({:?} 후 재시도 중... [{}/{}])", retry_interval, attempts, retry_count)
            };
            let _ = tx_stdin.blocking_send(stream::LogEvent::SystemNotice(log_msg));

            std::thread::sleep(retry_interval);
        }
    });

    // 3. Flush 타이머 태스크
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

    // 4. OS 시그널 바인딩 및 소비자(Consumer) 태스크 실행
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

    // 5. 종료 알림 발송
    let _ = notifier.send("🛑 **`outrrr` 로그 스트리밍 프록시가 종료되었습니다.**").await;

    Ok(())
}
