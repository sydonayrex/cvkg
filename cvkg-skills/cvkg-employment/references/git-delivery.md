# CVKG Git Delivery Pattern

Use this pattern when the user asks to commit and push the project.

## Sequence

```bash
git status --short
git add -A
git commit -m "<concise summary>"
git push origin HEAD:main -v
```

## Notes

- Inspect status before staging so you can answer if the worktree contains unrelated changes.
- Use `git add -A` only when the user means the whole project.
- Use a concise commit message that describes the actual work.
- Push to `main` explicitly with `git push origin HEAD:main -v`.
- If `git push HEAD:main` fails by trying the wrong remote, retry with the explicit remote. That retry pattern is durable; the original failure is just a setup symptom.

## Verification

After push, report the result directly:

```text
Commit: <short hash or message>
Push: origin HEAD:main -> main
Status: up-to-date or pushed
```
