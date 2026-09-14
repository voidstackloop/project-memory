#!/usr/bin/env python3
"""Compare the token cost of loading project context from pmem's memory_context
MCP tool against the token cost of an agent re-deriving the same facts by
reading the actual source files that contain them.

The 12 seeded memories below are real facts about *this* codebase (pmem
itself), established over the course of building and fixing it — not
synthetic filler. Each maps to a specific source file an agent would
otherwise have to read to learn the same thing.

Usage:
    python3 scripts/token_benchmark.py [path-to-pmem-binary]
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

try:
    import tiktoken
    ENC = tiktoken.get_encoding("cl100k_base")
    def count_tokens(text: str) -> int:
        return len(ENC.encode(text))
except ImportError:
    print("tiktoken not installed (pip install tiktoken) — falling back to a chars/4 estimate.\n", file=sys.stderr)
    def count_tokens(text: str) -> int:
        return len(text) // 4

BIN = sys.argv[1] if len(sys.argv) > 1 else "./target/release/pmem"
REPO_ROOT = Path(__file__).resolve().parent.parent

MEMORIES = [
    ("convention", "naming", "Use snake_case for all Rust identifiers, PascalCase for types.", ["rust", "style"]),
    ("decision", "database", "Use SQLite via rusqlite's bundled feature; no external DB service.", ["storage"]),
    ("pattern", "error-handling", "Use anyhow::Result for fallible functions, propagate with ?.", ["rust"]),
    ("decision", "encryption", "Sensitive content is encrypted at rest with AES-256-GCM; key derived via PBKDF2-HMAC-SHA256 (100k iterations) with a random salt and nonce per encryption. Stored as ENC:hex(salt||nonce||ciphertext).", ["security"]),
    ("pattern", "server-bind", "All three HTTP servers (dashboard, api, mcp) read PMEM_HOST from env, defaulting to 127.0.0.1. docker-compose sets it to 0.0.0.0, since a container's loopback isn't reachable through Docker's port mapping.", ["deploy"]),
    ("decision", "mcp-transports", "MCP stdio and MCP HTTP share one request handler (mcp_stdio::handle_request_line) so the JSON-RPC protocol is implemented once, not twice.", ["architecture"]),
    ("pattern", "mutex-safety", "Store mutex locks use .lock().unwrap_or_else(|e| e.into_inner()) to survive a poisoned lock instead of propagating the panic to every future request.", ["rust", "reliability"]),
    ("pattern", "atomic-writes", "Multi-row mutations that must succeed or fail together (dedupe merge, snapshot restore) run inside a single SQLite transaction via conn.unchecked_transaction(), not a loop of independent statements.", ["rust", "reliability"]),
    ("convention", "backup-naming", "Regular backups: backup_<timestamp>.db. Pre-restore safety copies: pre_restore_<timestamp>_<uuid8>.db, so list_backups/cleanup_backups (which only match the backup_ prefix) never sweep them up.", ["backup"]),
    ("decision", "rest-api", "The REST API mirrors the dashboard's CRUD/search/stats endpoints, with CORS enabled via tower-http since browser-based integrations are an intended use case.", ["api"]),
    ("context", "known-limitation", "fuzzy_search and a few other store.rs methods load the full memories table into memory before scoring/filtering client-side, rather than pushing candidate selection into SQL. Fine at typical single-project scale; a real limitation on very large stores.", ["performance"]),
    ("preference", "testing", "Every fix in this codebase gets verified with a live CLI/HTTP smoke test, not just cargo test, before being considered done.", ["testing"]),
]

# The actual source files that together contain the facts above.
SOURCE_FILES = [
    "src/main.rs", "src/store.rs", "src/encryption.rs", "src/mcp.rs",
    "src/mcp_stdio.rs", "src/api.rs", "src/dashboard.rs", "src/backup.rs",
]


def call(proc, req):
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())


def main():
    with tempfile.TemporaryDirectory() as workdir:
        subprocess.run([BIN, "--project", workdir, "init"], capture_output=True)
        for kind, key, content, tags in MEMORIES:
            subprocess.run([BIN, "--project", workdir, "add", "--kind", kind, "-k", key, "-c", content, "--tags", ",".join(tags)], capture_output=True)

        proc = subprocess.Popen([BIN, "--project", workdir, "stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
        call(proc, {"jsonrpc": "2.0", "id": 1, "method": "initialize"})

        context_resp = call(proc, {"jsonrpc": "2.0", "id": 2, "method": "tools/call",
                                    "params": {"name": "memory_context", "arguments": {}}})
        context_text = context_resp["result"]["content"][0]["text"]

        search_resp = call(proc, {"jsonrpc": "2.0", "id": 3, "method": "tools/call",
                                   "params": {"name": "memory_search", "arguments": {"query": "encryption", "limit": 3}}})
        search_text = search_resp["result"]["content"][0]["text"]

        proc.terminate()

    context_tokens = count_tokens(context_text)
    search_tokens = count_tokens(search_text)

    print(f"=== memory_context: all {len(MEMORIES)} memories in one call ===")
    print(f"{context_tokens} tokens\n")

    print("=== memory_search: one targeted query ('encryption', limit 3) ===")
    print(f"{search_tokens} tokens\n")

    print(f"=== reading the {len(SOURCE_FILES)} source files that contain these facts ===")
    total_source_tokens = 0
    for rel in SOURCE_FILES:
        path = REPO_ROOT / rel
        text = path.read_text()
        n = count_tokens(text)
        total_source_tokens += n
        print(f"  {rel:<22} {n:>7,} tokens  ({len(text.splitlines())} lines)")
    print(f"  {'total':<22} {total_source_tokens:>7,} tokens\n")

    print("=== summary ===")
    print(f"memory_context:  {context_tokens:>7,} tokens  ({context_tokens / total_source_tokens * 100:.1f}% of reading the source)")
    print(f"memory_search:   {search_tokens:>7,} tokens  ({search_tokens / total_source_tokens * 100:.2f}% of reading the source)")
    print(f"reading source:  {total_source_tokens:>7,} tokens")
    print(f"\nmemory_context is {total_source_tokens / context_tokens:.0f}x cheaper than reading the source files that contain the same facts.")
    print(f"memory_search is {total_source_tokens / search_tokens:.0f}x cheaper for a single targeted question.")


if __name__ == "__main__":
    main()
