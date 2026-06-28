#!/usr/bin/env bash
# publish_remaining.sh — publishes remaining CVKG 0.2.15 crates in dependency order.
# Run from the workspace root: bash scripts/publish_remaining.sh
set -euo pipefail

WORKSPACE="/D/rex/projects/cvkg"
cd "$WORKSPACE"

# Crates already published in this session (skip them).
ALREADY_DONE=(
  cvkg-runic-text
  cvkg-materials
  cvkg-reflect
  cvkg-core
  cvkg-svg-filters
  cvkg-svg-serialize
  cvkg-compositor
  cvkg-anim
  cvkg-scheduler
  cvkg-spatial
  cvkg-accessibility
  cvkg-layout
  cvkg-render-software
  cvkg-telemetry
  cvkg-themes
  cvkg-scene
  cvkg-render-gpu
  cvkg-physics
  cvkg-vdom
  cvkg-certification
  cvkg-flow
  cvkg-export-raster
  cvkg-render-native
)

# Crates with publish = false — never publish these.
NO_PUBLISH=(
  cvkg-test
  cvkg-gallery
  cvkg-game-hud
  adele-web-demo
  berserker
  berserker-fire-web-demo
  niflheim-wasi-demo
)

# Remaining crates in strict topological order.
# cvkg-macros is moved AFTER cvkg-components because its dev-dependencies
# require cvkg-components = "0.2.15" which must be on crates.io first.
PUBLISH_ORDER=(
  cvkg-themes          # deps: anim, core
  cvkg-scene           # deps: core, runic-text, spatial
  cvkg-render-gpu      # deps: compositor, core, runic-text, svg-filters, svg-serialize
  cvkg-physics         # deps: core, scene
  cvkg-vdom            # deps: core, runic-text, scene
  cvkg-certification   # deps: core, runic-text, scene, spatial, svg-serialize, themes
  cvkg-flow            # deps: core, scene, themes
  cvkg-export-raster   # deps: render-gpu
  cvkg-render-native   # deps: core, render-gpu, themes, vdom, runic-text
  cvkg-macros          # runtime: only proc-macro crates; dev-deps patched to >=0.2.14
  cvkg-components      # deps: anim, core, layout, runic-text, themes, vdom, macros
  cvkg-icons           # deps: components, core
  cvkg-cli             # deps: anim, core, export-raster, macros, physics
  cvkg-webkit-server   # deps: cli
  cvkg               # umbrella crate — depends on all of the above
)

# ── Helpers ──────────────────────────────────────────────────────────────────

is_in() {
  local item="$1"; shift
  for x in "$@"; do [[ "$x" == "$item" ]] && return 0; done
  return 1
}

publish_crate() {
  local name="$1"

  if is_in "$name" "${ALREADY_DONE[@]}"; then
    echo "  [SKIP] $name — already published this session"
    return 0
  fi

  if is_in "$name" "${NO_PUBLISH[@]}"; then
    echo "  [SKIP] $name — publish = false"
    return 0
  fi

  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  Publishing: $name"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  local output
  if output=$(cargo publish --allow-dirty -p "$name" 2>&1); then
    echo "$output"
    echo "  ✅ $name published successfully"
  else
    local exit_code=$?
    echo "$output"
    # If crates.io says already uploaded, treat as success.
    if echo "$output" | grep -qE "already exists|already uploaded"; then
      echo "  [SKIP] $name — version 0.2.15 already on crates.io"
    else
      echo "  ❌ $name FAILED (exit $exit_code)"
      echo "     Aborting. Fix the issue above then re-run the script."
      exit "$exit_code"
    fi
  fi
}

# ── Main ─────────────────────────────────────────────────────────────────────

echo "=== CVKG 0.2.15 — Publishing remaining crates ==="
echo "=== $(date) ==="
echo ""

for crate in "${PUBLISH_ORDER[@]}"; do
  publish_crate "$crate"
done

echo ""
echo "=== All done! ==="
