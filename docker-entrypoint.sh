#!/bin/sh
set -e
# Idempotent: creates .memory/ on first run, no-ops if it already exists.
pmem --project /workspace init >/dev/null 2>&1 || true
exec pmem --project /workspace "$@"
