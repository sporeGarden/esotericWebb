<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Contributing to Esoteric Webb

## Quick start

```bash
make check     # fmt + clippy + test + doc
make deny      # supply chain audit
cargo run --release --bin validate_all  # all experiments
```

## Code quality gates

Every change must pass before merge:

1. **`cargo fmt --all -- --check`** — consistent formatting
2. **`cargo clippy --workspace --all-targets -- -D warnings`** — pedantic + nursery, zero warnings
3. **`cargo test --workspace --lib --tests`** — all unit and integration tests pass
4. **`cargo doc --workspace --no-deps`** — docs compile without warnings
5. **`cargo deny check`** — no advisories, banned crates, or license violations

## Lint policy

- `unsafe_code = "forbid"` — no unsafe anywhere
- `missing_docs = "deny"` — every public item documented
- `unwrap_used = "deny"` / `expect_used = "deny"` — use `?`, `unwrap_or`, or match in library code; `#[allow]` only in test modules
- Clippy pedantic + nursery at warn level — treat all warnings as errors in CI

## Adding an experiment

1. Create `experiments/expNNN_descriptive_name/` with `Cargo.toml` + `src/main.rs`
2. Use the `esoteric_webb::experiment` harness (`check_bool`, `check_skip`, `exit`)
3. Add to `[workspace] members` in root `Cargo.toml`
4. Add to the `EXPERIMENTS` list in `webb/src/bin/validate_all/main.rs`
5. Update the table in `experiments/README.md`

## Filing an evolution gap

When Webb discovers a primal capability that's missing or broken:

1. Add a `GAP-NNN` entry in `EVOLUTION_GAPS.md` with evidence
2. Create a wateringHole handoff in `wateringHole/handoffs/` when ready
3. The owning spring picks it up, evolves, rebuilds, deploys to `plasmidBin/`
4. Webb absorbs via capability discovery — the gap closes

## Commit style

- Short imperative subject line (50 chars)
- Body explains **why**, not what
- Reference GAP/exp numbers when relevant

## Content authoring (for creatives)

Game content lives in `content/` as YAML — no Rust required.
See `specs/CONTENT_AUTHORING_SPEC.md` for the format and
`esotericwebb validate` to check your work.

## License

- Code: AGPL-3.0-or-later (add `// SPDX-License-Identifier: AGPL-3.0-or-later` to new `.rs` files)
- Docs: CC-BY-SA-4.0
- Game mechanics: ORC
