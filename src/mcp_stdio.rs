use crate::store::MemoryStore;
use crate::types::{MemoryInput, MemoryKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

impl JsonRpcResponse {
    fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

pub fn serve_stdio(project_dir: PathBuf) {
    let store = MemoryStore::open_in_project(&project_dir)
        .expect("Failed to open memory store. Run `pmem init` first.");
    let store = Arc::new(Mutex::new(store));

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if let Some(response) = handle_request_line(&line, &store) {
            let _ = writeln!(stdout, "{response}");
            let _ = stdout.flush();
        }
    }
}

/// Parses and dispatches a single JSON-RPC request line, returning the serialized
/// response (or None for notifications/blank lines, which get no response).
/// Shared by the stdio transport (serve_stdio) and the HTTP transport (mcp::serve).
pub fn handle_request_line(line: &str, store: &Arc<Mutex<MemoryStore>>) -> Option<String> {
    if line.trim().is_empty() {
        return None;
    }

    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            let resp = JsonRpcResponse::error(None, -32700, format!("Parse error: {e}"));
            return Some(serde_json::to_string(&resp).unwrap());
        }
    };

    let response = match req.method.as_str() {
        "initialize" => handle_initialize(req.id),
        "notifications/initialized" => return None,
        "tools/list" => handle_tools_list(req.id),
        "tools/call" => handle_tools_call(req.id, req.params, store),
        "ping" => JsonRpcResponse::success(req.id, serde_json::json!({})),
        _ => JsonRpcResponse::error(
            req.id,
            -32601,
            format!("Method not found: {}", req.method),
        ),
    };

    Some(serde_json::to_string(&response).unwrap())
}

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "project-memory",
                "version": "0.2.0"
            }
        }),
    )
}

fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    let tools = serde_json::json!([
        {
            "name": "memory_add",
            "description": "Store a project convention, pattern, decision, preference, or context",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "kind": {
                        "type": "string",
                        "enum": ["convention", "pattern", "decision", "preference", "context"],
                        "description": "Type of memory"
                    },
                    "key": {
                        "type": "string",
                        "description": "Short identifier (e.g. 'naming-rust', 'db-choice')"
                    },
                    "content": {
                        "type": "string",
                        "description": "The full memory content"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional tags for categorization"
                    }
                },
                "required": ["kind", "key", "content"]
            }
        },
        {
            "name": "memory_search",
            "description": "Search project memories by keyword with fuzzy matching",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 10
                    }
                },
                "required": ["query"]
            }
        },
        {
            "name": "memory_list",
            "description": "List all project memories, optionally filtered by kind",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "kind": {
                        "type": "string",
                        "enum": ["convention", "pattern", "decision", "preference", "context"]
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                }
            }
        },
        {
            "name": "memory_get",
            "description": "Get a single memory by ID with its related memories",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "memory_delete",
            "description": "Delete a memory by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }
        },
        {
            "name": "memory_context",
            "description": "Get all project memories as formatted context for AI consumption. Returns conventions, patterns, decisions, and preferences organized by category.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "memory_link",
            "description": "Link two related memories together",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_id": { "type": "string" },
                    "target_id": { "type": "string" }
                },
                "required": ["source_id", "target_id"]
            }
        }
    ]);

    JsonRpcResponse::success(id, serde_json::json!({ "tools": tools }))
}

fn handle_tools_call(
    id: Option<Value>,
    params: Value,
    store: &Arc<Mutex<MemoryStore>>,
) -> JsonRpcResponse {
    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(name) => name.to_string(),
        None => return JsonRpcResponse::error(id, -32602, "Missing 'name' in params".into()),
    };

    let args = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
    // Recover from a poisoned lock instead of panicking: the SQLite connection itself
    // isn't corrupted by a panic in a previous request, so one bad request shouldn't
    // take down every subsequent request for the rest of this long-lived session.
    let store = store.lock().unwrap_or_else(|e| e.into_inner());

    let result = match tool_name.as_str() {
        "memory_add" => tool_memory_add(&store, &args),
        "memory_search" => tool_memory_search(&store, &args),
        "memory_list" => tool_memory_list(&store, &args),
        "memory_get" => tool_memory_get(&store, &args),
        "memory_delete" => tool_memory_delete(&store, &args),
        "memory_context" => tool_memory_context(&store),
        "memory_link" => tool_memory_link(&store, &args),
        _ => Err(format!("Unknown tool: {tool_name}")),
    };

    match result {
        Ok(text) => JsonRpcResponse::success(
            id,
            serde_json::json!({
                "content": [{ "type": "text", "text": text }]
            }),
        ),
        Err(e) => JsonRpcResponse::success(
            id,
            serde_json::json!({
                "content": [{ "type": "text", "text": format!("Error: {}", e) }],
                "isError": true
            }),
        ),
    }
}

fn tool_memory_add(store: &MemoryStore, args: &Value) -> Result<String, String> {
    let kind: MemoryKind = args
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'kind'")?
        .parse::<MemoryKind>().map_err(|e| e)?;

    let key = args
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'key'")?
        .to_string();

    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'content'")?
        .to_string();

    let tags: Vec<String> = args
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let memory = store
        .add(MemoryInput { kind, key, content, tags, related_ids: Vec::new() })
        .map_err(|e| e.to_string())?;

    Ok(format!("Stored memory [{}] {}: {}", memory.kind, memory.key, memory.id))
}

fn tool_memory_search(store: &MemoryStore, args: &Value) -> Result<String, String> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'query'")?;

    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let scored = store.fuzzy_search(query, limit).map_err(|e| e.to_string())?;

    if scored.is_empty() {
        return Ok(format!("No memories found matching '{query}'"));
    }

    let mut output = format!("Found {} memories:\n\n", scored.len());
    for sm in &scored {
        let m = &sm.memory;
        output.push_str(&format!("[{}] {}: {} (score: {:.2})\n", m.kind, m.key, m.content, sm.score));
        if !m.tags.is_empty() {
            output.push_str(&format!("  tags: {}\n", m.tags.join(", ")));
        }
    }
    Ok(output)
}

fn tool_memory_list(store: &MemoryStore, args: &Value) -> Result<String, String> {
    let kind: Option<MemoryKind> = args
        .get("kind")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok());

    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as usize;

    let memories = store.list(kind, limit).map_err(|e| e.to_string())?;

    if memories.is_empty() {
        return Ok("No memories stored yet.".to_string());
    }

    let mut output = format!("{} memories:\n\n", memories.len());
    for m in &memories {
        output.push_str(&format!("[{}] {}: {}\n", m.kind, m.key, m.content));
    }
    Ok(output)
}

fn tool_memory_get(store: &MemoryStore, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'id'")?;

    match store.get(id).map_err(|e| e.to_string())? {
        Some(m) => {
            let mut output = format!(
                "ID: {}\nKind: {}\nKey: {}\nContent: {}\nTags: {}\nCreated: {}\nUpdated: {}",
                m.id, m.kind, m.key, m.content, m.tags.join(", "), m.created_at, m.updated_at
            );
            let related = store.get_related(id).map_err(|e| e.to_string())?;
            if !related.is_empty() {
                output.push_str("\nRelated memories:\n");
                for r in &related {
                    output.push_str(&format!("  - [{}] {}: {}\n", r.kind, r.key, &r.id[..8]));
                }
            }
            Ok(output)
        }
        None => Ok(format!("Memory '{id}' not found")),
    }
}

fn tool_memory_delete(store: &MemoryStore, args: &Value) -> Result<String, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'id'")?;

    if store.delete(id).map_err(|e| e.to_string())? {
        Ok(format!("Deleted memory {id}"))
    } else {
        Ok(format!("Memory '{id}' not found"))
    }
}

fn tool_memory_link(store: &MemoryStore, args: &Value) -> Result<String, String> {
    let source_id = args
        .get("source_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'source_id'")?;

    let target_id = args
        .get("target_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'target_id'")?;

    if store.link(source_id, target_id).map_err(|e| e.to_string())? {
        Ok(format!("Linked {source_id} -> {target_id}"))
    } else {
        Ok("One or both memory IDs not found".to_string())
    }
}

fn tool_memory_context(store: &MemoryStore) -> Result<String, String> {
    let memories = store.list(None, 1000).map_err(|e| e.to_string())?;

    if memories.is_empty() {
        return Ok("No project memories stored. This project has no saved conventions, patterns, or decisions.".to_string());
    }

    let mut conventions = Vec::new();
    let mut patterns = Vec::new();
    let mut decisions = Vec::new();
    let mut preferences = Vec::new();
    let mut contexts = Vec::new();

    for m in &memories {
        let entry = if m.tags.is_empty() {
            format!("- {}: {}", m.key, m.content)
        } else {
            format!("- {} [{}]: {}", m.key, m.tags.join(", "), m.content)
        };
        match m.kind {
            MemoryKind::Convention => conventions.push(entry),
            MemoryKind::Pattern => patterns.push(entry),
            MemoryKind::Decision => decisions.push(entry),
            MemoryKind::Preference => preferences.push(entry),
            MemoryKind::Context => contexts.push(entry),
        }
    }

    let mut output = String::from("# Project Memory Context\n\n");

    if !conventions.is_empty() {
        output.push_str("## Conventions\n");
        output.push_str(&conventions.join("\n"));
        output.push_str("\n\n");
    }
    if !patterns.is_empty() {
        output.push_str("## Patterns\n");
        output.push_str(&patterns.join("\n"));
        output.push_str("\n\n");
    }
    if !decisions.is_empty() {
        output.push_str("## Decisions\n");
        output.push_str(&decisions.join("\n"));
        output.push_str("\n\n");
    }
    if !preferences.is_empty() {
        output.push_str("## Preferences\n");
        output.push_str(&preferences.join("\n"));
        output.push_str("\n\n");
    }
    if !contexts.is_empty() {
        output.push_str("## Context\n");
        output.push_str(&contexts.join("\n"));
        output.push('\n');
    }

    Ok(output)
}
