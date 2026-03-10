use serde::Serialize;

// 协议蓝图：描述一个纯自研的去中心化存储系统设计。
#[derive(Debug, Clone, Serialize)]
pub struct ProtocolBlueprint {
    pub name: String,
    pub positioning: String,
    pub retrieval_engine: String,
    pub cost_engine: String,
    pub durability_engine: String,
    pub scheduler_engine: String,
    pub ux_engine: String,
    pub compliance_engine: String,
}

// 仿真输入：用于估算该协议在不同负载下的表现。
#[derive(Debug, Clone, Copy)]
pub struct SimulationInput {
    pub size_gb: f64,
    pub hot_data_percent: u8,
    pub months: u32,
}

// 仿真输出：对应你关注的核心痛点指标。
#[derive(Debug, Clone, Serialize)]
pub struct SimulationReport {
    pub protocol: String,
    pub p95_latency_ms: u32,
    pub annual_cost_volatility_percent: f64,
    pub durability_availability_percent: f64,
    pub expected_repair_bandwidth_percent: f64,
    pub user_experience_score: u8,
    pub notes: Vec<String>,
}

// 规划输入：用于生成分片放置、信誉评分和证明挑战计划。
#[derive(Debug, Clone, Copy)]
pub struct PlanInput {
    pub size_gb: f64,
    pub shard_mb: u32,
    pub replicas: u8,
    pub node_count: u16,
}

// 节点画像：描述节点基础性能和信誉。
#[derive(Debug, Clone, Serialize)]
pub struct NodeProfile {
    pub node_id: String,
    pub region: String,
    pub latency_ms: u32,
    pub online_percent: f64,
    pub throughput_mbps: u32,
    pub reputation_score: u16,
}

// 分片放置结果：记录某个分片的副本节点列表。
#[derive(Debug, Clone, Serialize)]
pub struct ShardPlacement {
    pub shard_id: u32,
    pub replica_nodes: Vec<String>,
}

// 证明挑战窗口：定义挑战频率和抽样比例。
#[derive(Debug, Clone, Serialize)]
pub struct ChallengeWindow {
    pub window_hours: u32,
    pub challenge_interval_minutes: u32,
    pub sample_ratio_percent: u8,
}

// 规划输出：汇总协议核心执行计划。
#[derive(Debug, Clone, Serialize)]
pub struct ProtocolPlanReport {
    pub protocol: String,
    pub total_shards: u32,
    pub effective_replicas: u8,
    pub selected_nodes: Vec<NodeProfile>,
    pub placement_preview: Vec<ShardPlacement>,
    pub challenge: ChallengeWindow,
    pub estimated_repair_trigger_percent: f64,
}

// 修复输入：用于挑战失败后生成自动修复任务。
#[derive(Debug, Clone, Copy)]
pub struct RepairInput {
    pub total_shards: u32,
    pub failed_challenges: u32,
    pub replicas: u8,
    pub node_count: u16,
}

// 修复任务状态。
#[derive(Debug, Clone, Serialize)]
pub enum RepairStatus {
    Queued,
    InProgress,
    Completed,
    Failed,
}

// 修复任务实体。
#[derive(Debug, Clone, Serialize)]
pub struct RepairTask {
    pub task_id: String,
    pub shard_id: u32,
    pub from_node: String,
    pub to_node: String,
    pub retry_count: u8,
    pub status: RepairStatus,
}

// 修复状态机输出。
#[derive(Debug, Clone, Serialize)]
pub struct RepairReport {
    pub protocol: String,
    pub queued: u32,
    pub in_progress: u32,
    pub completed: u32,
    pub failed: u32,
    pub estimated_repair_minutes: u32,
    pub tasks_preview: Vec<RepairTask>,
}

// 惩罚输入：用于模拟节点作恶和离线惩罚。
#[derive(Debug, Clone, Copy)]
pub struct PenaltyInput {
    pub node_count: u16,
    pub malicious_percent: u8,
    pub offline_percent: u8,
}

// 节点惩罚与恢复详情。
#[derive(Debug, Clone, Serialize)]
pub struct NodePenalty {
    pub node_id: String,
    pub reason: String,
    pub slash_percent: f64,
    pub reputation_before: u16,
    pub reputation_after: u16,
    pub recovery_epochs: u32,
}

// 惩罚引擎输出。
#[derive(Debug, Clone, Serialize)]
pub struct PenaltyReport {
    pub protocol: String,
    pub penalized_nodes: u32,
    pub average_slash_percent: f64,
    pub total_recovery_epochs: u32,
    pub penalties_preview: Vec<NodePenalty>,
}

// 冷热迁移输入：用于按访问热度执行分层迁移。
#[derive(Debug, Clone, Copy)]
pub struct MigrationInput {
    pub total_shards: u32,
    pub hot_shard_percent: u8,
    pub target_hot_percent: u8,
}

// 迁移动作。
#[derive(Debug, Clone, Serialize)]
pub struct MigrationAction {
    pub shard_id: u32,
    pub from_tier: String,
    pub to_tier: String,
    pub reason: String,
}

// 迁移执行结果。
#[derive(Debug, Clone, Serialize)]
pub struct MigrationReport {
    pub protocol: String,
    pub moved_to_hot: u32,
    pub moved_to_cold: u32,
    pub p95_latency_before_ms: u32,
    pub p95_latency_after_ms: u32,
    pub monthly_cost_before_usd: f64,
    pub monthly_cost_after_usd: f64,
    pub actions_preview: Vec<MigrationAction>,
}

// 一键工作流输入：统一驱动 plan/repair/penalty/migrate 四个阶段。
#[derive(Debug, Clone, Copy)]
pub struct WorkflowInput {
    pub size_gb: f64,
    pub shard_mb: u32,
    pub replicas: u8,
    pub node_count: u16,
    pub hot_data_percent: u8,
    pub target_hot_percent: u8,
    pub malicious_percent: u8,
    pub offline_percent: u8,
}

// 一键工作流输出：聚合各阶段结果并给出总览指标。
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowReport {
    pub protocol: String,
    pub plan: ProtocolPlanReport,
    pub repair: RepairReport,
    pub penalty: PenaltyReport,
    pub migration: MigrationReport,
    pub final_summary: WorkflowSummary,
}

// 总览摘要：用于演示和决策汇报。
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowSummary {
    pub storage_profile: String,
    pub reliability_score: u8,
    pub latency_delta_ms: i32,
    pub monthly_cost_delta_usd: f64,
    pub risk_level: String,
}

// 管理层报告：用于演示、汇报和决策。
#[derive(Debug, Clone, Serialize)]
pub struct ManagementReport {
    pub protocol: String,
    pub executive_summary: String,
    pub key_metrics: ReportMetrics,
    pub key_risks: Vec<String>,
    pub recommendations: Vec<String>,
}

// 报告指标区：统一呈现关键量化结果。
#[derive(Debug, Clone, Serialize)]
pub struct ReportMetrics {
    pub reliability_score: u8,
    pub risk_level: String,
    pub latency_delta_ms: i32,
    pub monthly_cost_delta_usd: f64,
    pub penalized_nodes: u32,
    pub repair_failed_tasks: u32,
}

// 参数寻优输入：给定业务目标，让系统反推参数组合。
#[derive(Debug, Clone, Copy)]
pub struct OptimizeInput {
    pub size_gb: f64,
    pub target_p95_ms: u32,
    pub max_monthly_budget_usd: f64,
    pub max_risk_level: u8,
}

// 候选参数方案。
#[derive(Debug, Clone, Serialize)]
pub struct OptimizeCandidate {
    pub replicas: u8,
    pub node_count: u16,
    pub hot_data_percent: u8,
    pub target_hot_percent: u8,
    pub predicted_p95_ms: u32,
    pub predicted_monthly_cost_usd: f64,
    pub predicted_reliability_score: u8,
    pub score: f64,
}

// 寻优结果：包含最优方案和候选列表。
#[derive(Debug, Clone, Serialize)]
pub struct OptimizeReport {
    pub protocol: String,
    pub target_p95_ms: u32,
    pub max_monthly_budget_usd: f64,
    pub best: OptimizeCandidate,
    pub top_candidates: Vec<OptimizeCandidate>,
    pub notes: Vec<String>,
}

// Pareto 前沿输入：用于多目标方案筛选。
#[derive(Debug, Clone, Copy)]
pub struct ParetoInput {
    pub size_gb: f64,
    pub max_monthly_budget_usd: f64,
}

// Pareto 输出：返回互不支配方案集合。
#[derive(Debug, Clone, Serialize)]
pub struct ParetoReport {
    pub protocol: String,
    pub frontier: Vec<OptimizeCandidate>,
    pub candidate_count: usize,
    pub notes: Vec<String>,
}

// Pareto 方案解释条目：为每个前沿点打标签并给出适用场景。
#[derive(Debug, Clone, Serialize)]
pub struct ParetoProfile {
    pub candidate: OptimizeCandidate,
    pub profile_tag: String,
    pub best_for: String,
    pub recommendation: String,
}

// Pareto 解释输出：便于管理层做方案选择。
#[derive(Debug, Clone, Serialize)]
pub struct ParetoExplainReport {
    pub protocol: String,
    pub summary: String,
    pub profiles: Vec<ParetoProfile>,
}

// 业务模板：按常见场景预置目标约束。
#[derive(Debug, Clone, Copy, Serialize)]
pub enum BusinessTemplate {
    Archive,
    Web3Frontend,
    AiInference,
}

// 模板推荐结果：聚合优化结果、工作流结果和管理层报告。
#[derive(Debug, Clone, Serialize)]
pub struct TemplateReport {
    pub protocol: String,
    pub template: String,
    pub description: String,
    pub applied_target_p95_ms: u32,
    pub applied_budget_usd: f64,
    pub optimize: OptimizeReport,
    pub workflow: WorkflowReport,
    pub management: ManagementReport,
}

// 模板对比行：用于并排评估不同业务模板。
#[derive(Debug, Clone, Serialize)]
pub struct TemplateMatrixRow {
    pub template: String,
    pub target_p95_ms: u32,
    pub budget_usd: f64,
    pub recommended_replicas: u8,
    pub recommended_node_count: u16,
    pub recommended_hot_percent: u8,
    pub predicted_p95_ms: u32,
    pub predicted_monthly_cost_usd: f64,
    pub reliability_score: u8,
    pub risk_level: String,
}

// 模板矩阵输出：聚合多个模板推荐结果。
#[derive(Debug, Clone, Serialize)]
pub struct TemplateMatrixReport {
    pub protocol: String,
    pub size_gb: f64,
    pub rows: Vec<TemplateMatrixRow>,
    pub decision_hint: String,
}

// 模板冠军结果：给出排序后的最佳模板和评分明细。
#[derive(Debug, Clone, Serialize)]
pub struct TemplateChampionReport {
    pub protocol: String,
    pub size_gb: f64,
    pub strategy: String,
    pub ranking: Vec<TemplateRankItem>,
    pub champion_template: String,
    pub champion_reason: String,
}

// 单模板评分行。
#[derive(Debug, Clone, Serialize)]
pub struct TemplateRankItem {
    pub template: String,
    pub composite_score: f64,
    pub predicted_p95_ms: u32,
    pub predicted_monthly_cost_usd: f64,
    pub reliability_score: u8,
    pub risk_level: String,
}

// 冠军裁决策略：用于同分时的优先级决策。
#[derive(Debug, Clone, Copy, Serialize)]
pub enum ChampionStrategy {
    Balanced,
    LatencyFirst,
    CostFirst,
    ReliabilityFirst,
}

// 模板预置参数。
#[derive(Debug, Clone, Copy)]
struct TemplatePreset {
    target_p95_ms: u32,
    budget_usd: f64,
    max_risk_level: u8,
    malicious_percent: u8,
    offline_percent: u8,
    description: &'static str,
}

// 返回协议蓝图，强调针对行业痛点的结构化机制。
pub fn blueprint() -> ProtocolBlueprint {
    ProtocolBlueprint {
        name: "NebulaStore".to_string(),
        positioning: "纯自研去中心化存储网络，强调低延迟与成本稳定".to_string(),
        retrieval_engine: "双层检索路径（边缘索引 + 区域热缓存），目标 p95 < 900ms".to_string(),
        cost_engine: "法币锚定结算 + 波动缓冲池 + 长单锁价机制".to_string(),
        durability_engine: "动态纠删码 + 跨域 7 副本 + 主动修复守护进程".to_string(),
        scheduler_engine: "节点健康评分调度（时延/在线率/丢包/吞吐）".to_string(),
        ux_engine: "账户抽象 + 无 gas 代付 + 一键上传网关".to_string(),
        compliance_engine: "客户端加密 + 区域策略 + 审计证明日志".to_string(),
    }
}

// 按输入规模生成仿真报告，帮助项目选型和参数规划。
pub fn simulate(input: SimulationInput) -> Result<SimulationReport, String> {
    if input.size_gb <= 0.0 {
        return Err("size_gb must be > 0".to_string());
    }
    if input.hot_data_percent > 100 {
        return Err("hot_data_percent must be in [0, 100]".to_string());
    }
    if input.months == 0 {
        return Err("months must be > 0".to_string());
    }

    // 热数据比例越高，对检索层压力越大。
    let latency = 420_u32 + (input.hot_data_percent as u32 * 6);

    // 通过法币锚定和锁价机制，把波动控制在较低区间。
    let base_vol = 18.0_f64;
    let volatility = (base_vol + (input.months as f64 * 0.12)).min(36.0);

    // 跨域副本和主动修复提升可用性，但会带来一定修复带宽。
    let availability = (99.20 + (input.months as f64 * 0.01)).min(99.90);
    let repair_bandwidth = 8.0 + (input.size_gb / 10_000.0 * 3.0);

    // UX 分数用来衡量大众可用程度（满分 100）。
    let ux_score = if input.hot_data_percent > 70 { 82 } else { 88 };

    Ok(SimulationReport {
        protocol: "NebulaStore".to_string(),
        p95_latency_ms: latency,
        annual_cost_volatility_percent: (volatility * 100.0).round() / 100.0,
        durability_availability_percent: (availability * 100.0).round() / 100.0,
        expected_repair_bandwidth_percent: (repair_bandwidth * 100.0).round() / 100.0,
        user_experience_score: ux_score,
        notes: vec![
            "通过边缘索引与区域热缓存缓解检索慢问题".to_string(),
            "通过法币锚定和锁价机制降低代币波动风险".to_string(),
            "通过 7 副本 + 动态纠删码 + 主动修复提升可用性".to_string(),
        ],
    })
}

// 生成协议执行计划：包括分片放置、节点信誉与证明挑战参数。
pub fn plan(input: PlanInput) -> Result<ProtocolPlanReport, String> {
    if input.size_gb <= 0.0 {
        return Err("size_gb must be > 0".to_string());
    }
    if input.shard_mb == 0 {
        return Err("shard_mb must be > 0".to_string());
    }
    if input.replicas < 3 || input.replicas > 12 {
        return Err("replicas must be in [3, 12]".to_string());
    }
    if input.node_count < input.replicas as u16 {
        return Err("node_count must be >= replicas".to_string());
    }

    let total_mb = (input.size_gb * 1024.0).ceil() as u32;
    let total_shards = total_mb.div_ceil(input.shard_mb);

    // 先构造候选节点并计算信誉分，随后按信誉排序用于放置。
    let mut nodes = build_nodes(input.node_count);
    nodes.sort_by(|a, b| b.reputation_score.cmp(&a.reputation_score));

    // 为避免输出过大，仅展示前 5 个分片的放置预览。
    let preview_count = total_shards.min(5);
    let mut placement_preview = Vec::with_capacity(preview_count as usize);

    for shard_id in 0..preview_count {
        let mut replica_nodes = Vec::with_capacity(input.replicas as usize);
        for replica_idx in 0..input.replicas {
            let index = ((shard_id as usize * 7) + replica_idx as usize) % nodes.len();
            replica_nodes.push(nodes[index].node_id.clone());
        }
        placement_preview.push(ShardPlacement {
            shard_id,
            replica_nodes,
        });
    }

    // 挑战周期按副本数增强，副本越多抽样越严格。
    let challenge = ChallengeWindow {
        window_hours: 24,
        challenge_interval_minutes: 20,
        sample_ratio_percent: (20 + input.replicas).min(45),
    };

    // 信誉越高，修复触发比例越低，代表系统稳定性更好。
    let avg_reputation =
        nodes.iter().map(|n| n.reputation_score as f64).sum::<f64>() / nodes.len() as f64;
    let estimated_repair_trigger_percent =
        (14.0 - (avg_reputation / 100.0).min(6.0)).max(5.0);

    Ok(ProtocolPlanReport {
        protocol: "NebulaStore".to_string(),
        total_shards,
        effective_replicas: input.replicas,
        selected_nodes: nodes,
        placement_preview,
        challenge,
        estimated_repair_trigger_percent: (estimated_repair_trigger_percent * 100.0).round()
            / 100.0,
    })
}

// 构建节点列表并基于性能指标计算信誉分。
fn build_nodes(node_count: u16) -> Vec<NodeProfile> {
    let regions = ["ap-east", "eu-central", "us-west", "sa-east"];
    let mut nodes = Vec::with_capacity(node_count as usize);

    for idx in 0..node_count {
        let latency_ms = 45 + ((idx as u32 * 17) % 120);
        let online_percent = 95.0 + ((idx as f64 * 0.23) % 4.5);
        let throughput_mbps = 180 + ((idx as u32 * 37) % 420);
        let reputation_score = score_node(latency_ms, online_percent, throughput_mbps);

        nodes.push(NodeProfile {
            node_id: format!("node-{:04}", idx + 1),
            region: regions[(idx as usize) % regions.len()].to_string(),
            latency_ms,
            online_percent: (online_percent * 100.0).round() / 100.0,
            throughput_mbps,
            reputation_score,
        });
    }

    nodes
}

// 信誉评分函数：时延越低、在线率越高、吞吐越大，分数越高。
fn score_node(latency_ms: u32, online_percent: f64, throughput_mbps: u32) -> u16 {
    let latency_score = (200_i32 - latency_ms as i32).clamp(20, 180) as f64;
    let online_score = (online_percent * 1.8).clamp(120.0, 185.0);
    let throughput_score = (throughput_mbps as f64 / 4.0).clamp(45.0, 170.0);

    (latency_score + online_score + throughput_score).round() as u16
}

// 挑战失败后触发自动修复状态机，输出任务队列与状态分布。
pub fn repair_state_machine(input: RepairInput) -> Result<RepairReport, String> {
    if input.total_shards == 0 {
        return Err("total_shards must be > 0".to_string());
    }
    if input.failed_challenges > input.total_shards {
        return Err("failed_challenges must be <= total_shards".to_string());
    }
    if input.replicas < 3 {
        return Err("replicas must be >= 3".to_string());
    }
    if input.node_count < input.replicas as u16 {
        return Err("node_count must be >= replicas".to_string());
    }

    // 按失败挑战数构造修复任务，预览前 12 条。
    let mut tasks_preview = Vec::new();
    let preview = input.failed_challenges.min(12);

    let mut queued = 0_u32;
    let mut in_progress = 0_u32;
    let mut completed = 0_u32;
    let mut failed = 0_u32;

    for idx in 0..preview {
        let status = match idx % 10 {
            0..=2 => RepairStatus::Queued,
            3..=5 => RepairStatus::InProgress,
            6..=8 => RepairStatus::Completed,
            _ => RepairStatus::Failed,
        };

        match status {
            RepairStatus::Queued => queued += 1,
            RepairStatus::InProgress => in_progress += 1,
            RepairStatus::Completed => completed += 1,
            RepairStatus::Failed => failed += 1,
        }

        tasks_preview.push(RepairTask {
            task_id: format!("repair-{:05}", idx + 1),
            shard_id: idx,
            from_node: format!("node-{:04}", (idx as usize % input.node_count as usize) + 1),
            to_node: format!(
                "node-{:04}",
                ((idx as usize + 5) % input.node_count as usize) + 1
            ),
            retry_count: if matches!(status, RepairStatus::Failed) { 2 } else { 0 },
            status,
        });
    }

    // 对未展示任务按同样比例估算状态分布。
    if input.failed_challenges > preview {
        let remain = input.failed_challenges - preview;
        queued += remain * 30 / 100;
        in_progress += remain * 30 / 100;
        completed += remain * 30 / 100;
        failed += remain - (remain * 90 / 100);
    }

    let estimated_repair_minutes = ((input.failed_challenges as f64 * 2.8)
        / (input.replicas as f64 * 0.9))
        .ceil() as u32;

    Ok(RepairReport {
        protocol: "NebulaStore".to_string(),
        queued,
        in_progress,
        completed,
        failed,
        estimated_repair_minutes,
        tasks_preview,
    })
}

// 节点作恶/离线惩罚引擎，输出惩罚比例和恢复周期。
pub fn apply_penalty_rules(input: PenaltyInput) -> Result<PenaltyReport, String> {
    if input.node_count == 0 {
        return Err("node_count must be > 0".to_string());
    }
    if input.malicious_percent > 100 || input.offline_percent > 100 {
        return Err("percent fields must be in [0, 100]".to_string());
    }

    let malicious_nodes = ((input.node_count as u32 * input.malicious_percent as u32) / 100)
        .min(input.node_count as u32);
    let offline_nodes = ((input.node_count as u32 * input.offline_percent as u32) / 100)
        .min(input.node_count as u32 - malicious_nodes);
    let penalized_nodes = malicious_nodes + offline_nodes;

    let mut penalties_preview = Vec::new();
    let preview = penalized_nodes.min(12);
    let mut total_slash = 0.0_f64;
    let mut total_recovery_epochs = 0_u32;

    for idx in 0..preview {
        let is_malicious = idx < malicious_nodes;
        let reason = if is_malicious {
            "malicious-proof".to_string()
        } else {
            "extended-offline".to_string()
        };

        let slash_percent = if is_malicious { 18.0 } else { 6.5 };
        let reputation_before = 420_u16.saturating_sub((idx as u16) * 7);
        let penalty_points = if is_malicious { 120 } else { 45 };
        let reputation_after = reputation_before.saturating_sub(penalty_points);
        let recovery_epochs = if is_malicious { 48 } else { 18 };

        total_slash += slash_percent;
        total_recovery_epochs += recovery_epochs;

        penalties_preview.push(NodePenalty {
            node_id: format!("node-{:04}", idx + 1),
            reason,
            slash_percent,
            reputation_before,
            reputation_after,
            recovery_epochs,
        });
    }

    let average_slash_percent = if preview > 0 {
        (total_slash / preview as f64 * 100.0).round() / 100.0
    } else {
        0.0
    };

    Ok(PenaltyReport {
        protocol: "NebulaStore".to_string(),
        penalized_nodes,
        average_slash_percent,
        total_recovery_epochs,
        penalties_preview,
    })
}

// 冷热分层迁移执行器，按目标热分片占比给出迁移动作和收益。
pub fn execute_tier_migration(input: MigrationInput) -> Result<MigrationReport, String> {
    if input.total_shards == 0 {
        return Err("total_shards must be > 0".to_string());
    }
    if input.hot_shard_percent > 100 || input.target_hot_percent > 100 {
        return Err("percent fields must be in [0, 100]".to_string());
    }

    let current_hot = input.total_shards * input.hot_shard_percent as u32 / 100;
    let target_hot = input.total_shards * input.target_hot_percent as u32 / 100;

    let (moved_to_hot, moved_to_cold) = if target_hot >= current_hot {
        (target_hot - current_hot, 0)
    } else {
        (0, current_hot - target_hot)
    };

    // 迁移预览仅保留前 10 条，避免响应过大。
    let preview_moves = (moved_to_hot + moved_to_cold).min(10);
    let mut actions_preview = Vec::with_capacity(preview_moves as usize);
    for idx in 0..preview_moves {
        let (from_tier, to_tier, reason) = if idx < moved_to_hot {
            (
                "cold-archive".to_string(),
                "hot-edge".to_string(),
                "access-ratio-increase".to_string(),
            )
        } else {
            (
                "hot-edge".to_string(),
                "cold-archive".to_string(),
                "access-ratio-drop".to_string(),
            )
        };

        actions_preview.push(MigrationAction {
            shard_id: idx,
            from_tier,
            to_tier,
            reason,
        });
    }

    // 简化估算：热分片越高，时延更低但成本更高。
    let p95_latency_before_ms = 1500_u32.saturating_sub(input.hot_shard_percent as u32 * 8);
    let p95_latency_after_ms = 1500_u32.saturating_sub(input.target_hot_percent as u32 * 8);

    let monthly_cost_before_usd =
        (1200.0 + input.hot_shard_percent as f64 * 18.0 + input.total_shards as f64 * 0.35)
            .round();
    let monthly_cost_after_usd =
        (1200.0 + input.target_hot_percent as f64 * 18.0 + input.total_shards as f64 * 0.35)
            .round();

    Ok(MigrationReport {
        protocol: "NebulaStore".to_string(),
        moved_to_hot,
        moved_to_cold,
        p95_latency_before_ms,
        p95_latency_after_ms,
        monthly_cost_before_usd,
        monthly_cost_after_usd,
        actions_preview,
    })
}

// 一键执行完整协议治理工作流。
pub fn run_workflow(input: WorkflowInput) -> Result<WorkflowReport, String> {
    let plan = plan(PlanInput {
        size_gb: input.size_gb,
        shard_mb: input.shard_mb,
        replicas: input.replicas,
        node_count: input.node_count,
    })?;

    // 用分片总量和热占比估算挑战失败规模。
    let failed_challenges = ((plan.total_shards as f64)
        * (0.025 + input.hot_data_percent as f64 / 10_000.0))
        .ceil() as u32;

    let repair = repair_state_machine(RepairInput {
        total_shards: plan.total_shards,
        failed_challenges,
        replicas: input.replicas,
        node_count: input.node_count,
    })?;

    let penalty = apply_penalty_rules(PenaltyInput {
        node_count: input.node_count,
        malicious_percent: input.malicious_percent,
        offline_percent: input.offline_percent,
    })?;

    let current_hot_percent = input.hot_data_percent;
    let migration = execute_tier_migration(MigrationInput {
        total_shards: plan.total_shards,
        hot_shard_percent: current_hot_percent,
        target_hot_percent: input.target_hot_percent,
    })?;

    let reliability_score = {
        let base = 90_i32;
        let penalty_deduction = (penalty.penalized_nodes as i32 / 2).min(18);
        let repair_deduction = (repair.failed as i32 / 2).min(12);
        (base - penalty_deduction - repair_deduction).clamp(55, 96) as u8
    };

    let latency_delta_ms = migration.p95_latency_before_ms as i32 - migration.p95_latency_after_ms as i32;
    let monthly_cost_delta_usd =
        ((migration.monthly_cost_after_usd - migration.monthly_cost_before_usd) * 100.0).round()
            / 100.0;

    let risk_level = if reliability_score >= 85 {
        "low"
    } else if reliability_score >= 70 {
        "medium"
    } else {
        "high"
    }
    .to_string();

    let storage_profile = if input.target_hot_percent >= 60 {
        "hot-biased"
    } else if input.target_hot_percent <= 35 {
        "cold-biased"
    } else {
        "balanced"
    }
    .to_string();

    Ok(WorkflowReport {
        protocol: "NebulaStore".to_string(),
        plan,
        repair,
        penalty,
        migration,
        final_summary: WorkflowSummary {
            storage_profile,
            reliability_score,
            latency_delta_ms,
            monthly_cost_delta_usd,
            risk_level,
        },
    })
}

// 生成管理层报告（结构化 JSON）。
pub fn build_management_report(input: WorkflowInput) -> Result<ManagementReport, String> {
    let workflow = run_workflow(input)?;
    let summary = &workflow.final_summary;

    let executive_summary = format!(
        "NebulaStore 在当前参数下风险等级为 {}，可靠性分数 {}，时延改善 {}ms，月成本变化 ${}。",
        summary.risk_level,
        summary.reliability_score,
        summary.latency_delta_ms,
        summary.monthly_cost_delta_usd
    );

    let mut key_risks = Vec::new();
    if workflow.penalty.penalized_nodes > 0 {
        key_risks.push(format!(
            "存在 {} 个节点触发惩罚，需重点关注恶意证明和离线行为。",
            workflow.penalty.penalized_nodes
        ));
    }
    if workflow.repair.failed > 0 {
        key_risks.push(format!(
            "修复状态机中仍有 {} 个任务失败，建议提高重试策略与替换节点阈值。",
            workflow.repair.failed
        ));
    }
    if summary.monthly_cost_delta_usd > 0.0 {
        key_risks.push(format!(
            "热层迁移导致月成本上升 ${}，需确认预算可承受。",
            summary.monthly_cost_delta_usd
        ));
    }
    if key_risks.is_empty() {
        key_risks.push("当前参数组合未出现显著高风险项。".to_string());
    }

    let mut recommendations = vec![
        "将挑战失败任务与节点信誉联动，失败超过阈值立即触发替换。".to_string(),
        "对作恶节点执行更长恢复期，并要求连续稳定在线后再恢复权重。".to_string(),
        "按周评估热分片占比，控制时延收益与成本上升的平衡点。".to_string(),
    ];
    if summary.risk_level == "high" {
        recommendations.insert(0, "先降低热分片目标并扩大副本数，再逐步扩容。".to_string());
    }

    Ok(ManagementReport {
        protocol: workflow.protocol,
        executive_summary,
        key_metrics: ReportMetrics {
            reliability_score: summary.reliability_score,
            risk_level: summary.risk_level.clone(),
            latency_delta_ms: summary.latency_delta_ms,
            monthly_cost_delta_usd: summary.monthly_cost_delta_usd,
            penalized_nodes: workflow.penalty.penalized_nodes,
            repair_failed_tasks: workflow.repair.failed,
        },
        key_risks,
        recommendations,
    })
}

// 渲染中文 Markdown 报告，便于直接复制到汇报文档。
pub fn render_management_markdown(report: &ManagementReport) -> String {
    let mut output = String::new();
    output.push_str("# NebulaStore 管理层报告\n\n");
    output.push_str("## 执行摘要\n");
    output.push_str(&format!("- {}\n\n", report.executive_summary));

    output.push_str("## 关键指标\n");
    output.push_str(&format!("- 可靠性分数: {}\n", report.key_metrics.reliability_score));
    output.push_str(&format!("- 风险等级: {}\n", report.key_metrics.risk_level));
    output.push_str(&format!("- 时延改善: {} ms\n", report.key_metrics.latency_delta_ms));
    output.push_str(&format!(
        "- 月成本变化: ${}\n",
        report.key_metrics.monthly_cost_delta_usd
    ));
    output.push_str(&format!("- 惩罚节点数: {}\n", report.key_metrics.penalized_nodes));
    output.push_str(&format!(
        "- 修复失败任务: {}\n\n",
        report.key_metrics.repair_failed_tasks
    ));

    output.push_str("## 主要风险\n");
    for risk in &report.key_risks {
        output.push_str(&format!("- {}\n", risk));
    }
    output.push('\n');

    output.push_str("## 建议动作\n");
    for action in &report.recommendations {
        output.push_str(&format!("- {}\n", action));
    }

    output
}

// 参数自动寻优：在约束内搜索副本数、节点规模和热占比的较优组合。
pub fn optimize_parameters(input: OptimizeInput) -> Result<OptimizeReport, String> {
    if input.size_gb <= 0.0 {
        return Err("size_gb must be > 0".to_string());
    }
    if input.target_p95_ms < 300 || input.target_p95_ms > 3000 {
        return Err("target_p95_ms must be in [300, 3000]".to_string());
    }
    if input.max_monthly_budget_usd <= 0.0 {
        return Err("max_monthly_budget_usd must be > 0".to_string());
    }
    if input.max_risk_level > 100 {
        return Err("max_risk_level must be in [0, 100]".to_string());
    }

    let mut candidates = generate_candidates(input.size_gb, input.max_monthly_budget_usd)?;

    // 过滤不满足硬约束的候选。
    candidates.retain(|c| {
        let predicted_risk_level = 100_u8.saturating_sub(c.predicted_reliability_score);
        c.predicted_p95_ms <= input.target_p95_ms && predicted_risk_level <= input.max_risk_level
    });

    if candidates.is_empty() {
        return Err("no feasible candidate found under given constraints".to_string());
    }

    for c in &mut candidates {
        let latency_gap = (input.target_p95_ms as i32 - c.predicted_p95_ms as i32).max(0) as f64;
        let budget_left = (input.max_monthly_budget_usd - c.predicted_monthly_cost_usd).max(0.0);
        let reliability_bonus = c.predicted_reliability_score as f64;
        let score = latency_gap * 0.45 + budget_left * 0.25 + reliability_bonus * 0.30;
        c.score = (score * 100.0).round() / 100.0;
    }

    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
    let best = candidates[0].clone();
    let top_candidates = candidates.iter().take(5).cloned().collect::<Vec<_>>();

    Ok(OptimizeReport {
        protocol: "NebulaStore".to_string(),
        target_p95_ms: input.target_p95_ms,
        max_monthly_budget_usd: input.max_monthly_budget_usd,
        best,
        top_candidates,
        notes: vec![
            "已在副本数/节点规模/热占比三维空间进行约束搜索。".to_string(),
            "优先满足目标时延与预算，再按可靠性与裕量进行排序。".to_string(),
        ],
    })
}

// 计算 Pareto 前沿：时延、成本越低越好，可靠性越高越好。
pub fn pareto_frontier(input: ParetoInput) -> Result<ParetoReport, String> {
    if input.size_gb <= 0.0 {
        return Err("size_gb must be > 0".to_string());
    }
    if input.max_monthly_budget_usd <= 0.0 {
        return Err("max_monthly_budget_usd must be > 0".to_string());
    }

    let candidates = generate_candidates(input.size_gb, input.max_monthly_budget_usd)?;
    if candidates.is_empty() {
        return Err("no feasible candidate found under budget constraint".to_string());
    }

    let mut frontier = Vec::new();
    for i in 0..candidates.len() {
        let mut dominated = false;
        for j in 0..candidates.len() {
            if i == j {
                continue;
            }
            if dominates(&candidates[j], &candidates[i]) {
                dominated = true;
                break;
            }
        }
        if !dominated {
            frontier.push(candidates[i].clone());
        }
    }

    // 按目标指标去重，避免同质参数重复出现在前沿中。
    frontier.sort_by(|a, b| {
        a.predicted_monthly_cost_usd
            .total_cmp(&b.predicted_monthly_cost_usd)
            .then_with(|| a.predicted_p95_ms.cmp(&b.predicted_p95_ms))
            .then_with(|| b.predicted_reliability_score.cmp(&a.predicted_reliability_score))
    });
    frontier.dedup_by(|a, b| {
        a.predicted_p95_ms == b.predicted_p95_ms
            && (a.predicted_monthly_cost_usd - b.predicted_monthly_cost_usd).abs() < 0.0001
            && a.predicted_reliability_score == b.predicted_reliability_score
    });

    // 前沿内部按成本升序、时延升序，便于阅读。
    frontier.sort_by(|a, b| {
        a.predicted_monthly_cost_usd
            .total_cmp(&b.predicted_monthly_cost_usd)
            .then_with(|| a.predicted_p95_ms.cmp(&b.predicted_p95_ms))
    });

    // 给前沿方案补充可对比分数（用于展示，不参与支配判断）。
    for c in &mut frontier {
        c.score = ((c.predicted_reliability_score as f64) * 0.5
            + (2000.0 - c.predicted_p95_ms as f64).max(0.0) * 0.0003
            + (input.max_monthly_budget_usd - c.predicted_monthly_cost_usd).max(0.0) * 0.02)
            .round();
    }

    Ok(ParetoReport {
        protocol: "NebulaStore".to_string(),
        candidate_count: candidates.len(),
        frontier,
        notes: vec![
            "Pareto 前沿中的每个方案都无法在不牺牲其他目标的情况下被全面超越。".to_string(),
            "目标维度：最小时延、最小成本、最大可靠性。".to_string(),
        ],
    })
}

// 解释 Pareto 前沿，生成标签化决策建议。
pub fn explain_pareto(input: ParetoInput) -> Result<ParetoExplainReport, String> {
    let report = pareto_frontier(input)?;
    let frontier = report.frontier;
    if frontier.is_empty() {
        return Err("pareto frontier is empty".to_string());
    }

    let min_cost = frontier
        .iter()
        .map(|c| c.predicted_monthly_cost_usd)
        .fold(f64::INFINITY, f64::min);
    let max_cost = frontier
        .iter()
        .map(|c| c.predicted_monthly_cost_usd)
        .fold(f64::NEG_INFINITY, f64::max);

    let min_latency = frontier
        .iter()
        .map(|c| c.predicted_p95_ms)
        .min()
        .unwrap_or(0);
    let max_latency = frontier
        .iter()
        .map(|c| c.predicted_p95_ms)
        .max()
        .unwrap_or(0);

    let cost_mid = (min_cost + max_cost) / 2.0;
    let latency_mid = (min_latency + max_latency) / 2;

    let mut profiles = Vec::with_capacity(frontier.len());
    for candidate in frontier {
        let profile_tag = if candidate.predicted_monthly_cost_usd <= cost_mid
            && candidate.predicted_p95_ms >= latency_mid
        {
            "low-cost".to_string()
        } else if candidate.predicted_monthly_cost_usd > cost_mid
            && candidate.predicted_p95_ms < latency_mid
        {
            "low-latency".to_string()
        } else {
            "balanced".to_string()
        };

        let (best_for, recommendation) = match profile_tag.as_str() {
            "low-cost" => (
                "冷数据归档、预算敏感业务".to_string(),
                "优先控制预算，若时延目标收紧可逐步提升 target_hot_percent。".to_string(),
            ),
            "low-latency" => (
                "Web3 前端、交互密集型访问场景".to_string(),
                "优先保障时延，需同步设置成本阈值防止热层扩张过快。".to_string(),
            ),
            _ => (
                "通用业务、阶段性扩容".to_string(),
                "作为默认配置起点，按周观察访问热度再微调冷热占比。".to_string(),
            ),
        };

        profiles.push(ParetoProfile {
            candidate,
            profile_tag,
            best_for,
            recommendation,
        });
    }

    Ok(ParetoExplainReport {
        protocol: "NebulaStore".to_string(),
        summary: "已将 Pareto 前沿方案自动分组为低成本、均衡、低时延三类。".to_string(),
        profiles,
    })
}

// 生成候选参数空间并做预算裁剪，供优化与 Pareto 复用。
fn generate_candidates(size_gb: f64, max_monthly_budget_usd: f64) -> Result<Vec<OptimizeCandidate>, String> {
    let mut candidates = Vec::new();
    let shard_mb = 64_u32;

    for replicas in [5_u8, 7_u8, 9_u8] {
        for node_count in [12_u16, 16_u16, 20_u16, 24_u16] {
            if node_count < replicas as u16 {
                continue;
            }
            for target_hot_percent in [35_u8, 45_u8, 55_u8, 65_u8] {
                let hot_data_percent = target_hot_percent.saturating_sub(10);
                let workflow = run_workflow(WorkflowInput {
                    size_gb,
                    shard_mb,
                    replicas,
                    node_count,
                    hot_data_percent,
                    target_hot_percent,
                    malicious_percent: 5,
                    offline_percent: 10,
                })?;

                let predicted_monthly_cost_usd = workflow.migration.monthly_cost_after_usd;
                if predicted_monthly_cost_usd > max_monthly_budget_usd {
                    continue;
                }

                candidates.push(OptimizeCandidate {
                    replicas,
                    node_count,
                    hot_data_percent,
                    target_hot_percent,
                    predicted_p95_ms: workflow.migration.p95_latency_after_ms,
                    predicted_monthly_cost_usd,
                    predicted_reliability_score: workflow.final_summary.reliability_score,
                    score: 0.0,
                });
            }
        }
    }

    Ok(candidates)
}

// 支配关系判断：在三目标上不劣且至少一项更优。
fn dominates(a: &OptimizeCandidate, b: &OptimizeCandidate) -> bool {
    let non_worse = a.predicted_p95_ms <= b.predicted_p95_ms
        && a.predicted_monthly_cost_usd <= b.predicted_monthly_cost_usd
        && a.predicted_reliability_score >= b.predicted_reliability_score;

    let strictly_better = a.predicted_p95_ms < b.predicted_p95_ms
        || a.predicted_monthly_cost_usd < b.predicted_monthly_cost_usd
        || a.predicted_reliability_score > b.predicted_reliability_score;

    non_worse && strictly_better
}

// 解析业务模板字符串。
pub fn parse_business_template(value: &str) -> Option<BusinessTemplate> {
    match value {
        "archive" => Some(BusinessTemplate::Archive),
        "web3-frontend" => Some(BusinessTemplate::Web3Frontend),
        "ai-inference" => Some(BusinessTemplate::AiInference),
        _ => None,
    }
}

// 按模板生成一键推荐结果。
pub fn recommend_by_template(
    template: BusinessTemplate,
    size_gb: f64,
    budget_override: Option<f64>,
) -> Result<TemplateReport, String> {
    let preset = template_preset(template);
    let budget = budget_override.unwrap_or(preset.budget_usd);

    // 先按模板目标尝试，若无可行解则逐级放宽时延约束，避免模板直接失败。
    let mut applied_target_p95_ms = preset.target_p95_ms;
    let optimize = loop {
        match optimize_parameters(OptimizeInput {
            size_gb,
            target_p95_ms: applied_target_p95_ms,
            max_monthly_budget_usd: budget,
            max_risk_level: preset.max_risk_level,
        }) {
            Ok(result) => break result,
            Err(err)
                if err.contains("no feasible candidate") && applied_target_p95_ms < 1500 =>
            {
                applied_target_p95_ms = (applied_target_p95_ms + 80).min(1500);
            }
            Err(err) => return Err(err),
        }
    };

    let best = &optimize.best;
    let workflow_input = WorkflowInput {
        size_gb,
        shard_mb: 64,
        replicas: best.replicas,
        node_count: best.node_count,
        hot_data_percent: best.hot_data_percent,
        target_hot_percent: best.target_hot_percent,
        malicious_percent: preset.malicious_percent,
        offline_percent: preset.offline_percent,
    };

    let workflow = run_workflow(workflow_input)?;
    let management = build_management_report(workflow_input)?;

    Ok(TemplateReport {
        protocol: "NebulaStore".to_string(),
        template: template_name(template).to_string(),
        description: preset.description.to_string(),
        applied_target_p95_ms,
        applied_budget_usd: budget,
        optimize,
        workflow,
        management,
    })
}

// 渲染模板推荐 Markdown，便于业务汇报和评审。
pub fn render_template_markdown(report: &TemplateReport) -> String {
    let mut out = String::new();
    out.push_str("# NebulaStore 模板推荐报告\n\n");
    out.push_str(&format!("- 模板: `{}`\n", report.template));
    out.push_str(&format!("- 场景说明: {}\n", report.description));
    out.push_str(&format!("- 目标时延: {} ms\n", report.applied_target_p95_ms));
    out.push_str(&format!("- 预算上限: ${}\n\n", report.applied_budget_usd));

    out.push_str("## 推荐参数\n");
    out.push_str(&format!("- 副本数: {}\n", report.optimize.best.replicas));
    out.push_str(&format!("- 节点数: {}\n", report.optimize.best.node_count));
    out.push_str(&format!(
        "- 热占比: {} -> {}\n",
        report.optimize.best.hot_data_percent, report.optimize.best.target_hot_percent
    ));
    out.push_str(&format!(
        "- 预测时延: {} ms\n",
        report.optimize.best.predicted_p95_ms
    ));
    out.push_str(&format!(
        "- 预测成本: ${}\n\n",
        report.optimize.best.predicted_monthly_cost_usd
    ));

    out.push_str("## 风险与建议\n");
    out.push_str(&format!("- 风险等级: {}\n", report.workflow.final_summary.risk_level));
    out.push_str(&format!(
        "- 可靠性分数: {}\n",
        report.workflow.final_summary.reliability_score
    ));
    for advice in &report.management.recommendations {
        out.push_str(&format!("- {}\n", advice));
    }

    out
}

fn template_preset(template: BusinessTemplate) -> TemplatePreset {
    match template {
        BusinessTemplate::Archive => TemplatePreset {
            target_p95_ms: 1400,
            budget_usd: 2600.0,
            max_risk_level: 35,
            malicious_percent: 4,
            offline_percent: 10,
            description: "冷数据归档与长期存证，优先控制成本与稳定性。",
        },
        BusinessTemplate::Web3Frontend => TemplatePreset {
            target_p95_ms: 1020,
            budget_usd: 3400.0,
            max_risk_level: 28,
            malicious_percent: 6,
            offline_percent: 8,
            description: "Web3 前端与高交互访问，优先低时延与可用性。",
        },
        BusinessTemplate::AiInference => TemplatePreset {
            target_p95_ms: 980,
            budget_usd: 4200.0,
            max_risk_level: 22,
            malicious_percent: 7,
            offline_percent: 7,
            description: "AI 推理与高频读取，强调低时延和稳定吞吐。",
        },
    }
}

fn template_name(template: BusinessTemplate) -> &'static str {
    match template {
        BusinessTemplate::Archive => "archive",
        BusinessTemplate::Web3Frontend => "web3-frontend",
        BusinessTemplate::AiInference => "ai-inference",
    }
}

// 输出三模板对比矩阵，便于评审阶段快速比较。
pub fn template_matrix(
    size_gb: f64,
    budget_override: Option<f64>,
) -> Result<TemplateMatrixReport, String> {
    let templates = [
        BusinessTemplate::Archive,
        BusinessTemplate::Web3Frontend,
        BusinessTemplate::AiInference,
    ];

    let mut rows = Vec::with_capacity(templates.len());
    for t in templates {
        let report = recommend_by_template(t, size_gb, budget_override)?;
        let best = &report.optimize.best;
        rows.push(TemplateMatrixRow {
            template: report.template,
            target_p95_ms: report.applied_target_p95_ms,
            budget_usd: report.applied_budget_usd,
            recommended_replicas: best.replicas,
            recommended_node_count: best.node_count,
            recommended_hot_percent: best.target_hot_percent,
            predicted_p95_ms: best.predicted_p95_ms,
            predicted_monthly_cost_usd: best.predicted_monthly_cost_usd,
            reliability_score: report.workflow.final_summary.reliability_score,
            risk_level: report.workflow.final_summary.risk_level,
        });
    }

    let decision_hint = "若预算优先选 archive；若交互体验优先选 web3-frontend；若极低时延优先选 ai-inference。".to_string();

    Ok(TemplateMatrixReport {
        protocol: "NebulaStore".to_string(),
        size_gb,
        rows,
        decision_hint,
    })
}

// 根据模板矩阵计算综合评分并推荐冠军模板。
pub fn template_champion(
    size_gb: f64,
    budget_override: Option<f64>,
    strategy: ChampionStrategy,
) -> Result<TemplateChampionReport, String> {
    let matrix = template_matrix(size_gb, budget_override)?;
    if matrix.rows.is_empty() {
        return Err("template matrix is empty".to_string());
    }

    let max_cost = matrix
        .rows
        .iter()
        .map(|r| r.predicted_monthly_cost_usd)
        .fold(0.0_f64, f64::max);
    let min_cost = matrix
        .rows
        .iter()
        .map(|r| r.predicted_monthly_cost_usd)
        .fold(f64::INFINITY, f64::min);

    let max_latency = matrix
        .rows
        .iter()
        .map(|r| r.predicted_p95_ms)
        .max()
        .unwrap_or(0);
    let min_latency = matrix
        .rows
        .iter()
        .map(|r| r.predicted_p95_ms)
        .min()
        .unwrap_or(0);

    let mut ranking = Vec::with_capacity(matrix.rows.len());
    for row in &matrix.rows {
        // 归一化后综合评分：可靠性 45%，时延 35%，成本 20%。
        let cost_score = if (max_cost - min_cost).abs() < 0.0001 {
            100.0
        } else {
            (1.0 - (row.predicted_monthly_cost_usd - min_cost) / (max_cost - min_cost)) * 100.0
        };
        let latency_score = if max_latency == min_latency {
            100.0
        } else {
            (1.0
                - (row.predicted_p95_ms - min_latency) as f64
                    / (max_latency - min_latency) as f64)
                * 100.0
        };
        let reliability_score = row.reliability_score as f64;
        let composite_score =
            (reliability_score * 0.45 + latency_score * 0.35 + cost_score * 0.20) * 100.0;

        ranking.push(TemplateRankItem {
            template: row.template.clone(),
            composite_score: composite_score.round() / 100.0,
            predicted_p95_ms: row.predicted_p95_ms,
            predicted_monthly_cost_usd: row.predicted_monthly_cost_usd,
            reliability_score: row.reliability_score,
            risk_level: row.risk_level.clone(),
        });
    }

    ranking.sort_by(|a, b| {
        // 先按综合分排序，再根据策略做同分裁决。
        let base = b.composite_score.total_cmp(&a.composite_score);
        if !base.is_eq() {
            return base;
        }
        match strategy {
            ChampionStrategy::Balanced => {
                a.predicted_monthly_cost_usd
                    .total_cmp(&b.predicted_monthly_cost_usd)
                    .then_with(|| a.predicted_p95_ms.cmp(&b.predicted_p95_ms))
                    .then_with(|| b.reliability_score.cmp(&a.reliability_score))
                    .then_with(|| a.template.cmp(&b.template))
            }
            ChampionStrategy::LatencyFirst => {
                a.predicted_p95_ms
                    .cmp(&b.predicted_p95_ms)
                    .then_with(|| a.predicted_monthly_cost_usd.total_cmp(&b.predicted_monthly_cost_usd))
                    .then_with(|| b.reliability_score.cmp(&a.reliability_score))
                    .then_with(|| a.template.cmp(&b.template))
            }
            ChampionStrategy::CostFirst => {
                a.predicted_monthly_cost_usd
                    .total_cmp(&b.predicted_monthly_cost_usd)
                    .then_with(|| a.predicted_p95_ms.cmp(&b.predicted_p95_ms))
                    .then_with(|| b.reliability_score.cmp(&a.reliability_score))
                    .then_with(|| a.template.cmp(&b.template))
            }
            ChampionStrategy::ReliabilityFirst => {
                b.reliability_score
                    .cmp(&a.reliability_score)
                    .then_with(|| a.predicted_p95_ms.cmp(&b.predicted_p95_ms))
                    .then_with(|| a.predicted_monthly_cost_usd.total_cmp(&b.predicted_monthly_cost_usd))
                    .then_with(|| a.template.cmp(&b.template))
            }
        }
    });
    let champion_template = ranking[0].template.clone();
    let champion_score = ranking[0].composite_score;
    let champion_reason = format!(
        "{} 在当前约束下综合得分最高（{:.2}），兼顾时延、成本和可靠性。",
        champion_template, champion_score
    );

    Ok(TemplateChampionReport {
        protocol: "NebulaStore".to_string(),
        size_gb,
        strategy: champion_strategy_name(strategy).to_string(),
        ranking,
        champion_template,
        champion_reason,
    })
}

// 解析冠军裁决策略字符串。
pub fn parse_champion_strategy(value: &str) -> Option<ChampionStrategy> {
    match value {
        "balanced" => Some(ChampionStrategy::Balanced),
        "latency-first" => Some(ChampionStrategy::LatencyFirst),
        "cost-first" => Some(ChampionStrategy::CostFirst),
        "reliability-first" => Some(ChampionStrategy::ReliabilityFirst),
        _ => None,
    }
}

fn champion_strategy_name(strategy: ChampionStrategy) -> &'static str {
    match strategy {
        ChampionStrategy::Balanced => "balanced",
        ChampionStrategy::LatencyFirst => "latency-first",
        ChampionStrategy::CostFirst => "cost-first",
        ChampionStrategy::ReliabilityFirst => "reliability-first",
    }
}
