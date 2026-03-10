#!/usr/bin/env bash
set -euo pipefail

# 用法：
#   ./scripts/demo_multi_scenario.sh [scenario_list]
# 示例：
#   ./scripts/demo_multi_scenario.sh "120:3000,1024:18000,10240:150000"
SCENARIO_LIST="${1:-120:3000,1024:18000,10240:150000}"
BASE_OUTPUT_DIR="./demo_outputs/multi_scenario"
MASTER_CSV="${BASE_OUTPUT_DIR}/multi-scenario-summary.csv"
MASTER_MD="${BASE_OUTPUT_DIR}/multi-scenario-summary.md"

mkdir -p "$BASE_OUTPUT_DIR"

echo 'scenario,size_gb,max_monthly_budget_usd,strategy,champion_template,composite_score,predicted_p95_ms,predicted_monthly_cost_usd,reliability_score,risk_level' > "$MASTER_CSV"
{
  echo '| scenario | size_gb | max_monthly_budget_usd | strategy | champion_template | composite_score | predicted_p95_ms | predicted_monthly_cost_usd | reliability_score | risk_level |'
  echo '| --- | ---: | ---: | --- | --- | ---: | ---: | ---: | ---: | --- |'
} > "$MASTER_MD"

IFS=',' read -r -a scenarios <<< "$SCENARIO_LIST"

for scenario in "${scenarios[@]}"; do
  size_gb="${scenario%%:*}"
  max_budget="${scenario##*:}"

  if [[ -z "$size_gb" || -z "$max_budget" || "$size_gb" == "$max_budget" ]]; then
    echo "invalid scenario: ${scenario}, expected size:budget" >&2
    exit 1
  fi

  scenario_name="size${size_gb}_budget${max_budget}"
  scenario_dir="${BASE_OUTPUT_DIR}/${scenario_name}"
  mkdir -p "$scenario_dir"

  ./scripts/demo_strategy_compare.sh "$size_gb" "$max_budget" "$scenario_dir" > "${scenario_dir}/run.log"

  # 汇总每个场景的四策略结果到总表。
  tail -n +2 "${scenario_dir}/champion-summary.csv" | while IFS=',' read -r strategy champion_template composite_score predicted_p95_ms predicted_monthly_cost_usd reliability_score risk_level; do
    echo "${scenario_name},${size_gb},${max_budget},${strategy},${champion_template},${composite_score},${predicted_p95_ms},${predicted_monthly_cost_usd},${reliability_score},${risk_level}" >> "$MASTER_CSV"
    echo "| ${scenario_name} | ${size_gb} | ${max_budget} | ${strategy} | ${champion_template} | ${composite_score} | ${predicted_p95_ms} | ${predicted_monthly_cost_usd} | ${reliability_score} | ${risk_level} |" >> "$MASTER_MD"
  done

done

echo "multi-scenario demo finished"
echo "master csv: ${MASTER_CSV}"
echo "master markdown: ${MASTER_MD}"