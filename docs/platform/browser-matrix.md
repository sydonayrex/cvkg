# Browser verification matrix

| Engine | Where | Status |
|---|---|---|
| Chromium 158 / RTX 4070 | Local Linux | Verified: DOM smoke pass, WebGPU path |
| WebKitGTK 2.54.3 | Local Linux | Verified earlier: DOM smoke, fallback UI, input and a11y paths |
| WebKit (macOS) | CI job `browser-matrix / webkit-macos` | Automated, not run on this Linux host |
| WebKit (iPhone 13 emulation) | CI job `browser-matrix / webkit-macos` | Automated, not run on this Linux host |
| Safari / iOS native | Not covered | No Apple device or macOS runner in this environment |

The CI workflow is manual (`workflow_dispatch`) to keep release builds free of
browser downloads. Run it with `gh workflow run browser-matrix.yml`, then read
each job's `npm run smoke` output. The runner script is
`demos/adele-web/scripts/browser_smoke.mjs`; it requires
`demos/adele-web/tests/dom_smoke.html` served on port 8765 and fails on a
non-passing smoke result or any page error.

Playwright's `webkit` engine is the closest automatable proxy for Safari. It is
not a substitute for a native Safari or iOS device pass; keep the last row open
until a macOS runner with Safari is available.
