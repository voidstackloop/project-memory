use axum::{
    Router,
    extract::State,
    response::Html,
    routing::get,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::store::MemoryStore;

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<MemoryStore>>,
}

pub async fn serve_dashboard(project_dir: PathBuf, port: u16) {
    let store = MemoryStore::open_in_project(&project_dir)
        .expect("Failed to open memory store. Run `pmem init` first.");

    let state = AppState {
        store: Arc::new(Mutex::new(store)),
    };

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/api/memories", get(api_memories))
        .route("/api/stats", get(api_stats))
        .with_state(state);

    let addr = format!("127.0.0.1:{port}");
    println!("  Dashboard: http://{addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind port");

    axum::serve(listener, app).await.expect("Server failed");
}

async fn dashboard(State(state): State<AppState>) -> Html<String> {
    let store = state.store.lock().unwrap_or_else(|e| e.into_inner());
    let memories = store.list(None, 10000).unwrap_or_default();
    let count = store.count().unwrap_or(0);
    let tags = store.list_tags().unwrap_or_default();
    let by_kind = store.count_by_kind().unwrap_or_default();

    let mut kind_rows = String::new();
    for (kind, cnt) in &by_kind {
        let color = match kind {
            crate::types::MemoryKind::Convention => "#06b6d4",
            crate::types::MemoryKind::Pattern => "#22c55e",
            crate::types::MemoryKind::Decision => "#eab308",
            crate::types::MemoryKind::Preference => "#a855f7",
            crate::types::MemoryKind::Context => "#3b82f6",
        };
        kind_rows.push_str(&format!(
            r#"<span class="badge" style="background:{color}">{kind}: {cnt}</span> "#
        ));
    }

    let mut tag_badges = String::new();
    for ti in tags.iter().take(20) {
        tag_badges.push_str(&format!(
            r#"<span class="tag">{} ({})</span> "#,
            ti.tag, ti.count
        ));
    }

    let mut memory_cards = String::new();
    for m in &memories {
        let color = match m.kind {
            crate::types::MemoryKind::Convention => "#06b6d4",
            crate::types::MemoryKind::Pattern => "#22c55e",
            crate::types::MemoryKind::Decision => "#eab308",
            crate::types::MemoryKind::Preference => "#a855f7",
            crate::types::MemoryKind::Context => "#3b82f6",
        };
        let tags_html: String = m.tags.iter()
            .map(|t| format!(r#"<span class="tag">{t}</span>"#))
            .collect::<Vec<_>>()
            .join(" ");
        memory_cards.push_str(&format!(
            r#"<div class="card" data-kind="{}" data-tags="{}">
                <div class="card-header">
                    <span class="badge" style="background:{}">{}</span>
                    <span class="key">{}</span>
                    <span class="id">{}</span>
                </div>
                <div class="content">{}</div>
                <div class="tags">{}</div>
            </div>"#,
            m.kind,
            m.tags.join(","),
            color,
            m.kind,
            m.key,
            &m.id[..8],
            m.content,
            tags_html
        ));
    }

    Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>project-memory dashboard</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #0f172a; color: #e2e8f0; padding: 2rem; }}
        h1 {{ color: #f8fafc; margin-bottom: 1rem; }}
        .stats {{ display: flex; gap: 1rem; margin-bottom: 2rem; flex-wrap: wrap; }}
        .stat {{ background: #1e293b; padding: 1rem 1.5rem; border-radius: 8px; }}
        .stat-value {{ font-size: 2rem; font-weight: bold; color: #38bdf8; }}
        .stat-label {{ color: #94a3b8; font-size: 0.875rem; }}
        .filter {{ margin-bottom: 1.5rem; }}
        .filter input {{ background: #1e293b; border: 1px solid #334155; color: #e2e8f0; padding: 0.5rem 1rem; border-radius: 6px; width: 100%; max-width: 400px; font-size: 1rem; }}
        .filter input:focus {{ outline: none; border-color: #38bdf8; }}
        .badges {{ margin-bottom: 1.5rem; }}
        .badge {{ display: inline-block; padding: 0.25rem 0.75rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; color: white; }}
        .tag {{ display: inline-block; padding: 0.125rem 0.5rem; background: #334155; border-radius: 4px; font-size: 0.75rem; margin: 0.125rem; }}
        .cards {{ display: grid; gap: 1rem; }}
        .card {{ background: #1e293b; padding: 1rem 1.5rem; border-radius: 8px; border-left: 4px solid #334155; }}
        .card-header {{ display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; }}
        .key {{ font-weight: 600; color: #f8fafc; }}
        .id {{ color: #64748b; font-size: 0.75rem; font-family: monospace; }}
        .content {{ color: #cbd5e1; line-height: 1.6; }}
        .tags {{ margin-top: 0.5rem; }}
        .empty {{ text-align: center; padding: 3rem; color: #64748b; }}
    </style>
</head>
<body>
    <h1>project-memory</h1>
    <div class="stats">
        <div class="stat"><div class="stat-value">{count}</div><div class="stat-label">memories</div></div>
        <div class="stat"><div class="stat-value">{}</div><div class="stat-label">tags</div></div>
    </div>
    <div class="badges">{kind_rows}</div>
    <div class="badges">{tag_badges}</div>
    <div class="filter"><input type="text" id="search" placeholder="Filter memories..." oninput="filterCards()"></div>
    <div class="cards" id="cards">{memory_cards}</div>
    <script>
        function filterCards() {{
            const q = document.getElementById('search').value.toLowerCase();
            document.querySelectorAll('.card').forEach(card => {{
                const text = card.textContent.toLowerCase();
                card.style.display = text.includes(q) ? '' : 'none';
            }});
        }}
    </script>
</body>
</html>"#,
        tags.len()
    ))
}

async fn api_memories(State(state): State<AppState>) -> String {
    let store = state.store.lock().unwrap_or_else(|e| e.into_inner());
    let memories = store.list(None, 10000).unwrap_or_default();
    serde_json::to_string_pretty(&memories).unwrap()
}

async fn api_stats(State(state): State<AppState>) -> String {
    let store = state.store.lock().unwrap_or_else(|e| e.into_inner());
    let count = store.count().unwrap_or(0);
    let by_kind = store.count_by_kind().unwrap_or_default();
    let tags = store.list_tags().unwrap_or_default();

    serde_json::to_string_pretty(&serde_json::json!({
        "total": count,
        "by_kind": by_kind.iter().map(|(k, c)| (k.to_string(), c)).collect::<Vec<_>>(),
        "tags": tags.iter().map(|t| (&t.tag, t.count)).collect::<Vec<_>>(),
    })).unwrap()
}
