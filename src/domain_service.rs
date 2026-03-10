use crate::model::{
    generate_content_hash, generate_data_id, ComplianceProfile, DomainRecord, DurabilityProfile,
    EconomicsProfile, RetrievalProfile,
    StateSnapshot,
};
use std::time::{SystemTime, UNIX_EPOCH};

// 注册新域名并写入默认治理策略。
pub fn register(
    snapshot: &mut StateSnapshot,
    owner: &str,
    domain: &str,
    deployment: &str,
    ttl: u64,
) -> Result<String, String> {
    // 输入校验提前失败，避免污染快照状态。
    validate_domain(domain)?;
    validate_ttl(ttl)?;

    if snapshot.domains.contains_key(domain) {
        return Err(format!("domain already exists: {}", domain));
    }

    let now = now_ts();
    // 默认策略体现 2026 场景下的实用型混合架构思路。
    let content_seed = format!("{}|{}|{}|{}", owner, domain, deployment, now);
    let record = DomainRecord {
        data_id: generate_data_id(),
        content_hash: generate_content_hash(&content_seed),
        stored_data_desc: String::new(),
        content_type: String::new(),
        preview_url: String::new(),
        domain: domain.to_string(),
        owner: owner.to_string(),
        deployment: deployment.to_string(),
        created_at: now,
        expires_at: now.saturating_add(ttl),
        last_renewed_at: now,
        retrieval_profile: RetrievalProfile {
            // 小规模数据默认使用更积极的低时延目标。
            target_p95_ms: 120,
            acceleration_layers: vec![
                "edge-cache".to_string(),
                "gateway-cache".to_string(),
                "origin-fallback".to_string(),
            ],
        },
        durability_profile: DurabilityProfile {
            min_replicas: 5,
            erasure_coding: "10+4".to_string(),
            repair_threshold: 3,
            node_clusters: vec![
                "nebula-self-hosted-core".to_string(),
                "nebula-self-hosted-edge".to_string(),
                "nebula-self-hosted-archive".to_string(),
            ],
        },
        economics_profile: EconomicsProfile {
            budget_mode: "hybrid-reserved+spot".to_string(),
            quote_valid_hours: 24,
            rebalance_threshold_percent: 20,
        },
        compliance_profile: ComplianceProfile {
            encryption: "client-side-aes256".to_string(),
            regional_pinning: vec!["ap-east".to_string(), "eu-central".to_string()],
            audit_trail: true,
        },
    };

    snapshot.domains.insert(domain.to_string(), record);
    snapshot.updated_at = now;

    Ok(format!(
        "registered domain={} owner={} deployment={} ttl={}s",
        domain, owner, deployment, ttl
    ))
}

// 解析域名，返回包含治理策略的完整 JSON 视图。
pub fn resolve(snapshot: &StateSnapshot, domain: &str) -> Result<String, String> {
    let now = now_ts();
    let record = snapshot
        .domains
        .get(domain)
        .ok_or_else(|| format!("domain not found: {}", domain))?;

    // 直接根据到期时间计算状态，不额外保存冗余字段。
    let status = if record.expires_at > now {
        "active"
    } else {
        "expired"
    };

    serde_json::to_string_pretty(&serde_json::json!({
        "status": status,
        "data_id": record.data_id,
        "content_hash": record.content_hash,
        "domain": record.domain,
        "owner": record.owner,
        "deployment": record.deployment,
        "created_at": record.created_at,
        "expires_at": record.expires_at,
        "retrieval_profile": record.retrieval_profile,
        "durability_profile": record.durability_profile,
        "economics_profile": record.economics_profile,
        "compliance_profile": record.compliance_profile
    }))
    .map_err(|e| format!("failed to render response json: {}", e))
}

// 列出全部域名记录，便于做控制面巡检。
pub fn list(snapshot: &StateSnapshot) -> Result<String, String> {
    serde_json::to_string_pretty(&snapshot.domains)
        .map_err(|e| format!("failed to render domains: {}", e))
}

// 续期域名，所有者校验通过后延长有效期。
pub fn renew(
    snapshot: &mut StateSnapshot,
    owner: &str,
    domain: &str,
    ttl: u64,
) -> Result<String, String> {
    validate_ttl(ttl)?;
    let now = now_ts();

    let record = snapshot
        .domains
        .get_mut(domain)
        .ok_or_else(|| format!("domain not found: {}", domain))?;

    if record.owner != owner {
        return Err(format!("owner mismatch for domain {}", domain));
    }

    // 如果已过期，从当前时间续；否则在原到期时间基础上顺延。
    let base = record.expires_at.max(now);
    record.expires_at = base.saturating_add(ttl);
    record.last_renewed_at = now;
    snapshot.updated_at = now;

    Ok(format!(
        "renewed domain={} owner={} new_expiry={}",
        domain, owner, record.expires_at
    ))
}

// 删除域名记录，仅允许所有者执行。
pub fn delete(snapshot: &mut StateSnapshot, owner: &str, domain: &str) -> Result<String, String> {
    let record = snapshot
        .domains
        .get(domain)
        .ok_or_else(|| format!("domain not found: {}", domain))?;

    if record.owner != owner {
        return Err(format!("owner mismatch for domain {}", domain));
    }

    snapshot.domains.remove(domain);
    snapshot.updated_at = now_ts();
    Ok(format!("deleted domain={} owner={}", domain, owner))
}

// 域名规则做最小可用校验，防止明显非法输入。
fn validate_domain(domain: &str) -> Result<(), String> {
    if domain.len() < 3 || domain.len() > 253 {
        return Err("domain length must be between 3 and 253".to_string());
    }
    if !domain.contains('.') {
        return Err("domain must contain at least one dot".to_string());
    }
    if domain
        .chars()
        .any(|ch| !(ch.is_ascii_alphanumeric() || ch == '.' || ch == '-'))
    {
        return Err("domain can only contain letters, digits, '.', '-'".to_string());
    }
    Ok(())
}

// TTL 限制在合理范围内，避免过短抖动和过长僵尸配置。
fn validate_ttl(ttl: u64) -> Result<(), String> {
    if !(60..=31_536_000).contains(&ttl) {
        return Err("ttl must be in [60, 31536000] seconds".to_string());
    }
    Ok(())
}

// 统一时间戳来源，便于后续替换为可注入时钟。
fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
