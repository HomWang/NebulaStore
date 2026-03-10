#!/usr/bin/env bash
set -euo pipefail

# 用法：
#   ./scripts/demo_api_strategy_compare.sh [api_base_url] [size_gb] [max_monthly_budget_usd] [output_dir]
API_BASE_URL="${1:-http://127.0.0.1:8091}"
SIZE_GB="${2:-120}"
MAX_BUDGET_USD="${3:-3000}"
OUTPUT_DIR="${4:-./demo_outputs/api_compare}"

STRATEGIES=(balanced latency-first cost-first reliability-first)
CSV_FILE="${OUTPUT_DIR}/api-champion-summary.csv"
MD_FILE="${OUTPUT_DIR}/api-champion-summary.md"

mkdir -p "$OUTPUT_DIR"

echo 'strategy,champion_template,composite_score,predicted_p95_ms,predicted_monthly_cost_usd,reliability_score,risk_level' > "$CSV_FILE"
{
  echo '| strategy | champion_template | composite_score | predicted_p95_ms | predicted_monthly_cost_usd | reliability_score | risk_level |'
  echo '| --- | --- | ---: | ---: | ---: | ---: | --- |'
} > "$MD_FILE"

# 先做健康检查，避免批量请求失败后难定位。
if ! env -u http_proxy -u https_proxy -u all_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY \
  curl -sS "${API_BASE_URL}/api/health" > /dev/null; then
  echo "API 不可用: ${API_BASE_URL}/api/health" >&2
  echo "请先启动服务，例如：cargo run -- serve --state ./state.snapshot --addr 127.0.0.1:8091" >&2
  exit 1
fi

echo "NebulaStore API strategy demo"
echo "api_base_url=${API_BASE_URL}, size_gb=${SIZE_GB}, max_monthly_budget_usd=${MAX_BUDGET_USD}"
echo

for strategy in "${STRATEGIES[@]}"; do
  json_file="${OUTPUT_DIR}/api-champion-${strategy}.json"

  payload="{\"size_gb\":${SIZE_GB},\"max_monthly_budget_usd\":${MAX_BUDGET_USD},\"strategy\":\"${strategy}\"}"

  env -u http_proxy -u https_proxy -u all_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY \
    curl -sS -X POST "${API_BASE_URL}/api/protocol/template/champion" \
      -H 'Content-Type: application/json' \
      -d "$payload" > "$json_file"

  # 先截取 ranking 的首个对象（冠军指标），再逐字段提取。
  champion_template=$(grep -oE '"champion_template":"[^"]+"' "$json_file" | head -n 1 | sed -E 's/.*"champion_template":"([^"]+)".*/\1/')
  top_rank_item=$(grep -oE '"ranking":\[\{[^}]*\}' "$json_file" | head -n 1)
  composite_score=$(echo "$top_rank_item" | grep -oE '"composite_score":[0-9.]+' | head -n 1 | sed -E 's/.*:([0-9.]+)/\1/')
  predicted_p95_ms=$(echo "$top_rank_item" | grep -oE '"predicted_p95_ms":[0-9]+' | head -n 1 | sed -E 's/.*:([0-9]+)/\1/')
  predicted_monthly_cost_usd=$(echo "$top_rank_item" | grep -oE '"predicted_monthly_cost_usd":[0-9.]+' | head -n 1 | sed -E 's/.*:([0-9.]+)/\1/')
  reliability_score=$(echo "$top_rank_item" | grep -oE '"reliability_score":[0-9]+' | head -n 1 | sed -E 's/.*:([0-9]+)/\1/')
  risk_level=$(echo "$top_rank_item" | grep -oE '"risk_level":"[^"]+"' | head -n 1 | sed -E 's/.*"risk_level":"([^"]+)".*/\1/')

  echo "${strategy} -> ${champion_template}"
  echo "${strategy},${champion_template},${composite_score},${predicted_p95_ms},${predicted_monthly_cost_usd},${reliability_score},${risk_level}" >> "$CSV_FILE"
  echo "| ${strategy} | ${champion_template} | ${composite_score} | ${predicted_p95_ms} | ${predicted_monthly_cost_usd} | ${reliability_score} | ${risk_level} |" >> "$MD_FILE"
done

echo
echo "api reports saved in: ${OUTPUT_DIR}"
echo "summary csv: ${CSV_FILE}"
echo "summary markdown: ${MD_FILE}"