use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use tower_http::cors::CorsLayer;

use crate::store::MemoryStore;
use crate::types::{MemoryInput, MemoryKind};

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<MemoryStore>>,
}

fn lock(state: &AppState) -> MutexGuard<'_, MemoryStore> {
    state.store.lock().unwrap_or_else(|e| e.into_inner())
}

pub async fn serve_api(project_dir: PathBuf, port: u16) {
    let store = MemoryStore::open_in_project(&project_dir)
        .expect("Failed to open memory store. Run `pmem init` first.");
    let state = AppState { store: Arc::new(Mutex::new(store)) };

    let app = Router::new()
        .route("/api/health", get(|| async { Json(serde_json::json!({"status": "ok"})) }))
        .route("/api/memories", get(list_memories).post(add_memory))
        .route("/api/memories/{id}", get(get_memory).put(update_memory).delete(delete_memory))
        .route("/api/search", get(search_memories))
        .route("/api/stats", get(stats))
        .route("/api/tags", get(tags))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let host = std::env::var("PMEM_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{host}:{port}");
    println!("  REST API: http://{addr}/api");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind port");

    axum::serve(listener, app).await.expect("Server failed");
}

#[derive(Deserialize)]
struct ListParams {
    kind: Option<String>,
    limit: Option<usize>,
}

async fn list_memories(State(state): State<AppState>, Query(p): Query<ListParams>) -> impl IntoResponse {
    let kind = p.kind.and_then(|k| k.parse::<MemoryKind>().ok());
    match lock(&state).list(kind, p.limit.unwrap_or(50)) {
        Ok(memories) => Json(memories).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_memory(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match lock(&state).get(&id) {
        Ok(Some(m)) => Json(m).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "memory not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn add_memory(State(state): State<AppState>, Json(input): Json<MemoryInput>) -> impl IntoResponse {
    match lock(&state).add(input) {
        Ok(m) => (StatusCode::CREATED, Json(m)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn update_memory(State(state): State<AppState>, Path(id): Path<String>, Json(input): Json<MemoryInput>) -> impl IntoResponse {
    match lock(&state).update(&id, input) {
        Ok(Some(m)) => Json(m).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "memory not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_memory(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match lock(&state).delete(&id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "memory not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    limit: Option<usize>,
}

async fn search_memories(State(state): State<AppState>, Query(p): Query<SearchParams>) -> impl IntoResponse {
    match lock(&state).fuzzy_search(&p.q, p.limit.unwrap_or(10)) {
        Ok(results) => Json(results).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn stats(State(state): State<AppState>) -> impl IntoResponse {
    let store = lock(&state);
    let count = store.count().unwrap_or(0);
    let by_kind = store.count_by_kind().unwrap_or_default();
    let tags = store.list_tags().unwrap_or_default();
    Json(serde_json::json!({
        "total": count,
        "by_kind": by_kind.iter().map(|(k, c)| (k.to_string(), c)).collect::<Vec<_>>(),
        "tags": tags,
    }))
}

async fn tags(State(state): State<AppState>) -> impl IntoResponse {
    Json(lock(&state).list_tags().unwrap_or_default())
}
