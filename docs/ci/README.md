# CI workflow source

`fidelity.yml` belongs at `.github/workflows/fidelity.yml`.

This token cannot create workflow files (`workflow` OAuth scope). After
`gh auth refresh -s workflow`, copy it:

```bash
mkdir -p .github/workflows
cp docs/ci/fidelity.yml .github/workflows/fidelity.yml
git add .github/workflows/fidelity.yml
git commit -m "ci: enable Phase 0 fidelity workflow"
git push
```
