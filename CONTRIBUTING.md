# Contributing to HartLab

Same branch split as Crownfall / Hearth.

```
feature / spike / fix  →  development  →  main
         PR                    PR after smoke
```

| Branch | Role |
|--------|------|
| `development` | Integration. **Default branch.** Open feature PRs here. |
| `main` | Production. Merge from `development` only after smoke, or when explicitly shipping. |
| `spike/*`, `feat/*`, `fix/*` | Short-lived work. Never target `main`. |

## Rules

1. Open feature/fix PRs against **`development`**, not `main`.
2. Merge into **`development` first** so the change can be smoked (local
   `./tests/fidelity/run-live.sh`, later the Vercel/session preview).
3. Merge **`development` → `main` only after** that smoke, or when asked to
   ship. `main` is the live site once https://hartlab.vesperforge.org exists.
4. If `development` is behind `main`, update `development` from `main` first,
   then land the feature there. Do not merge a feature straight to `main`
   just because `development` lagged.

```bash
git fetch origin
git checkout -b spike/short-name origin/development
# ...
gh pr create --base development
```

Promote when ready:

```bash
gh pr create --base main --head development --title "Promote development to main"
```
