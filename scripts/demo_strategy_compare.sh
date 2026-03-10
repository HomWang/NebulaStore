#!/usr/bin/env bash
set -euo pipefail

# 用法：
#   ./scripts/demo_strategy_compare.sh [size_gb] [max_monthly_budget_usd] [output_dir]
SIZE_GB="${1:-120}"
MAX_BUDGET_USD="${2:-3000}"
OUTPUT_DIR="${3:-./demo_outputs}"

STRATEGIES=(balanced latency-first cost-first reliability-first)
CSV_FILE="${OUTPUT_DIR}/champion-summary.csv"
MD_FILE="${OUTPUT_DIR}/champion-summary.md"
mkdir -p "$OUTPUT_DIR"

echo 'strategy,champion_template,composite_score,predicted_p95_ms,predicted_monthly_cost_usd,reliability_score,risk_level' > "$CSV_FILE"
{
  echo '| strategy | champion_template | composite_score | predicted_p95_ms | predicted_monthly_cost_usd | reliability_score | risk_level |'
  echo '| --- | --- | ---: | ---: | ---: | ---: | --- |'
} > "$MD_FILE"

echo "NebulaStore champion strategy demo"
echo "size_gb=${SIZE_GB}, max_monthly_budget_usd=${MAX_BUDGET_USD}"
echo

for strategy in "${STRATEGIES[@]}"; do
  json_file="${OUTPUT_DIR}/champion-${strategy}.json"

  cargo run --quiet -- protocol-template-champion \
    --size-gb "$SIZE_GB" \
    --max-monthly-budget-usd "$MAX_BUDGET_USD" \
    --strategy "$strategy" > "$json_file"

  # 抽取冠军和首位排名指标，用于生成演示汇总表。
  champion_template=$(grep '"champion_template"' "$json_file" | head -n 1 | sed -E 's/.*"champion_template": "([^"]+)".*/\1/')
  composite_score=$(grep -m1 '"composite_score"' "$json_file" | sed -E 's/.*"composite_score": ([0-9.]+).*/\1/')
  predicted_p95_ms=$(grep -m1 '"predicted_p95_ms"' "$json_file" | sed -E 's/.*"predicted_p95_ms": ([0-9]+).*/\1/')
  predicted_monthly_cost_usd=$(grep -m1 '"predicted_monthly_cost_usd"' "$json_file" | sed -E 's/.*"predicted_monthly_cost_usd": ([0-9.]+).*/\1/')
  reliability_score=$(grep -m1 '"reliability_score"' "$json_file" | sed -E 's/.*"reliability_score": ([0-9]+).*/\1/')
  risk_level=$(grep -m1 '"risk_level"' "$json_file" | sed -E 's/.*"risk_level": "([^"]+)".*/\1/')

  echo "${strategy} -> ${champion_template}"
  echo "${strategy},${champion_template},${composite_score},${predicted_p95_ms},${predicted_monthly_cost_usd},${reliability_score},${risk_level}" >> "$CSV_FILE"
  echo "| ${strategy} | ${champion_template} | ${composite_score} | ${predicted_p95_ms} | ${predicted_monthly_cost_usd} | ${reliability_score} | ${risk_level} |" >> "$MD_FILE"
done

echo
echo "full reports saved in: ${OUTPUT_DIR}"
echo "summary csv: ${CSV_FILE}"
echo "summary markdown: ${MD_FILE}"