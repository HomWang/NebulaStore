use crate::model::{generate_content_hash, generate_data_id, StateSnapshot};
use std::{fs, path::Path};

// 读取状态文件；若文件不存在则返回默认快照。
pub fn load_or_default(path: &Path) -> Result<StateSnapshot, String> {
    if !path.exists() {
        return Ok(StateSnapshot::default());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read state file {}: {}", path.display(), e))?;
    let mut snapshot = serde_json::from_str::<StateSnapshot>(&content)
        .map_err(|e| format!("invalid snapshot json in {}: {}", path.display(), e))?;

    // 兼容历史快照：若缺失 data_id，则自动补齐并回写。
    let mut migrated = false;
    for record in snapshot.domains.values_mut() {
        if record.data_id.is_empty() {
            record.data_id = generate_data_id();
            migrated = true;
        }
        if record.content_hash.is_empty() {
            let seed = format!(
                "{}|{}|{}|{}",
                record.owner, record.domain, record.deployment, record.created_at
            );
            record.content_hash = generate_content_hash(&seed);
            migrated = true;
        }
        if record.content_type.is_empty() {
            record.content_type = "application/json".to_string();
            migrated = true;
        }
        if record.preview_url.is_empty() {
            record.preview_url = String::new();
            migrated = true;
        }
    }
    if migrated {
        save(path, &snapshot)?;
    }

    Ok(snapshot)
}

// 将快照序列化并写入磁盘，保证目录存在。
pub fn save(path: &Path, snapshot: &StateSnapshot) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create state directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    let data = serde_json::to_string_pretty(snapshot)
        .map_err(|e| format!("failed to serialize snapshot: {}", e))?;
    fs::write(path, data)
        .map_err(|e| format!("failed to write state file {}: {}", path.display(), e))
}
