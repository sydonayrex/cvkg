# Browser verification

Native Safari and iOS verification is out of scope for this repository.

| Engine | Where | Status |
|---|---|---|
| Chromium (Playwright) | Local Linux | Verified: DOM smoke pass, 21 checks |
| Chromium (Playwright) | CI job `browser smoke / chromium-linux` | Automated on `workflow_dispatch` |
| WebKit 27.2 / WPE 2.55.0 | Local Linux | Previously verified with a local ICU shim; not in CI |
| Safari / iOS native | Not covered | Out of scope |

The CI workflow is manual (`workflow_dispatch`) to keep release builds free of
browser downloads. Run it with `gh workflow run browser-matrix.yml`, then read
the `chromium-linux` job's `npm run smoke` output. The runner script is
`demos/adele-web/scripts/browser_smoke.mjs`; it requires
`demos/adele-web/tests/dom_smoke.html` served on port 8765 and fails on a
non-passing smoke result or any page error.
