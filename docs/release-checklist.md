# Release Checklist (Manual Fallback)

Use this checklist when automation is unavailable or a workflow fails.

## Prerequisites

- `camgylph` repo clean on `main`
- `homebrew-tap` repo clean on `main`
- `gh` authenticated for `landxcape`

## One-Time Automation Setup

Set repo secret in `landxcape/camgylph`:

- Name: `HOMEBREW_TAP_TOKEN`
- Value: GitHub token with `repo` access to `landxcape/homebrew-tap`

Without this secret, tag automation still validates the release, but Homebrew tap sync is skipped.

## Manual Release Steps

1. Bump version in `Cargo.toml`.
2. Run checks:
   - `cargo fmt`
   - `cargo check`
   - `cargo test`
3. Commit and tag:
   - `git add ...`
   - `git commit -m "Release vX.Y.Z"`
   - `git tag -a vX.Y.Z -m "vX.Y.Z"`
4. Push:
   - `git push origin main`
   - `git push origin vX.Y.Z`
5. Create GitHub release:
   - `gh release create vX.Y.Z --repo landxcape/camgylph --title "vX.Y.Z" --notes-file <notes.md>`

## Manual Homebrew Tap Update

1. Compute tarball checksum:
   - `curl -L https://github.com/landxcape/camgylph/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256`
2. Update `homebrew-tap/Formula/camgylph.rb`:
   - `url` -> `.../vX.Y.Z.tar.gz`
   - `sha256` -> computed checksum
3. Commit and push tap:
   - `git add Formula/camgylph.rb`
   - `git commit -m "camgylph X.Y.Z"`
   - `git push origin main`
4. Verify install path:
   - `brew upgrade landxcape/tap/camgylph`
   - `brew info landxcape/tap/camgylph`
