use axum::{extract::State, response::IntoResponse, routing::post, Router};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::mcp_stdio::handle_request_line;
use crate::store::MemoryStore;

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<MemoryStore>>,
}

/// MCP over HTTP: POST a JSON-RPC request body to /mcp, get a JSON-RPC response back.
/// Reuses the same request handler as the stdio transport (mcp_stdio::handle_request_line)
/// so both transports share one implementation of the protocol.
pub async fn serve(project_dir: PathBuf, port: u16) {
    let store = MemoryStore::open_in_project(&project_dir)
        .expect("Failed to open memory store. Run `pmem init` first.");
    let state = AppState { store: Arc::new(Mutex::new(store)) };

    let app = Router::new().route("/mcp", post(handle_mcp)).with_state(state);

    let addr = format!("127.0.0.1:{port}");
    println!("  MCP endpoint: http://{addr}/mcp");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind port");

    axum::serve(listener, app).await.expect("Server failed");
}

async fn handle_mcp(State(state): State<AppState>, body: String) -> impl IntoResponse {
    // handle_request_line does blocking rusqlite I/O — run it off the async executor so a
    // slow query doesn't stall every other in-flight request on this worker thread.
    let store = state.store.clone();
    let response = tokio::task::spawn_blocking(move || handle_request_line(&body, &store).unwrap_or_default())
        .await
        .unwrap_or_default();
    ([("content-type", "application/json")], response)
}
