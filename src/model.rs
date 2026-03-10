use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static DATA_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

// 状态快照是控制面的核心持久化结构。
// 使用版本号便于后续做结构迁移。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub version: u32,
    pub updated_at: u64,
    pub domains: BTreeMap<String, DomainRecord>,
}

impl Default for StateSnapshot {
    // 默认状态用于首次启动时初始化空快照。
    fn default() -> Self {
        Self {
            version: 1,
            updated_at: 0,
            domains: BTreeMap::new(),
        }
    }
}

// 域名记录承载业务元数据和治理策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecord {
    #[serde(default)]
    pub data_id: String,
    #[serde(default)]
    pub content_hash: String,
    #[serde(default)]
    pub stored_data_desc: String,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub preview_url: String,
    pub domain: String,
    pub owner: String,
    pub deployment: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub last_renewed_at: u64,
    pub retrieval_profile: RetrievalProfile,
    pub durability_profile: DurabilityProfile,
    pub economics_profile: EconomicsProfile,
    pub compliance_profile: ComplianceProfile,
}

// 检索策略描述热路径性能目标与加速层配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalProfile {
    pub target_p95_ms: u64,
    pub acceleration_layers: Vec<String>,
}

// 耐久策略描述副本、纠删码和修复阈值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurabilityProfile {
    pub min_replicas: u8,
    pub erasure_coding: String,
    pub repair_threshold: u8,
    #[serde(default)]
    pub node_clusters: Vec<String>,
}

// 经济策略用于控制成本波动与预算行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicsProfile {
    pub budget_mode: String,
    pub quote_valid_hours: u32,
    pub rebalance_threshold_percent: u8,
}

// 合规策略用于定义加密、数据主权和审计能力。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceProfile {
    pub encryption: String,
    pub regional_pinning: Vec<String>,
    pub audit_trail: bool,
}

// 生成全局唯一、近似按时间递增的数据ID。
// 格式：nebula-{timestamp_ms_hex}{sequence_hex}{entropy_hex}
pub fn generate_data_id() -> String {
    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let seq = DATA_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id() as u64;
    let entropy = ts_ms.rotate_left(13) ^ seq.rotate_left(7) ^ pid;

    format!(
        "nebula-{:013x}{:06x}{:06x}",
        ts_ms,
        seq & 0xFF_FFFF,
        entropy & 0xFF_FFFF
    )
}

// 生成内容摘要（SHA-256 十六进制）。
pub fn generate_content_hash(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    finalize_digest(hasher)
}

// 生成二进制内容摘要（SHA-256 十六进制）。
pub fn generate_content_hash_bytes(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    finalize_digest(hasher)
}

fn finalize_digest(hasher: Sha256) -> String {
    let digest = hasher.finalize();

    digest
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}
