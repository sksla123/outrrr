# outrrr

> A robust, rate-limited, and context-windowed stdout-to-Discord log streaming proxy powered by Rust and Shoutrrr.

`outrrr`는 로컬 명령어, 백그라운드 스크립트 또는 Docker 컨테이너의 실시간 `stdout`/`stderr`를 디스코드 웹후크로 안전하게 전달하는 로그 프록시입니다.

과도한 로그 전송으로 인한 Discord API Rate Limit을 방지하기 위해 **Context Window**, **Rate Limiting**, **Cooldown** 기능을 제공합니다.

---

## Features

- 실시간 stdout/stderr 스트리밍
- Context Window 기반 버퍼링
- Discord Rate Limit 보호
- 자동 Cooldown
- Docker Socket Proxy 지원
- 시작/종료 알림 전송
- Shoutrrr 기반 Notification

---

## Quick Start

### docker-compose.yml

~yaml
services:
  outrrr:
    build: .
    environment:
      TARGET_CONTAINER: my-backend-app
      DOCKER_HOST: tcp://docker-proxy:2375
      NOTIFY_URL: discord://your_webhook_id@your_webhook_token

      CONTEXT_LENGTH: 1800
      CONTEXT_WINDOW_DURATION: 30s

      RATE_LIMIT_MSG_PER_MINUTE: 2
      COOL_DOWN_DURATION: 30s
~

빌드 및 실행

~bash
docker compose up --build -d
~

---

## Configuration

| Variable | Default | Description |
|-----------|----------|-------------|
| `NOTIFY_URL` | Required | Discord webhook URL |
| `TARGET_CONTAINER` | Required | Target container name or ID |
| `DOCKER_HOST` | `unix:///var/run/docker.sock` | Docker endpoint |
| `CONTEXT_LENGTH` | `1800` | Flush when accumulated text exceeds this size |
| `CONTEXT_WINDOW_DURATION` | `30s` | Flush after this idle window |
| `RATE_LIMIT_MSG_PER_MINUTE` | `2` | Maximum Discord messages per minute |
| `COOL_DOWN_DURATION` | `30s` | Drop logs during cooldown after rate limit |

---

## How It Works

1. `docker logs -f` 또는 파이프로 전달된 stdout을 읽습니다.
2. 로그를 Context Window에 누적합니다.
3. 아래 조건 중 하나를 만족하면 Discord로 전송합니다.

- 버퍼 길이 ≥ `CONTEXT_LENGTH`
- 버퍼가 비어있지 않은 상태에서 `CONTEXT_WINDOW_DURATION` 경과

4. 분당 전송 횟수가 제한을 초과하면

- 경고 메시지 전송
- `COOL_DOWN_DURATION` 동안 로그 폐기
- 이후 자동 복구

---

## Project Structure

~text
outrrr/
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── entrypoint.sh
└── src
    ├── cli.rs
    ├── config.rs
    ├── main.rs
    ├── notify.rs
    └── stream.rs
~

---

## Architecture

~text
docker logs -f
        │
        ▼
+------------------+
|     outrrr       |
|------------------|
| Context Window   |
| Rate Limiter     |
| Cooldown         |
+------------------+
        │
        ▼
   Shoutrrr
        │
        ▼
 Discord Webhook
~

---

## Example

~bash
docker logs -f my-container | outrrr
~

또는

~bash
python server.py 2>&1 | outrrr
~

---

## License

MIT


