# Dev notes

## Releasing new version

### 1. update release docs

First, make sure git work tree is clean - commit or stash all the changes. Then, ensure
`README.md` and `CHANGELOG.md` are up to date:

```sh
cargo xtask update-docs
```

Then edit `CHANGELOG.md` if needed.

⚠️ Do not change `## unreleased` section header, this will be handled automatically by
release process. If `## unreleased` is missing from `CHANGELOG.md`, `cargo xtask
update-docs` will fail. The heading must be exactly `## unreleased` (lowercase, no
trailing whitespace).

Once satisfied, commit your changes, if there are any.

```sh
git add README.md CHANGELOG.md
git commit -m "build: updated docs for next release."
```

⚠️ `git` message for this commit must be prefixed by `build:`

### 2. Run cargo-release

```sh
cargo release patch             # or: minor / major; this is dry run for preview only
cargo release patch --execute   # or: minor / major; actually updates version
```

This produces a **single commit** that:
- Renames `## unreleased` → `## vX.Y.Z (YYYY-MM-DD)` in `CHANGELOG.md` and re-seeds a
  blank `## unreleased` heading for the next cycle
- Bumps the version in `Cargo.toml` / `Cargo.lock`
- Creates the git tag `vX.Y.Z`

The commit message will be `build: bumped version to vX.Y.Z`.

### 3. Push

```sh
git push && git push --tags
```

This pushes both the commit and the tag. The `GitHub Actions` release workflow fires on
the tag, extracts the `## vX.Y.Z (...)` section from `CHANGELOG.md` as the release body,
and appends a **Full Changelog** comparison link.

`push = false` in the workspace `Cargo.toml` means `cargo-release` never pushes
automatically. Always use `git push --follow-tags`.

If `CHANGELOG.md` does not contain a `## vX.Y.Z (...)` section matching the pushed tag,
the `GitHub` release is created with an empty body. If you'd followed above instructions
correctly, this should never happen.
