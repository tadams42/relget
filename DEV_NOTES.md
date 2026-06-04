# Dev notes

## Releasing new version

### 1. Update docs

Regenerate the README app list and populate `## unreleased` with commits since the last tag:

```sh
cargo xtask update-docs
```

Safe to run multiple times — both operations are idempotent.

Review `CHANGELOG.md` and curate the bullet list as needed. **Do not rename the `## unreleased`
heading** — cargo-release does that automatically in the next step.

Stage the changes:

```sh
git add CHANGELOG.md README.md
```

### 2. Run cargo-release

Dry run (no changes):

```sh
cargo release patch   # or: minor / major
```

Execute when satisfied:

```sh
cargo release patch --execute
```

This produces a **single commit** that:
- Renames `## unreleased` → `## vX.Y.Z (YYYY-MM-DD)` in CHANGELOG.md (and re-seeds a blank
  `## unreleased` heading above it for the next cycle)
- Bumps the version in Cargo.toml / Cargo.lock
- Creates the git tag `vX.Y.Z`

The commit message will be `build: Bumped version to vX.Y.Z`.

### 3. Push

```sh
git push --follow-tags
```

This pushes both the commit and the tag. The GitHub Actions release workflow fires on the tag,
extracts the `## vX.Y.Z (...)` section from CHANGELOG.md as the release body, and appends a
**Full Changelog** comparison link.

---

## Notes

- `push = false` in the workspace Cargo.toml means cargo-release never pushes automatically.
  Always use `git push --follow-tags`.
- If `## unreleased` is missing from CHANGELOG.md, both `cargo xtask update-changelog` and
  `cargo release` will fail. The heading must be exactly `## unreleased` (lowercase, no trailing
  whitespace).
- If CHANGELOG.md does not contain a `## vX.Y.Z (...)` section matching the pushed tag, the
  GitHub release is created with an empty body.
