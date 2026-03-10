#!/usr/bin/env bash
set -euo pipefail

# 用法：
#   ./scripts/demo_api_e2e.sh [api_base_url] [size_gb] [max_monthly_budget_usd] [output_dir] [state_path]
API_BASE_URL="${1:-http://127.0.0.1:8091}"
SIZE_GB="${2:-120}"
MAX_BUDGET_USD="${3:-3000}"
OUTPUT_DIR="${4:-./demo_outputs/api_compare}"
STATE_PATH="${5:-./state.snapshot}"

ADDR="${API_BASE_URL#http://}"
ADDR="${ADDR%/}"
SERVER_STARTED_BY_SCRIPT=0
SERVER_PID=""

health_check() {
  env -u http_proxy -u https_proxy -u all_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY \
    curl -sS "${API_BASE_URL}/api/health" > /dev/null
}

cleanup() {
  if [[ "$SERVER_STARTED_BY_SCRIPT" -eq 1 && -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

if health_check; then
  echo "检测到现有 API 服务: ${API_BASE_URL}"
else
  echo "未检测到 API 服务，自动启动: ${ADDR}"
  env -u http_proxy -u https_proxy -u all_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY \
    cargo run --quiet -- serve --state "$STATE_PATH" --addr "$ADDR" > ./demo_outputs/api-server.log 2>&1 &
  SERVER_PID="$!"
  SERVER_STARTED_BY_SCRIPT=1

  ready=0
  for _ in $(seq 1 60); do
    if health_check; then
      ready=1
      break
    fi
    sleep 0.25
  done

  if [[ "$ready" -ne 1 ]]; then
    echo "API 服务启动失败，请查看日志: ./demo_outputs/api-server.log" >&2
    exit 1
  fi
fi

./scripts/demo_api_strategy_compare.sh "$API_BASE_URL" "$SIZE_GB" "$MAX_BUDGET_USD" "$OUTPUT_DIR"
echo "e2e completed: ${OUTPUT_DIR}"
