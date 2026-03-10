use crate::{domain_service, model, protocol, state_store};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::{HashMap, HashSet}, path::PathBuf, sync::Arc};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

// API 共享状态：仅保存状态文件路径和互斥锁。
// 所有请求按需加载/保存快照，保证 CLI 与 HTTP 共用同一数据文件。
#[derive(Clone)]
pub struct ApiState {
    pub snapshot_path: Arc<PathBuf>,
    pub lock: Arc<Mutex<()>>,
    pub node_id: Arc<String>,
    pub peers: Arc<Vec<String>>,
    pub upload_sessions: Arc<Mutex<HashMap<String, ChunkUploadSession>>>,
    pub upload_tmp_dir: Arc<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ChunkUploadSession {
    pub owner: String,
    pub data_name: String,
    pub file_name: String,
    pub content_type: Option<String>,
    pub total_chunks: u32,
    pub received_chunks: HashSet<u32>,
    pub temp_path: PathBuf,
}

// 创建 API 路由。
pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/p2p/state", get(p2p_state))
        .route("/api/p2p/snapshot", get(p2p_snapshot))
        .route("/api/p2p/consensus", get(p2p_consensus))
        .route("/api/p2p/vote", post(p2p_vote_snapshot))
        .route("/api/p2p/commit", post(p2p_commit_snapshot))
        .route("/api/chain/blocks", get(list_chain_blocks))
        .route("/api/data/upload", post(upload_data_object))
        .route("/api/data/upload/file", post(upload_data_file))
        .route("/api/data/upload/chunk/init", post(init_chunk_upload))
        .route("/api/data/upload/chunk/part", post(upload_chunk_part))
        .route("/api/data/upload/chunk/complete", post(complete_chunk_upload))
        .route("/api/data/objects", get(list_data_objects))
        .route("/api/data/objects/:data_id", get(get_data_object))
        .route("/api/data/objects/by-hash/:content_hash", get(get_data_object_by_hash))
        .route("/api/domains", get(list_domains))
        .route("/api/domains/:domain", get(resolve_domain).delete(delete_domain))
        .route("/api/domains/register", post(register_domain))
        .route("/api/domains/renew", post(renew_domain))
        .route("/api/protocol/blueprint", get(protocol_blueprint))
        .route("/api/protocol/simulate", post(protocol_simulate))
        .route("/api/protocol/plan", post(protocol_plan))
        .route("/api/protocol/repair", post(protocol_repair))
        .route("/api/protocol/penalty", post(protocol_penalty))
        .route("/api/protocol/migrate", post(protocol_migrate))
        .route("/api/protocol/workflow", post(protocol_workflow))
        .route("/api/protocol/report", post(protocol_report))
        .route("/api/protocol/optimize", post(protocol_optimize))
        .route("/api/protocol/pareto", post(protocol_pareto))
        .route("/api/protocol/pareto/explain", post(protocol_pareto_explain))
        .route("/api/protocol/template", post(protocol_template))
        .route("/api/protocol/template/matrix", post(protocol_template_matrix))
        .route("/api/protocol/template/champion", post(protocol_template_champion))
        .with_state(state)
}

// 区块信息视图（演示用聚合模型）。
#[derive(Debug, Serialize)]
struct ChainBlockView {
    height: u64,
    block_id: String,
    timestamp: u64,
    tx_count: u32,
    data_gb: f64,
    status: String,
}

// 数据对象信息视图（从域名记录映射）。
#[derive(Debug, Serialize)]
struct DataObjectView {
    data_id: String,
    content_hash: String,
    content_type: String,
    preview_url: Option<String>,
    file_name: Option<String>,
    file_size: Option<String>,
    file_type: Option<String>,
    file_preview: Option<String>,
    owner: String,
    deployment: String,
    stored_data: String,
    created_at: u64,
    expires_at: u64,
    target_p95_ms: u64,
    min_replicas: u8,
    node_clusters: Vec<String>,
}

#[derive(Debug, Clone)]
struct FileDataMeta {
    file_name: String,
    file_size: String,
    file_type: String,
    file_preview: String,
}

#[derive(Debug, Deserialize)]
// 数据上传请求体。
struct DataUploadRequest {
    owner: String,
    data_name: String,
    content: String,
    content_type: Option<String>,
    preview_url: Option<String>,
}

#[derive(Debug, Deserialize)]
// 分片上传初始化请求。
struct ChunkUploadInitRequest {
    owner: String,
    data_name: String,
    file_name: String,
    total_chunks: u32,
    content_type: Option<String>,
}

#[derive(Debug, Deserialize)]
// 分片上传单片请求。
struct ChunkUploadPartRequest {
    upload_id: String,
    chunk_index: u32,
    chunk_base64: String,
}

#[derive(Debug, Deserialize)]
// 分片上传完成请求。
struct ChunkUploadCompleteRequest {
    upload_id: String,
}

#[derive(Debug, Serialize)]
struct P2pStateView {
    node_id: String,
    updated_at: u64,
    domain_count: usize,
    snapshot_hash: String,
}

#[derive(Debug, Deserialize)]
struct P2pVoteRequest {
    proposer_node_id: String,
    snapshot_hash: String,
    updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct P2pVoteResponse {
    node_id: String,
    vote: bool,
    reason: String,
    local_hash: String,
}

#[derive(Debug, Deserialize)]
struct P2pCommitRequest {
    proposer_node_id: String,
    snapshot: crate::model::StateSnapshot,
}

// 健康检查接口。
async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

// P2P 节点摘要：用于共识轮次快速比对。
async fn p2p_state(State(state): State<ApiState>) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    let snapshot_hash = compute_snapshot_hash(&snapshot);
    (
        StatusCode::OK,
        Json(json!(P2pStateView {
            node_id: state.node_id.as_ref().clone(),
            updated_at: snapshot.updated_at,
            domain_count: snapshot.domains.len(),
            snapshot_hash,
        })),
    )
        .into_response()
}

// P2P 快照导出：供其他节点拉取并对齐状态。
async fn p2p_snapshot(State(state): State<ApiState>) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(json!(snapshot))).into_response()
}

// 共识状态可视化：用于观察当前节点连了哪些 peer。
async fn p2p_consensus(State(state): State<ApiState>) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "node_id": state.node_id.as_ref(),
            "peer_count": state.peers.len(),
            "peers": state.peers.as_ref(),
            "snapshot_hash": compute_snapshot_hash(&snapshot),
            "updated_at": snapshot.updated_at,
            "domain_count": snapshot.domains.len(),
        })),
    )
        .into_response()
}

// 写入前投票：仅检查是否可接受该版本，不直接落盘。
async fn p2p_vote_snapshot(
    State(state): State<ApiState>,
    Json(req): Json<P2pVoteRequest>,
) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    let local_hash = compute_snapshot_hash(&snapshot);
    let vote = req.updated_at > snapshot.updated_at
        || (req.updated_at == snapshot.updated_at && req.snapshot_hash >= local_hash);
    let reason = if vote {
        format!("accepted from {}", req.proposer_node_id)
    } else {
        "stale proposal".to_string()
    };

    (
        StatusCode::OK,
        Json(json!(P2pVoteResponse {
            node_id: state.node_id.as_ref().clone(),
            vote,
            reason,
            local_hash,
        })),
    )
        .into_response()
}

// 投票通过后提交：若提案快照更新，则落盘并加入共识收敛。
async fn p2p_commit_snapshot(
    State(state): State<ApiState>,
    Json(req): Json<P2pCommitRequest>,
) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let local_snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    let local_hash = compute_snapshot_hash(&local_snapshot);
    let incoming_hash = compute_snapshot_hash(&req.snapshot);
    let apply = should_adopt_snapshot(&local_snapshot, &req.snapshot);

    if apply {
        if let Err(err) = state_store::save(state.snapshot_path.as_ref(), &req.snapshot) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "node_id": state.node_id.as_ref(),
            "proposer_node_id": req.proposer_node_id,
            "applied": apply,
            "local_hash_before": local_hash,
            "incoming_hash": incoming_hash,
        })),
    )
        .into_response()
}

fn compute_snapshot_hash(snapshot: &crate::model::StateSnapshot) -> String {
    let bytes = serde_json::to_vec(snapshot).unwrap_or_default();
    let digest = Sha256::digest(&bytes);
    digest.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

// 返回最近区块信息（演示：由快照数据聚合构造）。
async fn list_chain_blocks(State(state): State<ApiState>) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    let mut blocks = Vec::new();

    // 固定返回创世区块，区块高度从 0 开始。
    blocks.push(ChainBlockView {
        height: 0,
        block_id: "nebula-genesis-00000000".to_string(),
        timestamp: snapshot.updated_at,
        tx_count: 1,
        data_gb: 0.0,
        status: "finalized".to_string(),
    });

    let object_count = snapshot.domains.len();
    let total_height = object_count as u64;
    for (idx, record) in snapshot.domains.values().rev().take(19).enumerate() {
        let h = total_height.saturating_sub(idx as u64);
        let data_gb = estimate_record_size_gb(record.stored_data_desc.as_str());

        blocks.push(ChainBlockView {
            height: h,
            block_id: format!("nebula-{:08x}", h),
            timestamp: record.last_renewed_at.max(record.created_at),
            tx_count: (record.durability_profile.min_replicas as u32).saturating_mul(3),
            data_gb: (data_gb * 100.0).round() / 100.0,
            status: "finalized".to_string(),
        });
    }

    (
        StatusCode::OK,
        Json(json!({"protocol": "NebulaStore", "total_height": total_height, "blocks": blocks})),
    )
        .into_response()
}

// 返回数据对象列表（演示：域名记录视角）。
async fn list_data_objects(State(state): State<ApiState>) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    let objects = build_data_objects(&snapshot);

    (StatusCode::OK, Json(json!({"protocol": "NebulaStore", "objects": objects}))).into_response()
}

// 上传数据对象并写入快照。
async fn upload_data_object(
    State(state): State<ApiState>,
    Json(req): Json<DataUploadRequest>,
) -> impl IntoResponse {
    if req.owner.trim().is_empty() || req.data_name.trim().is_empty() || req.content.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "owner/data_name/content must not be empty"})),
        )
            .into_response();
    }

    match persist_uploaded_object(
        &state,
        req.owner.trim(),
        req.data_name.trim(),
        model::generate_content_hash(&req.content),
        req.content_type.as_deref().unwrap_or("application/json"),
        req.preview_url
            .as_deref()
            .map(str::trim)
            .filter(|u| !u.is_empty()),
        summarize_uploaded_data(&req.content, req.content_type.as_deref()),
    )
    .await
    {
        Ok(obj) => (StatusCode::OK, Json(json!({"message": "upload accepted", "object": obj}))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err})),
        )
            .into_response(),
    }
}

// 文件上传（支持表单与拖拽上传）。
async fn upload_data_file(State(state): State<ApiState>, mut multipart: Multipart) -> impl IntoResponse {
    let mut owner = String::new();
    let mut data_name = String::new();
    let mut content_type: Option<String> = None;
    let mut file_name = String::from("upload.bin");
    let mut file_bytes: Vec<u8> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "owner" => {
                owner = field.text().await.unwrap_or_default();
            }
            "data_name" => {
                data_name = field.text().await.unwrap_or_default();
            }
            "content_type" => {
                let value = field.text().await.unwrap_or_default();
                if !value.trim().is_empty() {
                    content_type = Some(value);
                }
            }
            "file" => {
                file_name = field
                    .file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "upload.bin".to_string());
                match field.bytes().await {
                    Ok(bytes) => file_bytes = bytes.to_vec(),
                    Err(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": "invalid multipart file field"})),
                        )
                            .into_response();
                    }
                }
            }
            _ => {}
        }
    }

    if owner.trim().is_empty() || data_name.trim().is_empty() || file_bytes.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "owner/data_name/file must not be empty"})),
        )
            .into_response();
    }

    let desc = summarize_file_data(&file_name, file_bytes.len(), content_type.as_deref(), &file_bytes);

    match persist_uploaded_object(
        &state,
        owner.trim(),
        data_name.trim(),
        model::generate_content_hash_bytes(&file_bytes),
        content_type.as_deref().unwrap_or("application/octet-stream"),
        build_preview_url_for_media(content_type.as_deref(), &file_bytes).as_deref(),
        desc,
    )
    .await
    {
        Ok(obj) => (StatusCode::OK, Json(json!({"message": "file upload accepted", "object": obj}))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err})),
        )
            .into_response(),
    }
}

// 初始化分片上传会话，返回 upload_id。
async fn init_chunk_upload(
    State(state): State<ApiState>,
    Json(req): Json<ChunkUploadInitRequest>,
) -> impl IntoResponse {
    if req.owner.trim().is_empty() || req.data_name.trim().is_empty() || req.file_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "owner/data_name/file_name must not be empty"})),
        )
            .into_response();
    }

    if req.total_chunks == 0 || req.total_chunks > 100_000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "total_chunks must be in 1..=100000"})),
        )
            .into_response();
    }

    let upload_id = format!("upload-{}", model::generate_data_id());
    let temp_path = state.upload_tmp_dir.join(format!("{}.part", upload_id));
    if let Err(err) = tokio::fs::write(&temp_path, &[]).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("failed to init temp file: {}", err)})),
        )
            .into_response();
    }

    let mut sessions = state.upload_sessions.lock().await;
    sessions.insert(
        upload_id.clone(),
        ChunkUploadSession {
            owner: req.owner.trim().to_string(),
            data_name: req.data_name.trim().to_string(),
            file_name: req.file_name.trim().to_string(),
            content_type: req.content_type,
            total_chunks: req.total_chunks,
            received_chunks: HashSet::new(),
            temp_path,
        },
    );

    (StatusCode::OK, Json(json!({"upload_id": upload_id, "total_chunks": req.total_chunks}))).into_response()
}

// 上传分片，按 upload_id 和 chunk_index 顺序写入临时文件。
async fn upload_chunk_part(
    State(state): State<ApiState>,
    Json(req): Json<ChunkUploadPartRequest>,
) -> impl IntoResponse {
    let mut sessions = state.upload_sessions.lock().await;
    let Some(session) = sessions.get_mut(req.upload_id.trim()) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "upload session not found"})),
        )
            .into_response();
    };

    if req.chunk_index >= session.total_chunks {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "chunk_index out of range"})),
        )
            .into_response();
    }

    if session.received_chunks.contains(&req.chunk_index) {
        return (
            StatusCode::OK,
            Json(json!({"message": "chunk already uploaded", "received": session.received_chunks.len(), "total": session.total_chunks})),
        )
            .into_response();
    }

    let expected = session.received_chunks.len() as u32;
    if req.chunk_index != expected {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("chunk order mismatch, expected {}", expected)})),
        )
            .into_response();
    }

    let chunk_bytes = match STANDARD.decode(req.chunk_base64.as_bytes()) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "chunk_base64 decode failed"})),
            )
                .into_response();
        }
    };

    let mut file = match tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&session.temp_path)
        .await
    {
        Ok(f) => f,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("open temp file failed: {}", err)})),
            )
                .into_response();
        }
    };

    if let Err(err) = file.write_all(&chunk_bytes).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("write chunk failed: {}", err)})),
        )
            .into_response();
    }

    session.received_chunks.insert(req.chunk_index);

    (StatusCode::OK, Json(json!({"message": "chunk accepted", "received": session.received_chunks.len(), "total": session.total_chunks}))).into_response()
}

// 完成分片上传并落地为对象记录。
async fn complete_chunk_upload(
    State(state): State<ApiState>,
    Json(req): Json<ChunkUploadCompleteRequest>,
) -> impl IntoResponse {
    let session = {
        let mut sessions = state.upload_sessions.lock().await;
        match sessions.remove(req.upload_id.trim()) {
            Some(s) => s,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "upload session not found"})),
                )
                    .into_response();
            }
        }
    };

    if session.received_chunks.len() != session.total_chunks as usize {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "chunks not complete",
                "received": session.received_chunks.len(),
                "total": session.total_chunks
            })),
        )
            .into_response();
    }

    let bytes = match tokio::fs::read(&session.temp_path).await {
        Ok(v) => v,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("read temp file failed: {}", err)})),
            )
                .into_response();
        }
    };

    let desc = summarize_file_data(
        &session.file_name,
        bytes.len(),
        session.content_type.as_deref(),
        &bytes,
    );

    let result = persist_uploaded_object(
        &state,
        &session.owner,
        &session.data_name,
        model::generate_content_hash_bytes(&bytes),
        session
            .content_type
            .as_deref()
            .unwrap_or("application/octet-stream"),
        build_preview_url_for_media(session.content_type.as_deref(), &bytes).as_deref(),
        desc,
    )
    .await;

    let _ = tokio::fs::remove_file(&session.temp_path).await;

    match result {
        Ok(obj) => (StatusCode::OK, Json(json!({"message": "chunk upload completed", "object": obj}))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err})),
        )
            .into_response(),
    }
}

// 按 data_id 查询单个数据对象。
async fn get_data_object(
    State(state): State<ApiState>,
    Path(data_id): Path<String>,
) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    let objects = build_data_objects(&snapshot);
    match objects.into_iter().find(|obj| obj.data_id == data_id) {
        Some(obj) => (StatusCode::OK, Json(json!(obj))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("data object not found: {}", data_id)})),
        )
            .into_response(),
    }
}

// 按 content_hash 查询单个数据对象。
async fn get_data_object_by_hash(
    State(state): State<ApiState>,
    Path(content_hash): Path<String>,
) -> impl IntoResponse {
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    let objects = build_data_objects(&snapshot);
    match objects.into_iter().find(|obj| obj.content_hash == content_hash) {
        Some(obj) => (StatusCode::OK, Json(json!(obj))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("content hash not found: {}", content_hash)})),
        )
            .into_response(),
    }
}

// 统一构造对象列表，保证列表与按 ID 查询的一致性。
fn build_data_objects(snapshot: &crate::model::StateSnapshot) -> Vec<DataObjectView> {
    let object_count = snapshot.domains.len();

    snapshot
        .domains
        .values()
        .map(|record| {
            let parsed_meta = parse_file_summary(&record.stored_data_desc);
            let normalized_preview = if is_media_content_type(&record.content_type) && !record.preview_url.is_empty() {
                Some("可预览媒体（详情中查看）".to_string())
            } else {
                parsed_meta.as_ref().map(|m| m.file_preview.clone())
            };

            DataObjectView {
            data_id: record.data_id.clone(),
            content_hash: record.content_hash.clone(),
            content_type: if record.content_type.is_empty() {
                "application/json".to_string()
            } else {
                record.content_type.clone()
            },
            preview_url: if record.preview_url.is_empty() {
                None
            } else {
                Some(record.preview_url.clone())
            },
            file_name: parsed_meta.as_ref().map(|m| m.file_name.clone()),
            file_size: parsed_meta.as_ref().map(|m| m.file_size.clone()),
            file_type: parsed_meta.as_ref().map(|m| m.file_type.clone()),
            file_preview: normalized_preview,
            owner: record.owner.clone(),
            deployment: record.deployment.clone(),
            stored_data: if record.stored_data_desc.is_empty() {
                infer_stored_data(&record.domain, &record.deployment)
            } else {
                normalize_stored_data_content(&record.stored_data_desc)
            },
            created_at: record.created_at,
            expires_at: record.expires_at,
            target_p95_ms: tuned_p95_ms(record.retrieval_profile.target_p95_ms, object_count),
            min_replicas: record.durability_profile.min_replicas,
            node_clusters: record.durability_profile.node_clusters.clone(),
        }
        })
        .collect()
}

// 上传内容摘要：用于数据信息页展示“具体存储了什么”。
fn summarize_uploaded_data(content: &str, content_type: Option<&str>) -> String {
    let _ = content_type;
    // 文本/JSON 上传场景中，直接保存原始内容，避免前端查询时出现“类型+摘要”包装文本。
    content.to_string()
}

fn normalize_stored_data_content(raw: &str) -> String {
    // 兼容旧数据："application/json | 内容摘要: {...}" -> "{...}"
    if let Some((_, summary)) = raw.split_once(" | 内容摘要: ") {
        return summary.to_string();
    }
    raw.to_string()
}

// 文件摘要：用于前端展示文件名、大小和可读预览。
fn summarize_file_data(file_name: &str, size_bytes: usize, content_type: Option<&str>, bytes: &[u8]) -> String {
    let max_len = 72usize;

    let type_label = content_type
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("application/octet-stream");
    let human_size = format_bytes_human(size_bytes as u64);

    if is_textual_content_type(type_label) {
        let text = String::from_utf8_lossy(bytes);
        let compact = text.replace('\n', " ");
        let preview = if compact.chars().count() > max_len {
            format!("{}...", compact.chars().take(max_len).collect::<String>())
        } else {
            compact
        };

        format!(
            "文件: {} | 大小: {} | 类型: {} | 文本预览: {}",
            file_name, human_size, type_label, preview
        )
    } else {
        let preview_hint = if is_media_content_type(type_label) {
            "可预览媒体（详情中查看）"
        } else {
            "二进制内容不展示文本预览"
        };

        format!(
            "文件: {} | 大小: {} | 类型: {} | {}",
            file_name, human_size, type_label, preview_hint
        )
    }
}

fn format_bytes_human(size_bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = size_bytes as f64;
    let mut idx = 0usize;

    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }

    if idx == 0 {
        format!("{} {}", size_bytes, UNITS[idx])
    } else {
        format!("{:.2} {}", size, UNITS[idx])
    }
}

fn estimate_record_size_gb(stored_data_desc: &str) -> f64 {
    let Some(meta) = parse_file_summary(stored_data_desc) else {
        return 0.0;
    };

    let Some(size_bytes) = parse_size_label_to_bytes(meta.file_size.as_str()) else {
        return 0.0;
    };

    size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

fn parse_size_label_to_bytes(raw: &str) -> Option<u64> {
    let size = raw.trim();
    if size.is_empty() {
        return None;
    }

    let upper = size.to_ascii_uppercase();
    let mut parts = upper.split_whitespace();
    let value_text = parts.next()?;
    let unit = parts.next().unwrap_or("B");

    let value = value_text.parse::<f64>().ok()?;
    if value < 0.0 {
        return None;
    }

    let multiplier = match unit {
        "B" => 1.0,
        "KB" => 1024.0,
        "MB" => 1024.0 * 1024.0,
        "GB" => 1024.0 * 1024.0 * 1024.0,
        "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    Some((value * multiplier).round() as u64)
}

fn parse_file_summary(summary: &str) -> Option<FileDataMeta> {
    if !summary.starts_with("文件: ") {
        return None;
    }

    let mut file_name = String::new();
    let mut file_size = String::new();
    let mut file_type = String::new();
    let mut file_preview = String::new();

    for part in summary.split(" | ") {
        if let Some(v) = part.strip_prefix("文件: ") {
            file_name = v.trim().to_string();
        } else if let Some(v) = part.strip_prefix("大小: ") {
            file_size = normalize_size_label(v.trim());
        } else if let Some(v) = part.strip_prefix("类型: ") {
            file_type = v.trim().to_string();
        } else if let Some(v) = part.strip_prefix("文本预览: ") {
            file_preview = v.trim().to_string();
        } else if part.contains("二进制内容不展示文本预览") {
            file_preview = "二进制内容不展示文本预览".to_string();
        }
    }

    if file_name.is_empty() && file_size.is_empty() && file_type.is_empty() {
        return None;
    }

    Some(FileDataMeta {
        file_name,
        file_size,
        file_type,
        file_preview,
    })
}

fn normalize_size_label(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    if let Some(num) = lower.strip_suffix(" bytes") {
        if let Ok(bytes) = num.trim().parse::<u64>() {
            return format_bytes_human(bytes);
        }
    }
    raw.to_string()
}

fn is_textual_content_type(content_type: &str) -> bool {
    let ct = content_type.to_ascii_lowercase();
    ct.starts_with("text/")
        || ct.contains("json")
        || ct.contains("xml")
        || ct.contains("yaml")
        || ct.contains("javascript")
        || ct.contains("typescript")
}

fn is_media_content_type(content_type: &str) -> bool {
    let ct = content_type.to_ascii_lowercase();
    ct.starts_with("image/") || ct.starts_with("video/")
}

fn build_preview_url_for_media(content_type: Option<&str>, bytes: &[u8]) -> Option<String> {
    let ct = content_type?.trim().to_ascii_lowercase();
    if !(ct.starts_with("image/") || ct.starts_with("video/")) {
        return None;
    }

    // 限制内联预览大小，避免 snapshot 过大导致接口响应变慢。
    if bytes.len() > 8 * 1024 * 1024 {
        return None;
    }

    let b64 = STANDARD.encode(bytes);
    Some(format!("data:{};base64,{}", ct, b64))
}

// 将上传对象持久化到快照，返回统一对象视图。
async fn persist_uploaded_object(
    state: &ApiState,
    owner: &str,
    data_name: &str,
    content_hash: String,
    content_type: &str,
    preview_url: Option<&str>,
    stored_data_desc: String,
) -> Result<DataObjectView, String> {
    let _guard = state.lock.lock().await;
    let mut snapshot = state_store::load_or_default(state.snapshot_path.as_ref())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let data_id = model::generate_data_id();
    let domain = format!(
        "{}.data.nebula.local",
        &data_id[data_id.len().saturating_sub(12)..]
    );

    let record = model::DomainRecord {
        data_id: data_id.clone(),
        content_hash,
        stored_data_desc,
        content_type: content_type.to_string(),
        preview_url: preview_url.unwrap_or_default().to_string(),
        domain: domain.clone(),
        owner: owner.to_string(),
        deployment: data_name.to_string(),
        created_at: now,
        expires_at: now.saturating_add(31_536_000),
        last_renewed_at: now,
        retrieval_profile: model::RetrievalProfile {
            target_p95_ms: 120,
            acceleration_layers: vec![
                "edge-cache".to_string(),
                "gateway-cache".to_string(),
                "origin-fallback".to_string(),
            ],
        },
        durability_profile: model::DurabilityProfile {
            min_replicas: 5,
            erasure_coding: "10+4".to_string(),
            repair_threshold: 3,
            node_clusters: vec![
                "nebula-self-hosted-core".to_string(),
                "nebula-self-hosted-edge".to_string(),
                "nebula-self-hosted-archive".to_string(),
            ],
        },
        economics_profile: model::EconomicsProfile {
            budget_mode: "hybrid-reserved+spot".to_string(),
            quote_valid_hours: 24,
            rebalance_threshold_percent: 20,
        },
        compliance_profile: model::ComplianceProfile {
            encryption: "client-side-aes256".to_string(),
            regional_pinning: vec!["ap-east".to_string(), "eu-central".to_string()],
            audit_trail: true,
        },
    };

    snapshot.domains.insert(domain, record);
    snapshot.updated_at = now;

    ensure_write_quorum_votes(state, &snapshot).await?;
    state_store::save(state.snapshot_path.as_ref(), &snapshot)?;
    broadcast_snapshot_commit(state, &snapshot).await;

    Ok(build_data_objects(&snapshot)
        .into_iter()
        .find(|o| o.data_id == data_id)
        .unwrap_or_else(|| DataObjectView {
            data_id,
            content_hash: String::new(),
            content_type: content_type.to_string(),
            preview_url: preview_url.map(|u| u.to_string()),
            file_name: None,
            file_size: None,
            file_type: None,
            file_preview: None,
            owner: owner.to_string(),
            deployment: data_name.to_string(),
            stored_data: "上传成功".to_string(),
            created_at: now,
            expires_at: now,
            target_p95_ms: 120,
            min_replicas: 5,
            node_clusters: vec!["nebula-self-hosted-core".to_string()],
        }))
}

async fn ensure_write_quorum_votes(
    state: &ApiState,
    candidate_snapshot: &crate::model::StateSnapshot,
) -> Result<(), String> {
    if state.peers.is_empty() {
        return Ok(());
    }

    let total_nodes = state.peers.len() + 1;
    let required = total_nodes / 2 + 1;
    let mut accepted = 1usize;
    let candidate_hash = compute_snapshot_hash(candidate_snapshot);

    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| format!("build quorum http client failed: {}", e))?;

    for peer in state.peers.iter() {
        let vote_url = format!("{}/api/p2p/vote", peer);
        let response = match client
            .post(&vote_url)
            .json(&json!({
                "proposer_node_id": state.node_id.as_ref(),
                "snapshot_hash": candidate_hash,
                "updated_at": candidate_snapshot.updated_at,
            }))
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => continue,
        };
        if !response.status().is_success() {
            continue;
        }

        let vote = match response.json::<P2pVoteResponse>().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        if vote.vote {
            accepted += 1;
        }
    }

    if accepted < required {
        return Err(format!(
            "write quorum not reached: accepted={} required={} total_nodes={}",
            accepted, required, total_nodes
        ));
    }

    Ok(())
}

async fn broadcast_snapshot_commit(state: &ApiState, snapshot: &crate::model::StateSnapshot) {
    if state.peers.is_empty() {
        return;
    }

    let client = match Client::builder().timeout(Duration::from_secs(5)).build() {
        Ok(c) => c,
        Err(_) => return,
    };

    for peer in state.peers.iter() {
        let commit_url = format!("{}/api/p2p/commit", peer);
        let _ = client
            .post(&commit_url)
            .json(&json!({
                "proposer_node_id": state.node_id.as_ref(),
                "snapshot": snapshot,
            }))
            .send()
            .await;
    }
}

fn should_adopt_snapshot(
    local: &crate::model::StateSnapshot,
    incoming: &crate::model::StateSnapshot,
) -> bool {
    if incoming.updated_at > local.updated_at {
        return true;
    }
    if incoming.updated_at < local.updated_at {
        return false;
    }

    let local_hash = compute_snapshot_hash(local);
    let incoming_hash = compute_snapshot_hash(incoming);
    incoming_hash > local_hash
}

// 按对象规模对目标时延做轻量调优：数据量越小，P95 目标应越低。
fn tuned_p95_ms(configured_ms: u64, object_count: usize) -> u64 {
    let base = configured_ms.clamp(80, 1500);
    match object_count {
        0..=10 => base.min(120),
        11..=100 => base.min(180),
        101..=1000 => base.min(260),
        _ => base,
    }
}

// 根据域名和部署信息推断存储内容类型，便于前端直观展示“存了什么数据”。
fn infer_stored_data(domain: &str, deployment: &str) -> String {
    let key = format!("{} {}", domain.to_ascii_lowercase(), deployment.to_ascii_lowercase());

    if key.contains("api") {
        "API 索引、请求日志与检索元数据".to_string()
    } else if key.contains("archive") {
        "冷归档分片、审计轨迹与长期备份数据".to_string()
    } else if key.contains("ai") || key.contains("model") {
        "模型权重、推理输入输出缓存与结果快照".to_string()
    } else if key.contains("media") || key.contains("image") || key.contains("video") {
        "多媒体对象、缩略图索引与访问统计数据".to_string()
    } else {
        "业务对象数据、路由索引与治理元信息".to_string()
    }
}

#[derive(Debug, Deserialize)]
// 注册请求体。
struct RegisterRequest {
    owner: String,
    domain: String,
    deployment: String,
    ttl: u64,
}

#[derive(Debug, Deserialize)]
// 续期请求体。
struct RenewRequest {
    owner: String,
    domain: String,
    ttl: u64,
}

#[derive(Debug, Deserialize)]
// 删除请求的查询参数。
struct DeleteQuery {
    owner: String,
}

#[derive(Debug, Deserialize)]
// 协议仿真请求体。
struct ProtocolSimulateRequest {
    size_gb: f64,
    hot_data_percent: u8,
    months: u32,
}

#[derive(Debug, Deserialize)]
// 协议执行计划请求体。
struct ProtocolPlanRequest {
    size_gb: f64,
    shard_mb: u32,
    replicas: u8,
    node_count: u16,
}

#[derive(Debug, Deserialize)]
// 自动修复状态机请求体。
struct ProtocolRepairRequest {
    total_shards: u32,
    failed_challenges: u32,
    replicas: u8,
    node_count: u16,
}

#[derive(Debug, Deserialize)]
// 惩罚恢复引擎请求体。
struct ProtocolPenaltyRequest {
    node_count: u16,
    malicious_percent: u8,
    offline_percent: u8,
}

#[derive(Debug, Deserialize)]
// 冷热迁移执行器请求体。
struct ProtocolMigrateRequest {
    total_shards: u32,
    hot_shard_percent: u8,
    target_hot_percent: u8,
}

#[derive(Debug, Deserialize)]
// 一键工作流请求体。
struct ProtocolWorkflowRequest {
    size_gb: f64,
    shard_mb: u32,
    replicas: u8,
    node_count: u16,
    hot_data_percent: u8,
    target_hot_percent: u8,
    malicious_percent: u8,
    offline_percent: u8,
}

#[derive(Debug, Deserialize)]
// 管理层报告请求体。
struct ProtocolReportRequest {
    size_gb: f64,
    shard_mb: u32,
    replicas: u8,
    node_count: u16,
    hot_data_percent: u8,
    target_hot_percent: u8,
    malicious_percent: u8,
    offline_percent: u8,
}

#[derive(Debug, Deserialize)]
// 参数自动寻优请求体。
struct ProtocolOptimizeRequest {
    size_gb: f64,
    target_p95_ms: u32,
    max_monthly_budget_usd: f64,
    max_risk_level: u8,
}

#[derive(Debug, Deserialize)]
// Pareto 前沿请求体。
struct ProtocolParetoRequest {
    size_gb: f64,
    max_monthly_budget_usd: f64,
}

#[derive(Debug, Deserialize)]
// 业务模板选型请求体。
struct ProtocolTemplateRequest {
    template: String,
    size_gb: f64,
    max_monthly_budget_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
// 模板矩阵请求体。
struct ProtocolTemplateMatrixRequest {
    size_gb: f64,
    max_monthly_budget_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
// 模板冠军请求体。
struct ProtocolTemplateChampionRequest {
    size_gb: f64,
    max_monthly_budget_usd: Option<f64>,
    strategy: Option<String>,
}

// 注册域名接口。
async fn register_domain(
    State(state): State<ApiState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    with_mut_snapshot(&state, |snapshot| {
        domain_service::register(snapshot, &req.owner, &req.domain, &req.deployment, req.ttl)
    })
    .await
}

// 续期域名接口。
async fn renew_domain(
    State(state): State<ApiState>,
    Json(req): Json<RenewRequest>,
) -> impl IntoResponse {
    with_mut_snapshot(&state, |snapshot| {
        domain_service::renew(snapshot, &req.owner, &req.domain, req.ttl)
    })
    .await
}

// 删除域名接口（owner 通过 query 参数传递）。
async fn delete_domain(
    State(state): State<ApiState>,
    Path(domain): Path<String>,
    Query(query): Query<DeleteQuery>,
) -> impl IntoResponse {
    with_mut_snapshot(&state, |snapshot| {
        domain_service::delete(snapshot, &query.owner, &domain)
    })
    .await
}

// 查询单个域名接口。
async fn resolve_domain(State(state): State<ApiState>, Path(domain): Path<String>) -> impl IntoResponse {
    // 查询操作也需要加锁，避免并发写入时读到不一致状态。
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    match domain_service::resolve(&snapshot, &domain) {
        Ok(value) => {
            let parsed = serde_json::from_str::<serde_json::Value>(&value)
                .unwrap_or_else(|_| json!({"raw": value}));
            (StatusCode::OK, Json(parsed)).into_response()
        }
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 查询全部域名接口。
async fn list_domains(State(state): State<ApiState>) -> impl IntoResponse {
    // 列表接口复用快照读取逻辑，保证输出稳定。
    let _guard = state.lock.lock().await;
    let snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    match domain_service::list(&snapshot) {
        Ok(value) => {
            let parsed = serde_json::from_str::<serde_json::Value>(&value)
                .unwrap_or_else(|_| json!({"raw": value}));
            (StatusCode::OK, Json(parsed)).into_response()
        }
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回自研协议蓝图。
async fn protocol_blueprint() -> impl IntoResponse {
    (StatusCode::OK, Json(json!(protocol::blueprint()))).into_response()
}

// 返回自研协议仿真结果。
async fn protocol_simulate(Json(req): Json<ProtocolSimulateRequest>) -> impl IntoResponse {
    match protocol::simulate(protocol::SimulationInput {
        size_gb: req.size_gb,
        hot_data_percent: req.hot_data_percent,
        months: req.months,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回自研协议分片放置和挑战计划。
async fn protocol_plan(Json(req): Json<ProtocolPlanRequest>) -> impl IntoResponse {
    match protocol::plan(protocol::PlanInput {
        size_gb: req.size_gb,
        shard_mb: req.shard_mb,
        replicas: req.replicas,
        node_count: req.node_count,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回挑战失败后的自动修复状态机结果。
async fn protocol_repair(Json(req): Json<ProtocolRepairRequest>) -> impl IntoResponse {
    match protocol::repair_state_machine(protocol::RepairInput {
        total_shards: req.total_shards,
        failed_challenges: req.failed_challenges,
        replicas: req.replicas,
        node_count: req.node_count,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回节点惩罚与恢复执行结果。
async fn protocol_penalty(Json(req): Json<ProtocolPenaltyRequest>) -> impl IntoResponse {
    match protocol::apply_penalty_rules(protocol::PenaltyInput {
        node_count: req.node_count,
        malicious_percent: req.malicious_percent,
        offline_percent: req.offline_percent,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回冷热分层迁移执行结果。
async fn protocol_migrate(Json(req): Json<ProtocolMigrateRequest>) -> impl IntoResponse {
    match protocol::execute_tier_migration(protocol::MigrationInput {
        total_shards: req.total_shards,
        hot_shard_percent: req.hot_shard_percent,
        target_hot_percent: req.target_hot_percent,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回一键治理工作流报告。
async fn protocol_workflow(Json(req): Json<ProtocolWorkflowRequest>) -> impl IntoResponse {
    match protocol::run_workflow(protocol::WorkflowInput {
        size_gb: req.size_gb,
        shard_mb: req.shard_mb,
        replicas: req.replicas,
        node_count: req.node_count,
        hot_data_percent: req.hot_data_percent,
        target_hot_percent: req.target_hot_percent,
        malicious_percent: req.malicious_percent,
        offline_percent: req.offline_percent,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回管理层报告 JSON。
async fn protocol_report(Json(req): Json<ProtocolReportRequest>) -> impl IntoResponse {
    match protocol::build_management_report(protocol::WorkflowInput {
        size_gb: req.size_gb,
        shard_mb: req.shard_mb,
        replicas: req.replicas,
        node_count: req.node_count,
        hot_data_percent: req.hot_data_percent,
        target_hot_percent: req.target_hot_percent,
        malicious_percent: req.malicious_percent,
        offline_percent: req.offline_percent,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回参数自动寻优结果。
async fn protocol_optimize(Json(req): Json<ProtocolOptimizeRequest>) -> impl IntoResponse {
    match protocol::optimize_parameters(protocol::OptimizeInput {
        size_gb: req.size_gb,
        target_p95_ms: req.target_p95_ms,
        max_monthly_budget_usd: req.max_monthly_budget_usd,
        max_risk_level: req.max_risk_level,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回 Pareto 前沿结果。
async fn protocol_pareto(Json(req): Json<ProtocolParetoRequest>) -> impl IntoResponse {
    match protocol::pareto_frontier(protocol::ParetoInput {
        size_gb: req.size_gb,
        max_monthly_budget_usd: req.max_monthly_budget_usd,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回 Pareto 方案解释与场景建议。
async fn protocol_pareto_explain(Json(req): Json<ProtocolParetoRequest>) -> impl IntoResponse {
    match protocol::explain_pareto(protocol::ParetoInput {
        size_gb: req.size_gb,
        max_monthly_budget_usd: req.max_monthly_budget_usd,
    }) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回业务模板一键推荐结果。
async fn protocol_template(Json(req): Json<ProtocolTemplateRequest>) -> impl IntoResponse {
    let Some(template) = protocol::parse_business_template(&req.template) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid template, expected: archive|web3-frontend|ai-inference"})),
        )
            .into_response();
    };

    match protocol::recommend_by_template(template, req.size_gb, req.max_monthly_budget_usd) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回三模板并排对比结果。
async fn protocol_template_matrix(
    Json(req): Json<ProtocolTemplateMatrixRequest>,
) -> impl IntoResponse {
    match protocol::template_matrix(req.size_gb, req.max_monthly_budget_usd) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 返回模板冠军推荐与评分排名。
async fn protocol_template_champion(
    Json(req): Json<ProtocolTemplateChampionRequest>,
) -> impl IntoResponse {
    let strategy = match req.strategy.as_deref() {
        Some(value) => match protocol::parse_champion_strategy(value) {
            Some(s) => s,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "invalid strategy, expected: balanced|latency-first|cost-first|reliability-first"})),
                )
                    .into_response();
            }
        },
        None => protocol::ChampionStrategy::Balanced,
    };

    match protocol::template_champion(req.size_gb, req.max_monthly_budget_usd, strategy) {
        Ok(report) => (StatusCode::OK, Json(json!(report))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}

// 统一的可变快照操作模板：加锁、加载、执行、保存。
async fn with_mut_snapshot<F>(state: &ApiState, f: F) -> axum::response::Response
where
    F: FnOnce(&mut crate::model::StateSnapshot) -> Result<String, String>,
{
    let _guard = state.lock.lock().await;

    let mut snapshot = match state_store::load_or_default(state.snapshot_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response();
        }
    };

    match f(&mut snapshot) {
        Ok(message) => match state_store::save(state.snapshot_path.as_ref(), &snapshot) {
            Ok(()) => (StatusCode::OK, Json(json!({"message": message}))).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": err})),
            )
                .into_response(),
        },
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }
}
