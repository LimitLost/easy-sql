# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repo.

`easy-sql` — a macro-first SQL toolkit (compile-time-checked queries + optional
migrations) built on sqlx. Rust only.

## Quality Standard

Precision top priority. No shortcuts, no deferred polish, no skipped verification. Speed irrelevant.

- Every claim traceable to fetched source with line numbers.
- Every fix verified empirically, not inferred.
- Never trust agent summary — verify actual artifacts.
- When doubt: extra fetch, extra count, extra comparison.

## Repository layout

Four independent crates (not a Cargo workspace):

- `-main/` → `easy-sql` — primary library crate (drivers, markers, migrations, tests).
- `macros/` → `easy-sql-macros` — proc-macros (`query!`, `#[derive(Table)]`, …).
- `build/` → `easy-sql-build` — build-time helper crate.
- `compilation-data/` → `easy-sql-compilation-data` — shared base used by `macros` and `build`.

Gotcha: the main crate dir is literally named `-main` (leading dash), which breaks bare
shell commands — use `./-main` or `--` (e.g. `ls -- -main`, `cd ./-main`).

## Documentation conventions (load-bearing)

- Every public item (Rust `///`, modules `//!`) opens with a one-sentence purpose, then a
  `Reason: ...` line explaining the design choice. Reason justifies existence.
- Document the steps inside a function: sequential steps use `// Step N: ...`; a set of
  non-sequential items inside one comment block uses `// - ...` per item.
- Document struct **fields** / enum **variants**, not the type as a whole. Each enum
  (tagged-union) variant describes its own distinguished case.
- Docs should read faster than the code. `Reason:` is exempt — it may be longer when needed.

## Structure rules

- A named subdomain with 2+ related functions/types/structs → its own file/submodule
  (heuristic, not a hard trigger). E.g. a file holding both `generate` logic and its
  settings types → move the settings into a `settings` submodule.
- Test-code submodules go in a sibling `tests.rs` file — never an inline
  `#[cfg(test)] mod tests {}` left in a production source file. (The broad
  general/integration suite lives in the `src/tests/` module tree; `#[cfg(test)]` on
  production items — fields, impls, test-only helpers — is a separate, allowed idiom, not
  a test submodule.)

## Workflow rules

- Use `cargo clippy` (not `cargo build`) to check for warnings.
- Run tests via the scripts, not `cargo test` directly: `./scripts/test-all.sh`, or
  `./scripts/test-specific.sh <name-fragment>`. Feature flags are listed in each script's
  `# Usage:` header (e.g. `--migrations`, `--math`); if a needed option is missing, add it
  to the script rather than hand-rolling `cargo test`. Run scripts **without** a `bash`/`sh`
  prefix (`./scripts/test-all.sh`, not `bash ./scripts/…`) to avoid an approval prompt.
- Before using a dependency, read its docs/source (locally, or `https://docs.rs/<crate>`).
- `#[always_context]`: if it errors because a field lacks `Debug`, first try implementing
  `Debug`; otherwise mark that field `#[context(no)]`; if no field on the invocation can be
  used, put `#[no_context_inputs]` on the invocation (or one level up, to minimize repetition).

## Other rules

- Names: prefer short with a clarifying comment over long self-explaining.
- Never revert unrelated dirty work-tree changes during a focused edit.
- Never run `git add` / `git commit` — the user reviews first.
- Document every env var in `.env.example` (at `-main/.env.example`) with a description.
