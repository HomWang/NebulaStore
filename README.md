# 去中心化存储控制面原型（2026）

这是一个使用 Rust 实现的可落地原型，用于把去中心化存储中的治理策略标准化、可执行化。

## 项目目标

在 2026 年视角下，主流去中心化存储仍存在以下痛点：

- 热数据检索延迟高。
- 成本受代币市场波动影响大。
- 可用性和长期耐久性存在不确定性。
- 节点质量差异大导致性能不稳定。
- 用户与企业使用门槛高。
- 合规与审计实现复杂。

本项目目标已升级：实现一个纯自研的去中心化存储协议原型（NebulaStore），并针对现有主流方案痛点给出系统级治理机制。

## 项目结构

```text
decentralized-storage/
  Cargo.toml
  README.md
  docs/
    architecture.md
  frontend/
    src/
    package.json
    vite.config.ts
  src/
    main.rs             # CLI 与 HTTP 服务入口
    api.rs              # HTTP API 路由与请求处理
    protocol.rs         # 自研存储协议蓝图与仿真引擎
    model.rs            # 状态快照与治理策略数据模型
    state_store.rs      # 快照读写与持久化
    domain_service.rs   # 域名注册/解析/续期/删除核心逻辑
```

## 快速开始（CLI）

```bash
cd decentralized-storage
cargo run -- domain --state ./state.snapshot --action register --owner alice --domain local.jiamiseo.com --deployment deploy-000003 --ttl 300
```

查询域名：

```bash
cargo run -- domain --state ./state.snapshot --action resolve --domain local.jiamiseo.com
```

查看全部：

```bash
cargo run -- domain --state ./state.snapshot --action list
```

续期：

```bash
cargo run -- domain --state ./state.snapshot --action renew --owner alice --domain local.jiamiseo.com --ttl 600
```

删除：

```bash
cargo run -- domain --state ./state.snapshot --action delete --owner alice --domain local.jiamiseo.com
```

打印治理蓝图：

```bash
cargo run -- blueprint
```

自研协议仿真（新增）：

```bash
cargo run -- protocol --size-gb 50 --hot-data-percent 65 --months 12
```

自研协议执行计划（分片/信誉/挑战）：

```bash
cargo run -- protocol-plan --size-gb 120 --shard-mb 64 --replicas 7 --node-count 16
```

挑战失败自动修复状态机：

```bash
cargo run -- protocol-repair --total-shards 1920 --failed-challenges 120 --replicas 7 --node-count 16
```

节点惩罚与恢复规则：

```bash
cargo run -- protocol-penalty --node-count 64 --malicious-percent 8 --offline-percent 12
```

冷热分层迁移执行器：

```bash
cargo run -- protocol-migrate --total-shards 1920 --hot-shard-percent 35 --target-hot-percent 55
```

一键闭环治理工作流（plan -> repair -> penalty -> migrate -> summary）：

```bash
cargo run -- protocol-workflow --size-gb 120 --shard-mb 64 --replicas 7 --node-count 16 --hot-data-percent 45 --target-hot-percent 58 --malicious-percent 6 --offline-percent 10
```

管理层中文报告（支持 markdown/json）：

```bash
cargo run -- protocol-report --size-gb 120 --shard-mb 64 --replicas 7 --node-count 16 --hot-data-percent 45 --target-hot-percent 58 --malicious-percent 6 --offline-percent 10 --format markdown
```

参数自动寻优（根据目标时延/预算反推参数）：

```bash
cargo run -- protocol-optimize --size-gb 120 --target-p95-ms 1150 --max-monthly-budget-usd 3000 --max-risk-level 30
```

Pareto 前沿（成本-时延-可靠性多目标最优边界）：

```bash
cargo run -- protocol-pareto --size-gb 120 --max-monthly-budget-usd 3000
```

Pareto 方案解释器（自动分组与场景建议）：

```bash
cargo run -- protocol-pareto-explain --size-gb 120 --max-monthly-budget-usd 3000
```

业务模板一键选型（archive/web3-frontend/ai-inference）：

```bash
cargo run -- protocol-template --template web3-frontend --size-gb 120 --format markdown
```

业务模板对比矩阵（三模板并排）：

```bash
cargo run -- protocol-template-matrix --size-gb 120
```

业务模板冠军推荐（含评分排序）：

```bash
cargo run -- protocol-template-champion --size-gb 120 --strategy latency-first
```

四策略并排演示（用于评审展示）：

```bash
for s in balanced latency-first cost-first reliability-first; do
  echo "--- $s ---"
  cargo run --quiet -- protocol-template-champion --size-gb 120 --strategy "$s" | grep -E '"strategy"|"champion_template"'
done
```

一键策略对比脚本（推荐）：

```bash
chmod +x ./scripts/demo_strategy_compare.sh
./scripts/demo_strategy_compare.sh 120 3000
```

脚本会输出三类文件：

- 逐策略完整 JSON：`./demo_outputs/champion-*.json`
- 汇总 CSV：`./demo_outputs/champion-summary.csv`
- 汇总 Markdown 表格：`./demo_outputs/champion-summary.md`

查看汇总表：

```bash
cat ./demo_outputs/champion-summary.md
```

多场景批量评测（120GB/1TB/10TB）：

```bash
chmod +x ./scripts/demo_multi_scenario.sh
./scripts/demo_multi_scenario.sh "120:3000,1024:18000,10240:150000"
cat ./demo_outputs/multi_scenario/multi-scenario-summary.md
```

API 批量评测（先启动服务）：

```bash
chmod +x ./scripts/demo_api_strategy_compare.sh
./scripts/demo_api_strategy_compare.sh http://127.0.0.1:8091 120 3000
cat ./demo_outputs/api_compare/api-champion-summary.md
```

API 端到端一键评测（自动起服务/跑评测/自动停服）：

```bash
chmod +x ./scripts/demo_api_e2e.sh
./scripts/demo_api_e2e.sh http://127.0.0.1:8091 120 3000
cat ./demo_outputs/api_compare/api-champion-summary.md
```

## 5 分钟评审流程

1. 展示协议闭环能力（计划、修复、惩罚、迁移）：

```bash
cargo run -- protocol-workflow --size-gb 120 --shard-mb 64 --replicas 7 --node-count 16 --hot-data-percent 45 --target-hot-percent 58 --malicious-percent 6 --offline-percent 10
```

2. 输出管理层可读报告（Markdown）：

```bash
cargo run -- protocol-report --size-gb 120 --shard-mb 64 --replicas 7 --node-count 16 --hot-data-percent 45 --target-hot-percent 58 --malicious-percent 6 --offline-percent 10 --format markdown
```

3. 对比四种冠军裁决策略并展示结论：

```bash
./scripts/demo_strategy_compare.sh 120 3000
```

4. 若需展示服务化能力，运行 API 版批量对比：

```bash
./scripts/demo_api_e2e.sh http://127.0.0.1:8091 120 3000
```

## 下一步（HTTP API）

启动服务：

```bash
cargo run -- serve --state ./state.snapshot --addr 127.0.0.1:8091
```

可用接口：

- `GET /api/health`
- `GET /api/chain/blocks`
- `POST /api/data/upload`
- `POST /api/data/upload/file`
- `POST /api/data/upload/chunk/init`
- `POST /api/data/upload/chunk/part`
- `POST /api/data/upload/chunk/complete`
- `GET /api/data/objects`
- `GET /api/data/objects/:data_id`
- `GET /api/data/objects/by-hash/:content_hash`
- `GET /api/domains`
- `GET /api/domains/:domain`
- `POST /api/domains/register`
- `POST /api/domains/renew`
- `DELETE /api/domains/:domain?owner=alice`
- `GET /api/protocol/blueprint`
- `POST /api/protocol/simulate`
- `POST /api/protocol/plan`
- `POST /api/protocol/repair`
- `POST /api/protocol/penalty`
- `POST /api/protocol/migrate`
- `POST /api/protocol/workflow`
- `POST /api/protocol/report`
- `POST /api/protocol/optimize`
- `POST /api/protocol/pareto`
- `POST /api/protocol/pareto/explain`
- `POST /api/protocol/template`
- `POST /api/protocol/template/matrix`
- `POST /api/protocol/template/champion`

## React SPA 控制台

用于查看区块信息、策略冠军推荐，并在独立“数据信息页”按数据ID或关键词查询存储了什么数据。

当前前端支持“列表 + 详情弹窗”交互：

- 区块信息：列表中可进入“区块详情弹窗”，并展示当前链总高度。
- 数据信息：通过“数据上传”按钮打开上传弹窗；列表与查询结果都可进入“数据详情弹窗”。
- 当数据类型为 `image/*` 或 `video/*` 且存在可预览地址时，详情弹窗支持“点击按钮预览媒体”。
- 数据 `data_id` 与 `content_hash` 在查询结果、列表、详情弹窗中均提供复制 icon，方便快速转发。

`GET /api/chain/blocks` 返回 `total_height` 字段，表示当前链总高度。

区块中的 `status=finalized` 表示在当前演示系统中已确认且不可回滚；`data_gb` 为基于副本与时延目标推算的估算值，不是链上真实观测值。

`data_id` 采用 `nebula-{timestamp+sequence+entropy}` 方案（时间有序 + 并发安全序列 + 熵混合），可支撑高并发生成并避免冲突。

每条数据对象同时生成 `content_hash`（SHA-256）用于校验与二级检索。

`P95(ms)` 会根据当前对象规模自动调优：小规模场景会下探到更低时延目标（例如 120ms），避免展示为不合理的高延迟。

前端数据信息页展示的是“目标时延(策略值)”，不是实时观测统计值。

按内容哈希查询示例：

```bash
curl -sS 'http://127.0.0.1:8091/api/data/objects/by-hash/<64位sha256>'
```

文本上传示例：

```bash
curl -sS -X POST 'http://127.0.0.1:8091/api/data/upload' \
  -H 'Content-Type: application/json' \
  -d '{"owner":"alice","data_name":"dataset-demo","content_type":"text/plain","content":"this is my uploaded data payload"}'
```

文件上传示例（API 保留，适合自动化脚本）：

```bash
curl -sS -X POST 'http://127.0.0.1:8091/api/data/upload/file' \
  -F 'owner=alice' \
  -F 'data_name=dataset-demo' \
  -F 'content_type=text/plain' \
  -F 'file=@./README.md'
```

分片上传起步版（解决大文件上传中网络抖动问题）：

1. `POST /api/data/upload/chunk/init` 创建会话，获取 `upload_id`。
2. `POST /api/data/upload/chunk/part` 逐片上传（当前版本要求按 `0..N-1` 顺序）。
3. `POST /api/data/upload/chunk/complete` 合并分片并入库。

说明：当前分片功能为“起步版”，重点是把大文件拆分后独立重传单片，后续可再增强为断点续传（查询已上传片段并从中断点继续）。

前端上传方式（当前默认）：

- 上传区提供 `Tabs` 切换：`数据编辑上传` 与 `文件/分片上传`。
- 默认选中 `数据编辑上传`，采用 `JSON` 请求方式与大文本框编辑提交。
- 切换到 `文件/分片上传` 后可使用文件直传或分片上传能力。
- 二进制文件类型（如 `image/jpeg`）在对象列表中不会展示乱码预览，而是显示“二进制内容不展示文本预览”。

启动步骤：

```bash
# 终端 1：启动后端 API
cargo run -- serve --state ./state.snapshot --addr 127.0.0.1:8091

# 终端 2：启动 React SPA
cd frontend
npm install
npm run dev
```

打开地址：`http://127.0.0.1:5174`

冠军裁决策略可选值：

- `balanced`
- `latency-first`
- `cost-first`
- `reliability-first`

冠军策略 API 调用示例：

```bash
curl -sS -X POST 'http://127.0.0.1:8091/api/protocol/template/champion' \
  -H 'Content-Type: application/json' \
  -d '{"size_gb":120,"max_monthly_budget_usd":3000,"strategy":"cost-first"}'
```

## 对痛点的工程化应对

- 检索慢：在 `retrieval_profile` 中定义目标延迟与缓存加速层。
- 成本波动：在 `economics_profile` 中定义报价有效期与再平衡阈值。
- 可用性风险：在 `durability_profile` 中定义副本数、纠删码、修复阈值。
- 节点异构：使用节点健康评分调度，避免低质量节点拖累全网。
- 体验门槛：提供统一 CLI 与 HTTP 两种操作面。
- 合规压力：内置客户端加密、区域 pin、审计追踪策略字段。

## 注意事项

- 当前实现是控制面原型，不是完整的数据面存储系统。
- `.snapshot` 为本地 JSON 状态文件，适合演示与可重复测试。
- 生产化还需补充真实 P2P 网络、共识和密码学证明模块。

## NebulaStore 核心机制

- 检索层：边缘索引 + 区域热缓存，目标亚秒级检索。
- 成本层：法币锚定结算 + 锁价机制 + 波动缓冲池，降低价格抖动。
- 耐久层：7 副本 + 动态纠删码 + 主动修复守护进程。
- 调度层：基于时延/在线率/吞吐的节点健康评分调度。
- 体验层：账户抽象 + 无 gas 代付 + 一键上传网关。
- 证明层：按时间窗口执行抽样挑战，验证副本持续可用。
- 修复层：挑战失败自动进入任务队列并执行重复制。
- 惩罚层：作恶与长离线分别执行不同罚没与恢复周期。
- 迁移层：按热度自动在 hot-edge 与 cold-archive 间迁移分片。
