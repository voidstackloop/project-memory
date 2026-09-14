#!/usr/bin/env python3
"""Compare the token cost of loading project context from pmem's MCP tools
against the token cost of an agent re-deriving the same facts by reading the
source that contains them.

The 50 seeded memories below are real facts about *this* codebase (pmem
itself) — bugs found and fixed, decisions made, benchmark results — recorded
over the course of actually building and fixing it. Not synthetic filler.
Each maps to real source/doc files an agent would otherwise have to read.

Runs memory_context once (the full dump), then 50 individual memory_search
calls — one per fact, each a plausible query for that fact — and reports
per-query token counts plus aggregate statistics, not just one example.

Usage:
    python3 scripts/token_benchmark.py [path-to-pmem-binary]
"""
import json
import statistics
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

# (kind, key, content, tags, search_query)
MEMORIES = [
    ("decision", "encryption-algo", "Encryption was rewritten from XOR to real AES-256-GCM with PBKDF2-HMAC-SHA256 key derivation (100k iterations).", ["security", "encryption"], "AES-256-GCM"),
    ("pattern", "snapshot-restore", "Snapshot restore preserves each memory's original id via Store::replace_all so related_ids captured in the snapshot still resolve after restore.", ["snapshot", "rust"], "snapshot restore related_ids"),
    ("pattern", "backup-validation", "Backup restore validates the backup file with a read-only SQLite open plus a query against the memories table before overwriting the live database.", ["backup", "security"], "backup validation"),
    ("pattern", "dedupe-merge", "Duplicate memory merges run inside a single SQLite transaction via merge_duplicates, so a crash mid-merge can't leave duplicates deleted without the survivor updated.", ["dedupe", "rust"], "dedupe merge transaction"),
    ("decision", "mcp-http", "pmem serve implements a real MCP HTTP server reusing mcp_stdio::handle_request_line, replacing an earlier stub that printed success and did nothing.", ["mcp", "http"], "MCP HTTP server"),
    ("pattern", "auto-fix", "validate-improved --fix now actually dedupes tags via validation::auto_fix, replacing a version that was a hardcoded no-op.", ["validation"], "validate auto-fix"),
    ("context", "dedupe-underflow", "dedupe merge-group guards against an empty duplicate-group list before computing groups.len() - 1, fixing an integer underflow panic.", ["bugfix", "rust"], "merge-group underflow panic"),
    ("context", "search-print", "highlight-search and faceted-search now print their results; they previously computed matches and silently discarded them on success.", ["search", "bugfix"], "highlight-search faceted-search print"),
    ("pattern", "mutex-safety", "Store mutex locks use .lock().unwrap_or_else(|e| e.into_inner()) so a single panic under the lock doesn't crash every subsequent request in mcp_stdio and dashboard.", ["rust", "reliability"], "poisoned mutex recovery"),
    ("context", "double-encrypt", "Encryption now checks for an existing ENC: prefix before encrypting, preventing unrecoverable double-encryption of already-encrypted content.", ["security", "bugfix"], "double encryption guard"),
    ("pattern", "backup-readonly", "Backup validation opens the file read-only via rusqlite OpenFlags::SQLITE_OPEN_READ_ONLY and queries the memories table, instead of the earlier version which silently CREATE TABLE IF NOT EXISTS'd into any file.", ["backup"], "read-only backup validation"),
    ("context", "dashboard-mutex", "dashboard.rs's three handlers (dashboard, api_memories, api_stats) all got the poisoned-mutex-safe lock pattern, after an earlier fix landed only in mcp_stdio.rs.", ["dashboard", "bugfix"], "dashboard poisoned mutex"),
    ("pattern", "replace-all", "Store::replace_all deletes and re-inserts the entire memory table inside one transaction, used by snapshot restore so a crash can't leave the store partially emptied.", ["snapshot", "rust"], "replace_all transaction"),
    ("context", "auto-fix-precision", "auto_fix now returns which memory ids it actually changed, so only those get re-saved instead of every memory's updated_at being bumped.", ["validation", "bugfix"], "auto_fix updated_at"),
    ("pattern", "cleanup-backups", "cleanup_backups and list_backups only match files with the backup_ prefix, so pre_restore_*.db safety copies are never swept up by normal housekeeping.", ["backup"], "cleanup_backups prefix filter"),
    ("context", "merge-duplicates-check", "merge_duplicates checks that its UPDATE actually affected a row before committing, aborting and rolling back the deletes if the survivor no longer exists.", ["dedupe", "bugfix"], "merge_duplicates affected rows"),
    ("context", "decrypt-empty-key", "decrypt() now errors loudly with 'cannot decrypt: no encryption key provided' instead of silently returning raw ciphertext when called with an empty key.", ["security", "bugfix"], "decrypt empty key error"),
    ("pattern", "mcp-content-type", "mcp.rs's HTTP handler now returns application/json content-type instead of the default text/plain axum sets for a bare String response.", ["mcp", "http"], "MCP HTTP content-type"),
    ("pattern", "safety-copy-naming", "pre_restore safety-copy filenames include an 8-char uuid suffix to avoid collisions when two restores happen within the same second.", ["backup"], "pre_restore filename collision"),
    ("pattern", "spawn-blocking", "mcp.rs's HTTP handler wraps the blocking SQLite call in tokio::task::spawn_blocking so it doesn't stall the async executor under concurrent requests.", ["mcp", "performance"], "spawn_blocking async"),
    ("decision", "rest-api-impl", "api.rs was a not-yet-implemented stub; it's now a real REST API with CRUD, fuzzy search, stats, and tags endpoints over MemoryStore.", ["api"], "REST API implementation"),
    ("decision", "cors", "tower-http's CORS layer, previously a declared but unused dependency, is now wired into api.rs via CorsLayer::permissive().", ["api", "http"], "CORS tower-http"),
    ("context", "api-health", "An /api/health endpoint was added to api.rs to match what main.rs's startup message already advertised but that didn't previously exist.", ["api", "bugfix"], "api health endpoint"),
    ("decision", "dockerfile", "The Dockerfile is a two-stage build: rust:1-slim-bookworm compiles the release binary, then debian:bookworm-slim runs it.", ["docker"], "Dockerfile multi-stage build"),
    ("decision", "compose-services", "docker-compose.yml runs three services (dashboard :3778, api :3779, mcp :3777), all bind-mounting the repo at /workspace so they share one .memory/store.db.", ["docker"], "docker-compose services"),
    ("pattern", "entrypoint", "docker-entrypoint.sh runs pmem init against /workspace before exec'ing the real command, idempotently (a no-op if .memory/ already exists).", ["docker"], "docker-entrypoint init"),
    ("context", "bind-address", "dashboard.rs, api.rs, and mcp.rs all hardcoded 127.0.0.1, which is loopback only within a container's own network namespace and unreachable via Docker's port mapping even though the process starts up looking fine.", ["docker", "bugfix"], "127.0.0.1 Docker port mapping"),
    ("pattern", "pmem-host", "The bind host is now read from a PMEM_HOST env var, defaulting to 127.0.0.1 for safety in plain local use; docker-compose.yml sets PMEM_HOST=0.0.0.0 for each service.", ["docker", "config"], "PMEM_HOST environment variable"),
    ("decision", "stdio-not-compose", "pmem stdio is deliberately not a docker-compose service, since it's a process an MCP client spawns and talks to over stdin/stdout, not a long-running network server.", ["docker", "mcp"], "stdio not a compose service"),
    ("decision", "readme-rewrite", "README.md was rewritten from a raw line-count/build-status-table dump into a standard open-source structure: why, features, quick start, command reference, architecture link.", ["docs"], "README rewrite"),
    ("decision", "architecture-doc", "docs/ARCHITECTURE.md documents the storage/CLI/transport layers, replacing a file-by-file source tree that used to live in the README.", ["docs"], "architecture documentation"),
    ("decision", "development-doc", "docs/DEVELOPMENT.md covers both the Docker and plain-cargo local development workflows, including why PMEM_HOST matters.", ["docs"], "development documentation"),
    ("context", "opencode-integration", "OpenCode, an independent third-party coding agent installed via npx opencode-ai, was configured with a standard opencode.json MCP config and confirmed connecting to pmem stdio via 'opencode mcp list'.", ["mcp", "benchmark"], "OpenCode MCP connection"),
    ("context", "token-savings", "memory_context costs about 508 tokens for 12 real facts, versus 34,177 tokens to read the 8 source files containing those facts, roughly 67x cheaper.", ["benchmark", "tokens"], "memory_context token savings"),
    ("context", "search-token-savings", "A single targeted memory_search call costs about 156 tokens, roughly 219x cheaper than reading the source files that contain the answer.", ["benchmark", "tokens"], "memory_search token cost"),
    ("context", "scaling-benchmark", "Performance was measured at 100, 1,000, and 5,000 memories, showing roughly linear growth: 5x the memories produced 5 to 8x the latency for search, list, and stats.", ["benchmark", "performance"], "performance scaling benchmark"),
    ("context", "fulltable-scan", "fuzzy_search and list_tags load the entire memories table into memory before scoring or aggregating client-side, instead of pushing the work into SQL, a known efficiency gap.", ["performance", "knownissue"], "full table scan efficiency"),
    ("context", "binary-size", "The release binary is 9.9 MB.", ["benchmark", "footprint"], "binary size"),
    ("context", "cold-start", "Cold start, process spawn to first MCP response, averages 2.1ms, which matters because an MCP client spawns a fresh pmem stdio process per session.", ["benchmark", "footprint"], "cold start latency"),
    ("context", "encryption-throughput", "Encryption throughput is about 18.4ms per call including process startup, dominated by PBKDF2's 100,000 iterations, deliberately slow by design.", ["benchmark", "security"], "encryption throughput PBKDF2"),
    ("context", "v1-v2-duplication", "The codebase has a recurring v1/v2 module duplication pattern across analytics, export, validation, and search; new requirements got a parallel module instead of extending the original.", ["architecture", "knownissue"], "v1 v2 module duplication"),
    ("context", "encryption-v2-dead", "encryption_v2.rs was dead code whose EncryptionConfig claimed AES-256-GCM and PBKDF2 but actually implemented plain XOR, now removed from the build.", ["security", "knownissue"], "encryption_v2 dead code XOR"),
    ("context", "cli-dead-code", "cli_improvements.rs and cli_ux.rs are dead code, declared modules with zero call sites anywhere in main.rs.", ["knownissue"], "cli_improvements cli_ux dead code"),
    ("context", "main-rs-size", "main.rs is a single file of roughly 2,500+ lines dispatching all 73 CLI subcommands via clap.", ["architecture"], "main.rs command dispatch size"),
    ("context", "schema", "The memories table has columns id, kind, key, content, tags, related_ids, created_at, updated_at, alongside memory_links, memory_versions, and audit_log tables.", ["storage", "schema"], "memories table schema"),
    ("context", "fk-not-enforced", "Foreign key cascades are declared in the schema (ON DELETE CASCADE) but never actually enforced, since PRAGMA foreign_keys is never turned on.", ["storage", "knownissue"], "foreign keys not enforced"),
    ("context", "store-connection", "MemoryStore wraps a single rusqlite Connection per project, opened at .memory/store.db.", ["storage", "architecture"], "MemoryStore single connection"),
    ("context", "get-related-n1", "get_related issues one SQL query per related id in a loop instead of a single batched query, an N+1 pattern.", ["storage", "knownissue"], "get_related N+1 query"),
    ("context", "batch-ops-loop", "batch_delete, batch_add_tag, and batch_update_kind each loop over ids issuing individual queries instead of one batched SQL statement.", ["storage", "knownissue"], "batch operations loop"),
    ("preference", "testing-discipline", "Every fix in this codebase gets verified with a live CLI, HTTP, or browser smoke test, not just cargo test, before being considered done.", ["testing", "preference"], "live testing discipline"),
]

# The actual source/doc files that together contain the 50 facts above.
SOURCE_FILES = [
    "src/main.rs", "src/store.rs", "src/encryption.rs", "src/mcp.rs",
    "src/mcp_stdio.rs", "src/api.rs", "src/dashboard.rs", "src/backup.rs",
    "src/validation.rs", "Dockerfile", "docker-compose.yml", "docker-entrypoint.sh",
    "README.md", "docs/ARCHITECTURE.md", "docs/DEVELOPMENT.md", "docs/BENCHMARK.md",
]


def call(proc, req):
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())


def main():
    with tempfile.TemporaryDirectory() as workdir:
        subprocess.run([BIN, "--project", workdir, "init"], capture_output=True)
        for kind, key, content, tags, _query in MEMORIES:
            subprocess.run([BIN, "--project", workdir, "add", "--kind", kind, "-k", key, "-c", content, "--tags", ",".join(tags)], capture_output=True)

        proc = subprocess.Popen([BIN, "--project", workdir, "stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
        call(proc, {"jsonrpc": "2.0", "id": 1, "method": "initialize"})

        context_resp = call(proc, {"jsonrpc": "2.0", "id": 2, "method": "tools/call",
                                    "params": {"name": "memory_context", "arguments": {}}})
        context_text = context_resp["result"]["content"][0]["text"]
        context_tokens = count_tokens(context_text)

        print(f"=== test 0: memory_context, all {len(MEMORIES)} memories in one call ===")
        print(f"{context_tokens} tokens\n")

        print(f"=== tests 1-{len(MEMORIES)}: one memory_search per fact ===")
        search_token_counts = []
        req_id = 3
        for i, (kind, key, _content, _tags, query) in enumerate(MEMORIES, start=1):
            resp = call(proc, {"jsonrpc": "2.0", "id": req_id, "method": "tools/call",
                                "params": {"name": "memory_search", "arguments": {"query": query, "limit": 3}}})
            req_id += 1
            text = resp["result"]["content"][0]["text"]
            n = count_tokens(text)
            search_token_counts.append(n)
            found_target = key in text
            mark = "ok" if found_target else "MISS"
            print(f"  {i:>2}. [{mark}] {query:<38} {n:>4} tokens")

        proc.terminate()

    total_source_tokens = 0
    print(f"\n=== reading the {len(SOURCE_FILES)} source/doc files that contain these facts ===")
    for rel in SOURCE_FILES:
        path = REPO_ROOT / rel
        text = path.read_text()
        n = count_tokens(text)
        total_source_tokens += n
        print(f"  {rel:<24} {n:>7,} tokens  ({len(text.splitlines())} lines)")
    print(f"  {'total':<24} {total_source_tokens:>7,} tokens")

    print(f"\n=== summary over {len(search_token_counts)} memory_search tests ===")
    print(f"mean:   {statistics.mean(search_token_counts):.1f} tokens")
    print(f"median: {statistics.median(search_token_counts):.1f} tokens")
    print(f"min:    {min(search_token_counts)} tokens")
    print(f"max:    {max(search_token_counts)} tokens")
    print(f"stdev:  {statistics.stdev(search_token_counts):.1f} tokens")

    mean_search = statistics.mean(search_token_counts)
    print("\n=== summary ===")
    print(f"memory_context ({len(MEMORIES)} facts, 1 call): {context_tokens:>7,} tokens  ({context_tokens / total_source_tokens * 100:.1f}% of reading the source)")
    print(f"memory_search (avg of {len(search_token_counts)} queries): {mean_search:>7.1f} tokens  ({mean_search / total_source_tokens * 100:.2f}% of reading the source)")
    print(f"reading source ({len(SOURCE_FILES)} files):    {total_source_tokens:>7,} tokens")
    print(f"\nmemory_context is {total_source_tokens / context_tokens:.0f}x cheaper than reading the source files that contain the same facts.")
    print(f"the average memory_search is {total_source_tokens / mean_search:.0f}x cheaper for a single targeted question.")


if __name__ == "__main__":
    main()
