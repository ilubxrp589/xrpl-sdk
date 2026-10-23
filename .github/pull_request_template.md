## What this changes

<!-- What it does for someone using the SDK, and why. Link the issue. -->

## How it was checked

<!-- Tests added; for anything that must agree with the ledger, the rippled answer it was checked against. -->

## Checklist

- [ ] Tests cover the change, and `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` and `cargo fmt --check` are clean
- [ ] Every new public item has a doc comment (with an example where it helps)
- [ ] CHANGELOG.md has a line under **Unreleased** (and says so if it breaks the API)
- [ ] No seed, key, phrase, API key or private URL anywhere in the change
