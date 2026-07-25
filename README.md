# outrrr

> **A lightweight, rate-limited stdout/stderr log streaming proxy for Discord, powered by Rust and Shoutrrr.**

`outrrr` captures real-time logs from Docker containers or any command-line application and forwards them to Discord via webhooks. It protects against Discord API rate limits using **context-window buffering**, **message rate limiting**, and an **automatic cooldown mechanism**, making it suitable for long-running services and production environments.

---

## Features

* **Real-time log streaming**

  * Streams `stdout` and `stderr` directly to Discord.

* **Context Window buffering**

  * Buffers logs until either:

    * the accumulated text reaches a configured size, or
    * a configurable timeout expires.

* **Discord Rate Limit protection**

  * Prevents excessive webhook requests.

* **Automatic Cooldown**

  * Drops incoming logs temporarily when the configured rate limit is exceeded.

* **Docker-friendly**

  * Supports monitoring Docker containers through a Docker Socket Proxy.

* **Lifecycle notifications**

  * Sends startup and shutdown notifications automatically.

---

## Architecture

```text
                  docker logs -f
                        │
                        ▼
                +----------------+
                |    outrrr      |
                |----------------|
                | Context Window |
                | Rate Limiter   |
                | Cooldown       |
                +----------------+
                        │
                        ▼
                   Shoutrrr
                        │
                        ▼
               Discord Webhook
```

---

## Quick Start

### Requirements

* Docker
* Docker Compose

---

### Configuration

```yaml
services:
  outrrr:
    build: .

    environment:
      TARGET_CONTAINER: my-backend-app
      DOCKER_HOST: tcp://docker-proxy:2375
      NOTIFY_URL: discord://YOUR_WEBHOOK_ID@YOUR_WEBHOOK_TOKEN

      CONTEXT_LENGTH: 1800
      CONTEXT_WINDOW_DURATION: 30s

      RATE_LIMIT_MSG_PER_MINUTE: 2
      COOL_DOWN_DURATION: 30s
```

---

### Build

```bash
docker compose up --build -d
```

---

## Configuration Reference

| Variable                    | Required |            Default            | Description                                  |
| --------------------------- | :------: | :---------------------------: | -------------------------------------------- |
| `NOTIFY_URL`                |     ✅    |               -               | Discord webhook URL                          |
| `TARGET_CONTAINER`          |     ✅    |               -               | Docker container to monitor                  |
| `DOCKER_HOST`               |     ❌    | `unix:///var/run/docker.sock` | Docker endpoint                              |
| `CONTEXT_LENGTH`            |     ❌    |             `1800`            | Flush buffer after this many characters      |
| `CONTEXT_WINDOW_DURATION`   |     ❌    |             `30s`             | Flush after timeout if buffer is not empty   |
| `RATE_LIMIT_MSG_PER_MINUTE` |     ❌    |              `2`              | Maximum Discord messages per minute          |
| `COOL_DOWN_DURATION`        |     ❌    |             `30s`             | Time spent dropping logs after rate limiting |

---

## How It Works

1. Reads log output from `stdin` or `docker logs -f`.
2. Buffers incoming logs in a context window.
3. Flushes the buffer when **either**:

   * the configured buffer size is reached, or
   * the configured timeout expires.
4. Sends the buffered logs to Discord.
5. If the configured message rate is exceeded:

   * sends a warning,
   * enters cooldown mode,
   * drops logs until cooldown ends.

---

## Examples

### Docker

```bash
docker logs -f my-container | outrrr
```

### Any CLI application

```bash
python server.py 2>&1 | outrrr
```

### Shell script

```bash
./backup.sh 2>&1 | outrrr
```

---

## Project Structure

```text
outrrr
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
```

---

## Why outrrr?

Unlike simple webhook loggers, **outrrr** is designed to operate safely in production by reducing webhook traffic while preserving useful logging context.

Its buffering strategy minimizes API requests without sacrificing log readability, and the built-in cooldown mechanism prevents runaway logging from overwhelming Discord.

---

## Built With

* Rust
* Tokio
* Shoutrrr
* Docker
* Docker Socket Proxy

---

## License

MIT

