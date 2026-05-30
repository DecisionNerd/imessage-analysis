# Releasing

Releases are triggered by pushing a version tag. The CI pipeline handles everything else.

## Prerequisites

- The `HOMEBREW_TAP_TOKEN` secret must be set on the `imessage-analysis` repository (a GitHub PAT with `repo` scope on `DecisionNerd/homebrew-tap`). This was set up during initial project configuration.
- The `DecisionNerd/homebrew-tap` repository must exist with a `Formula/imessage-analysis.rb` file on its `main` branch.

## Cutting a release

```sh
# Make sure you're on main and the working tree is clean
git checkout main
git pull

# Tag and push — this triggers the release workflow
git tag v0.2.0
git push origin v0.2.0
```

## What the workflow does

`.github/workflows/release.yml` runs on `macos-14` (arm64) and:

1. Builds both release binaries: `imessage-analysis` and `imessage-mcp`
2. Creates a source tarball with `git archive`
3. Computes the SHA256 of the tarball
4. Creates a GitHub Release and uploads the tarball + binaries as assets
5. Clones `DecisionNerd/homebrew-tap`, creates a branch `update-imessage-analysis-<version>`, patches the formula's `url` and `sha256`, and pushes
6. Opens a pull request against `homebrew-tap` via the GitHub API

## Merging the formula PR

After the workflow completes, a PR appears in `github.com/DecisionNerd/homebrew-tap`. Review and merge it. Once merged, users on the tap will receive the update on their next `brew upgrade`.

## Version numbering

This project follows [Semantic Versioning](https://semver.org):

- **Patch** (`v0.1.x`) — bug fixes, no schema changes
- **Minor** (`v0.x.0`) — new features, backward-compatible
- **Major** (`vx.0.0`) — breaking changes to the CLI interface or Parquet schema

When the Parquet schema changes (new or renamed columns), increment `schema_version` in `crates/imessage-core/src/storage/metadata.rs` so that stale datasets are detected and users are prompted to re-run `sync`.

## Homebrew tap structure

```
github.com/DecisionNerd/homebrew-tap
└── Formula/
    └── imessage-analysis.rb   # Updated automatically on each release
```

To install from the tap:
```sh
brew tap DecisionNerd/tap
brew install imessage-analysis
```
