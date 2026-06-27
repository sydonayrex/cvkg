#!/usr/bin/env bash
set -euo pipefail
if grep -rnE 'path\s*=\s*"[^"]*".*version\s*=\s*"[0-9]' demos/*/Cargo.toml; then
  echo "ERROR: demo crate pins an internal dependency by path AND version."
  echo "Use \`crate-name.workspace = true\` instead. See Task 0.5 of the implementation plan."
  exit 1
fi
