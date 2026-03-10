mod api;
mod domain_service;
mod model;
mod protocol;
mod state_store;

use clap::{Parser, Subcommand, ValueEnum};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(name = "decentralized-storage")]
#[command(about = "去中心化存储控制面原型")]
struct Cli {
    // 顶层子命令：domain/blueprint/serve。
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // 域名相关操作（注册、解析、续期等）。
    Domain(DomainArgs),
    // 输出治理蓝图说明。
    Blueprint,
    // 启动 HTTP API 服务。
    Serve(ServeArgs),
    // 自研同级协议蓝图与仿真。
    Protocol(ProtocolArgs),
    // 自研协议执行计划（分片、信誉、挑战）。
    ProtocolPlan(ProtocolPlanArgs),
    // 挑战失败后的自动修复状态机。
    ProtocolRepair(ProtocolRepairArgs),
    // 节点惩罚与恢复规则执行器。
    ProtocolPenalty(ProtocolPenaltyArgs),
    // 冷热分层迁移执行器。
    ProtocolMigrate(ProtocolMigrateArgs),
    // 一键执行完整协议治理流程。
    ProtocolWorkflow(ProtocolWorkflowArgs),
    // 生成管理层报告。
    ProtocolReport(ProtocolReportArgs),
    // 参数自动寻优。
    ProtocolOptimize(ProtocolOptimizeArgs),
    // 多目标 Pareto 前沿。
    ProtocolPareto(ProtocolParetoArgs),
    // Pareto 前沿解释器。
    ProtocolParetoExplain(ProtocolParetoArgs),
    // 业务模板一键选型。
    ProtocolTemplate(ProtocolTemplateArgs),
    // 业务模板对比矩阵。
    ProtocolTemplateMatrix(ProtocolTemplateMatrixArgs),
    // 业务模板冠军推荐。
    ProtocolTemplateChampion(ProtocolTemplateChampionArgs),
}

#[derive(Parser, Debug)]
struct DomainArgs {
    // 快照文件路径。
    #[arg(long)]
    state: PathBuf,
    // 具体动作。
    #[arg(long)]
    action: DomainAction,
    #[arg(long)]
    owner: Option<String>,
    #[arg(long)]
    domain: Option<String>,
    #[arg(long)]
    deployment: Option<String>,
    #[arg(long)]
    ttl: Option<u64>,
}

// HTTP 服务启动参数。
#[derive(Parser, Debug)]
struct ServeArgs {
    // 快照文件路径，CLI 和 HTTP 会共用它。
    #[arg(long)]
    state: PathBuf,
    // 监听地址，例如 127.0.0.1:8090。
    #[arg(long, default_value = "127.0.0.1:8090")]
    addr: String,
    // 节点唯一标识，用于 P2P 共识可观测性。
    #[arg(long, default_value = "node-local")]
    node_id: String,
    // P2P 对等节点列表（逗号分隔），例如 http://10.0.0.2:8091,http://10.0.0.3:8091
    #[arg(long, value_delimiter = ',')]
    peers: Vec<String>,
}

// 协议仿真参数。
#[derive(Parser, Debug)]
struct ProtocolArgs {
    // 数据大小（GB）。
    #[arg(long)]
    size_gb: f64,
    // 热数据占比（0-100）。
    #[arg(long)]
    hot_data_percent: u8,
    // 仿真时长（月）。
    #[arg(long)]
    months: u32,
}

// 协议执行计划参数。
#[derive(Parser, Debug)]
struct ProtocolPlanArgs {
    // 数据大小（GB）。
    #[arg(long)]
    size_gb: f64,
    // 单分片大小（MB）。
    #[arg(long, default_value_t = 64)]
    shard_mb: u32,
    // 副本数。
    #[arg(long, default_value_t = 7)]
    replicas: u8,
    // 节点数量。
    #[arg(long, default_value_t = 16)]
    node_count: u16,
}

// 自动修复状态机参数。
#[derive(Parser, Debug)]
struct ProtocolRepairArgs {
    // 总分片数。
    #[arg(long)]
    total_shards: u32,
    // 挑战失败数量。
    #[arg(long)]
    failed_challenges: u32,
    // 副本数。
    #[arg(long, default_value_t = 7)]
    replicas: u8,
    // 节点数量。
    #[arg(long, default_value_t = 16)]
    node_count: u16,
}

// 惩罚引擎参数。
#[derive(Parser, Debug)]
struct ProtocolPenaltyArgs {
    // 节点数量。
    #[arg(long)]
    node_count: u16,
    // 作恶节点占比（0-100）。
    #[arg(long)]
    malicious_percent: u8,
    // 长时间离线节点占比（0-100）。
    #[arg(long)]
    offline_percent: u8,
}

// 冷热迁移参数。
#[derive(Parser, Debug)]
struct ProtocolMigrateArgs {
    // 总分片数。
    #[arg(long)]
    total_shards: u32,
    // 当前热分片占比（0-100）。
    #[arg(long)]
    hot_shard_percent: u8,
    // 目标热分片占比（0-100）。
    #[arg(long)]
    target_hot_percent: u8,
}

// 一键工作流参数。
#[derive(Parser, Debug)]
struct ProtocolWorkflowArgs {
    // 数据大小（GB）。
    #[arg(long)]
    size_gb: f64,
    // 单分片大小（MB）。
    #[arg(long, default_value_t = 64)]
    shard_mb: u32,
    // 副本数。
    #[arg(long, default_value_t = 7)]
    replicas: u8,
    // 节点总数。
    #[arg(long, default_value_t = 16)]
    node_count: u16,
    // 当前热数据占比（0-100）。
    #[arg(long)]
    hot_data_percent: u8,
    // 目标热数据占比（0-100）。
    #[arg(long)]
    target_hot_percent: u8,
    // 作恶节点占比（0-100）。
    #[arg(long, default_value_t = 5)]
    malicious_percent: u8,
    // 长离线节点占比（0-100）。
    #[arg(long, default_value_t = 10)]
    offline_percent: u8,
}

// 管理层报告输出格式。
#[derive(Debug, Clone, ValueEnum)]
enum ReportFormat {
    Markdown,
    Json,
}

// 管理层报告参数。
#[derive(Parser, Debug)]
struct ProtocolReportArgs {
    #[arg(long)]
    size_gb: f64,
    #[arg(long, default_value_t = 64)]
    shard_mb: u32,
    #[arg(long, default_value_t = 7)]
    replicas: u8,
    #[arg(long, default_value_t = 16)]
    node_count: u16,
    #[arg(long)]
    hot_data_percent: u8,
    #[arg(long)]
    target_hot_percent: u8,
    #[arg(long, default_value_t = 5)]
    malicious_percent: u8,
    #[arg(long, default_value_t = 10)]
    offline_percent: u8,
    #[arg(long, value_enum, default_value = "markdown")]
    format: ReportFormat,
}

// 参数寻优参数。
#[derive(Parser, Debug)]
struct ProtocolOptimizeArgs {
    #[arg(long)]
    size_gb: f64,
    #[arg(long)]
    target_p95_ms: u32,
    #[arg(long)]
    max_monthly_budget_usd: f64,
    #[arg(long, default_value_t = 30)]
    max_risk_level: u8,
}

// Pareto 前沿参数。
#[derive(Parser, Debug)]
struct ProtocolParetoArgs {
    #[arg(long)]
    size_gb: f64,
    #[arg(long)]
    max_monthly_budget_usd: f64,
}

// 模板类型。
#[derive(Debug, Clone, ValueEnum)]
enum TemplateKind {
    Archive,
    Web3Frontend,
    AiInference,
}

// 业务模板参数。
#[derive(Parser, Debug)]
struct ProtocolTemplateArgs {
    #[arg(long, value_enum)]
    template: TemplateKind,
    #[arg(long)]
    size_gb: f64,
    #[arg(long)]
    max_monthly_budget_usd: Option<f64>,
    #[arg(long, value_enum, default_value = "markdown")]
    format: ReportFormat,
}

// 模板矩阵参数。
#[derive(Parser, Debug)]
struct ProtocolTemplateMatrixArgs {
    #[arg(long)]
    size_gb: f64,
    #[arg(long)]
    max_monthly_budget_usd: Option<f64>,
}

// 冠军裁决策略。
#[derive(Debug, Clone, ValueEnum)]
enum ChampionStrategyArg {
    Balanced,
    LatencyFirst,
    CostFirst,
    ReliabilityFirst,
}

// 模板冠军参数。
#[derive(Parser, Debug)]
struct ProtocolTemplateChampionArgs {
    #[arg(long)]
    size_gb: f64,
    #[arg(long)]
    max_monthly_budget_usd: Option<f64>,
    #[arg(long, value_enum, default_value = "balanced")]
    strategy: ChampionStrategyArg,
}

#[derive(Debug, Clone, ValueEnum)]
enum DomainAction {
    Register,
    Resolve,
    List,
    Renew,
    Delete,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Domain(args) => run_domain(args),
        Commands::Blueprint => Ok(blueprint().to_string()),
        Commands::Serve(args) => run_serve(args).await,
        Commands::Protocol(args) => run_protocol(args),
        Commands::ProtocolPlan(args) => run_protocol_plan(args),
        Commands::ProtocolRepair(args) => run_protocol_repair(args),
        Commands::ProtocolPenalty(args) => run_protocol_penalty(args),
        Commands::ProtocolMigrate(args) => run_protocol_migrate(args),
        Commands::ProtocolWorkflow(args) => run_protocol_workflow(args),
        Commands::ProtocolReport(args) => run_protocol_report(args),
        Commands::ProtocolOptimize(args) => run_protocol_optimize(args),
        Commands::ProtocolPareto(args) => run_protocol_pareto(args),
        Commands::ProtocolParetoExplain(args) => run_protocol_pareto_explain(args),
        Commands::ProtocolTemplate(args) => run_protocol_template(args),
        Commands::ProtocolTemplateMatrix(args) => run_protocol_template_matrix(args),
        Commands::ProtocolTemplateChampion(args) => run_protocol_template_champion(args),
    };

    match result {
        Ok(output) => println!("{}", output),
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    }
}

// 运行 protocol 子命令，输出自研协议的仿真结果。
fn run_protocol(args: ProtocolArgs) -> Result<String, String> {
    let report = protocol::simulate(protocol::SimulationInput {
        size_gb: args.size_gb,
        hot_data_percent: args.hot_data_percent,
        months: args.months,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render protocol simulation: {}", e))
}

// 运行 protocol-plan 子命令，输出分片放置和证明挑战计划。
fn run_protocol_plan(args: ProtocolPlanArgs) -> Result<String, String> {
    let report = protocol::plan(protocol::PlanInput {
        size_gb: args.size_gb,
        shard_mb: args.shard_mb,
        replicas: args.replicas,
        node_count: args.node_count,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render protocol plan: {}", e))
}

// 运行 protocol-repair 子命令，输出自动修复状态机结果。
fn run_protocol_repair(args: ProtocolRepairArgs) -> Result<String, String> {
    let report = protocol::repair_state_machine(protocol::RepairInput {
        total_shards: args.total_shards,
        failed_challenges: args.failed_challenges,
        replicas: args.replicas,
        node_count: args.node_count,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render protocol repair report: {}", e))
}

// 运行 protocol-penalty 子命令，输出惩罚与恢复结果。
fn run_protocol_penalty(args: ProtocolPenaltyArgs) -> Result<String, String> {
    let report = protocol::apply_penalty_rules(protocol::PenaltyInput {
        node_count: args.node_count,
        malicious_percent: args.malicious_percent,
        offline_percent: args.offline_percent,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render protocol penalty report: {}", e))
}

// 运行 protocol-migrate 子命令，输出冷热迁移执行结果。
fn run_protocol_migrate(args: ProtocolMigrateArgs) -> Result<String, String> {
    let report = protocol::execute_tier_migration(protocol::MigrationInput {
        total_shards: args.total_shards,
        hot_shard_percent: args.hot_shard_percent,
        target_hot_percent: args.target_hot_percent,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render protocol migration report: {}", e))
}

// 运行 protocol-workflow 子命令，一次性输出完整治理报告。
fn run_protocol_workflow(args: ProtocolWorkflowArgs) -> Result<String, String> {
    let report = protocol::run_workflow(protocol::WorkflowInput {
        size_gb: args.size_gb,
        shard_mb: args.shard_mb,
        replicas: args.replicas,
        node_count: args.node_count,
        hot_data_percent: args.hot_data_percent,
        target_hot_percent: args.target_hot_percent,
        malicious_percent: args.malicious_percent,
        offline_percent: args.offline_percent,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render protocol workflow report: {}", e))
}

// 运行 protocol-report 子命令，输出管理层报告（Markdown/JSON）。
fn run_protocol_report(args: ProtocolReportArgs) -> Result<String, String> {
    let report = protocol::build_management_report(protocol::WorkflowInput {
        size_gb: args.size_gb,
        shard_mb: args.shard_mb,
        replicas: args.replicas,
        node_count: args.node_count,
        hot_data_percent: args.hot_data_percent,
        target_hot_percent: args.target_hot_percent,
        malicious_percent: args.malicious_percent,
        offline_percent: args.offline_percent,
    })?;

    match args.format {
        ReportFormat::Markdown => Ok(protocol::render_management_markdown(&report)),
        ReportFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|e| format!("failed to render management report json: {}", e)),
    }
}

// 运行 protocol-optimize 子命令，自动反推最优参数组合。
fn run_protocol_optimize(args: ProtocolOptimizeArgs) -> Result<String, String> {
    let report = protocol::optimize_parameters(protocol::OptimizeInput {
        size_gb: args.size_gb,
        target_p95_ms: args.target_p95_ms,
        max_monthly_budget_usd: args.max_monthly_budget_usd,
        max_risk_level: args.max_risk_level,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render optimize report: {}", e))
}

// 运行 protocol-pareto 子命令，输出多目标前沿方案集合。
fn run_protocol_pareto(args: ProtocolParetoArgs) -> Result<String, String> {
    let report = protocol::pareto_frontier(protocol::ParetoInput {
        size_gb: args.size_gb,
        max_monthly_budget_usd: args.max_monthly_budget_usd,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render pareto report: {}", e))
}

// 运行 protocol-pareto-explain 子命令，输出前沿分组与场景建议。
fn run_protocol_pareto_explain(args: ProtocolParetoArgs) -> Result<String, String> {
    let report = protocol::explain_pareto(protocol::ParetoInput {
        size_gb: args.size_gb,
        max_monthly_budget_usd: args.max_monthly_budget_usd,
    })?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render pareto explain report: {}", e))
}

// 运行 protocol-template 子命令，按业务模板输出一键推荐结果。
fn run_protocol_template(args: ProtocolTemplateArgs) -> Result<String, String> {
    let template = match args.template {
        TemplateKind::Archive => protocol::BusinessTemplate::Archive,
        TemplateKind::Web3Frontend => protocol::BusinessTemplate::Web3Frontend,
        TemplateKind::AiInference => protocol::BusinessTemplate::AiInference,
    };

    let report = protocol::recommend_by_template(template, args.size_gb, args.max_monthly_budget_usd)?;

    match args.format {
        ReportFormat::Markdown => Ok(protocol::render_template_markdown(&report)),
        ReportFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|e| format!("failed to render template report json: {}", e)),
    }
}

// 运行 protocol-template-matrix 子命令，输出三模板并排对比。
fn run_protocol_template_matrix(args: ProtocolTemplateMatrixArgs) -> Result<String, String> {
    let report = protocol::template_matrix(args.size_gb, args.max_monthly_budget_usd)?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render template matrix report: {}", e))
}

// 运行 protocol-template-champion 子命令，输出冠军模板及评分排名。
fn run_protocol_template_champion(args: ProtocolTemplateChampionArgs) -> Result<String, String> {
    let strategy = match args.strategy {
        ChampionStrategyArg::Balanced => protocol::ChampionStrategy::Balanced,
        ChampionStrategyArg::LatencyFirst => protocol::ChampionStrategy::LatencyFirst,
        ChampionStrategyArg::CostFirst => protocol::ChampionStrategy::CostFirst,
        ChampionStrategyArg::ReliabilityFirst => protocol::ChampionStrategy::ReliabilityFirst,
    };

    let report = protocol::template_champion(args.size_gb, args.max_monthly_budget_usd, strategy)?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to render template champion report: {}", e))
}

// 运行 domain 子命令。
fn run_domain(args: DomainArgs) -> Result<String, String> {
    let mut snapshot = state_store::load_or_default(&args.state)?;

    let output = match args.action {
        DomainAction::Register => {
            let owner = required(args.owner, "owner")?;
            let domain = required(args.domain, "domain")?;
            let deployment = required(args.deployment, "deployment")?;
            let ttl = required(args.ttl, "ttl")?;

            let message =
                domain_service::register(&mut snapshot, &owner, &domain, &deployment, ttl)?;
            state_store::save(&args.state, &snapshot)?;
            message
        }
        DomainAction::Resolve => {
            let domain = required(args.domain, "domain")?;
            domain_service::resolve(&snapshot, &domain)?
        }
        DomainAction::List => domain_service::list(&snapshot)?,
        DomainAction::Renew => {
            let owner = required(args.owner, "owner")?;
            let domain = required(args.domain, "domain")?;
            let ttl = required(args.ttl, "ttl")?;

            let message = domain_service::renew(&mut snapshot, &owner, &domain, ttl)?;
            state_store::save(&args.state, &snapshot)?;
            message
        }
        DomainAction::Delete => {
            let owner = required(args.owner, "owner")?;
            let domain = required(args.domain, "domain")?;
            let message = domain_service::delete(&mut snapshot, &owner, &domain)?;
            state_store::save(&args.state, &snapshot)?;
            message
        }
    };

    Ok(output)
}

// 读取必填参数的通用工具函数。
fn required<T>(value: Option<T>, key: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing required --{}", key))
}

// 输出治理蓝图文本。
fn blueprint() -> &'static str {
    r#"NebulaStore 协议治理蓝图（2026）
- 热路径加速：边缘索引 + 区域热缓存，目标亚秒级访问。
- 成本可预测：法币锚定 + 锁价窗口 + 波动缓冲池。
- 可用性保障：7 副本 + 动态纠删码 + 主动修复阈值。
- 异构节点治理：健康评分调度 + 故障切换 + 热点迁移。
- 体验优化：统一 CLI/HTTP 接口 + 一键上传网关 + gas 代付。
- 合规能力：客户端加密 + 区域 pinning + 审计轨迹。
- 部署策略：NebulaStore 主网络覆盖冷热分层数据。"#
}

// 启动 HTTP API 服务。
async fn run_serve(args: ServeArgs) -> Result<String, String> {
    let upload_tmp_dir = PathBuf::from("./tmp_uploads");
    tokio::fs::create_dir_all(&upload_tmp_dir)
        .await
        .map_err(|e| format!("failed to create tmp upload dir: {}", e))?;

    let peers = sanitize_peers(args.peers);

    let state = api::ApiState {
        snapshot_path: Arc::new(args.state.clone()),
        lock: Arc::new(Mutex::new(())),
        node_id: Arc::new(args.node_id.clone()),
        peers: Arc::new(peers.clone()),
        upload_sessions: Arc::new(Mutex::new(HashMap::new())),
        upload_tmp_dir: Arc::new(upload_tmp_dir),
    };

    if !peers.is_empty() {
        let sync_state = state.clone();
        tokio::spawn(async move {
            run_p2p_consensus_loop(sync_state).await;
        });
    }

    let app = api::build_router(state);

    let listener = tokio::net::TcpListener::bind(&args.addr)
        .await
        .map_err(|e| format!("failed to bind {}: {}", args.addr, e))?;

    println!("HTTP API running at http://{}", args.addr);
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("http server crashed: {}", e))?;

    Ok("server stopped".to_string())
}

#[derive(Debug, Clone, Deserialize)]
struct PeerStateDigest {
    node_id: String,
    updated_at: u64,
    snapshot_hash: String,
}

fn sanitize_peers(raw: Vec<String>) -> Vec<String> {
    raw.into_iter()
        .map(|p| p.trim().trim_end_matches('/').to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

async fn run_p2p_consensus_loop(state: api::ApiState) {
    let client = match Client::builder().timeout(Duration::from_secs(3)).build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("p2p: failed to create http client: {}", e);
            return;
        }
    };

    loop {
        if let Err(err) = sync_once(&client, &state).await {
            eprintln!("p2p: sync round failed: {}", err);
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn sync_once(client: &Client, state: &api::ApiState) -> Result<(), String> {
    let local_snapshot = {
        let _guard = state.lock.lock().await;
        state_store::load_or_default(state.snapshot_path.as_ref())?
    };
    let local_hash = hash_snapshot(&local_snapshot);

    let mut votes: HashMap<String, usize> = HashMap::new();
    votes.insert(local_hash.clone(), 1);
    let mut hash_max_updated: HashMap<String, u64> = HashMap::new();
    hash_max_updated.insert(local_hash.clone(), local_snapshot.updated_at);

    let mut digest_by_hash: HashMap<String, PeerStateDigest> = HashMap::new();
    let mut peer_by_hash: HashMap<String, String> = HashMap::new();

    for peer in state.peers.iter() {
        let url = format!("{}/api/p2p/state", peer);
        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(_) => continue,
        };
        if !response.status().is_success() {
            continue;
        }
        let digest = match response.json::<PeerStateDigest>().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        *votes.entry(digest.snapshot_hash.clone()).or_insert(0) += 1;
        let e = hash_max_updated
            .entry(digest.snapshot_hash.clone())
            .or_insert(0);
        if digest.updated_at > *e {
            *e = digest.updated_at;
        }
        peer_by_hash
            .entry(digest.snapshot_hash.clone())
            .or_insert_with(|| peer.clone());
        digest_by_hash.insert(digest.snapshot_hash.clone(), digest);
    }

    let Some((winner_hash, _)) = votes
        .into_iter()
        .max_by(|a, b| {
            let a_updated = *hash_max_updated.get(&a.0).unwrap_or(&0);
            let b_updated = *hash_max_updated.get(&b.0).unwrap_or(&0);
            a.1.cmp(&b.1)
                .then_with(|| a_updated.cmp(&b_updated))
                .then_with(|| a.0.cmp(&b.0))
        })
    else {
        return Ok(());
    };

    if winner_hash == local_hash {
        return Ok(());
    }

    let Some(winner_digest) = digest_by_hash.get(&winner_hash) else {
        return Ok(());
    };

    let Some(peer_base) = peer_by_hash.get(&winner_hash) else {
        return Ok(());
    };

    let snapshot_url = format!("{}/api/p2p/snapshot", peer_base);
    let peer_snapshot = client
        .get(snapshot_url)
        .send()
        .await
        .map_err(|e| format!("fetch peer snapshot failed: {}", e))?
        .json::<model::StateSnapshot>()
        .await
        .map_err(|e| format!("decode peer snapshot failed: {}", e))?;

    let peer_hash = hash_snapshot(&peer_snapshot);
    if peer_hash != winner_hash {
        return Ok(());
    }

    let adopt = peer_snapshot.updated_at > local_snapshot.updated_at
        || (peer_snapshot.updated_at == local_snapshot.updated_at && peer_hash > local_hash);

    if adopt {
        let _guard = state.lock.lock().await;
        state_store::save(state.snapshot_path.as_ref(), &peer_snapshot)?;
        eprintln!(
            "p2p: adopted peer snapshot hash={} updated_at={} node={}",
            peer_hash, peer_snapshot.updated_at, winner_digest.node_id
        );
    }

    Ok(())
}

fn hash_snapshot(snapshot: &model::StateSnapshot) -> String {
    let bytes = serde_json::to_vec(snapshot).unwrap_or_default();
    let digest = Sha256::digest(&bytes);
    digest.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}
