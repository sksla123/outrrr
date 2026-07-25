#!/bin/sh
set -e

if [ -z "$TARGET_CONTAINER" ]; then
    echo "[outrrr-entrypoint] Error: TARGET_CONTAINER environment variable is required."
    exit 1
fi

CONFIG_PATH="${CONFIG_PATH:-/etc/outrrr/config.yaml}"

# NOTIFY_URL 환경변수가 존재하면 config.yaml을 동적으로 생성
if [ -n "$NOTIFY_URL" ]; then
    echo "[outrrr-entrypoint] Generating config at $CONFIG_PATH from environment variables..."
    mkdir -p $(dirname "$CONFIG_PATH")
    
    # 환경변수가 없을 경우의 기본값 설정
    CONTEXT_LENGTH="${CONTEXT_LENGTH:-1800}"
    CONTEXT_WINDOW_DURATION="${CONTEXT_WINDOW_DURATION:-30s}"
    RATE_LIMIT_MSG_PER_MINUTE="${RATE_LIMIT_MSG_PER_MINUTE:-2}"
    COOL_DOWN_DURATION="${COOL_DOWN_DURATION:-30s}"

    cat << YAML_EOF > "$CONFIG_PATH"
notify_urls:
  - "$NOTIFY_URL"
context_length: $CONTEXT_LENGTH
context_window_duration: "$CONTEXT_WINDOW_DURATION"
rate_limit_msg_per_minute: $RATE_LIMIT_MSG_PER_MINUTE
cool_down_duration: "$COOL_DOWN_DURATION"
YAML_EOF
elif [ ! -f "$CONFIG_PATH" ]; then
    echo "[outrrr-entrypoint] Error: Config file not found at $CONFIG_PATH and NOTIFY_URL is not set."
    exit 1
fi

echo "[outrrr-entrypoint] Streaming logs from container '$TARGET_CONTAINER' using config '$CONFIG_PATH'..."

exec docker logs -f --tail 0 "$TARGET_CONTAINER" 2>&1 | /usr/local/bin/outrrr --config "$CONFIG_PATH"
