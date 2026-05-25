# Dev notes

## Releasing new version

```sh
cargo xtask update-readme
git add README.md
git commit
```

```sh
cargo release patch # dry run by default
cargo release patch --execute # commits and tags
```
