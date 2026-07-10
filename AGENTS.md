# DataScope — AI Agent Guide

## Project Overview
DataScope is a Rust CLI tool for data profiling. It reads CSV, TSV, JSON, and JSONL files, infers column types, computes statistics, detects data quality issues, and identifies value patterns.

## Build & Test
```bash
# Build
cargo build

# Run tests (29 unit + 20 integration = 49 total)
cargo test

# Lint
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Run
cargo run -- tests/fixtures/sample.csv
cargo run -- tests/fixtures/sample.csv -o json
cat data.csv | cargo run -- - -f csv
```

## Architecture
- `src/main.rs` — CLI entry, I/O, orchestration
- `src/cli.rs` — clap argument parsing
- `src/format/` — Input readers (CSV/TSV via `csv` crate, JSON/JSONL via `serde_json`)
- `src/infer.rs` — Type inference (integer, float, boolean, date, datetime, string, null, empty)
- `src/stats.rs` — Numeric statistics and outlier detection (IQR method)
- `src/pattern.rs` — Pattern detection (email, URL, IPv4/IPv6, UUID, phone, credit card, zip code)
- `src/profile.rs` — Profiling engine, quality checks, quality scoring
- `src/output/` — Output renderers (text, JSON, CSV)

## Key Design Decisions
- All processing is local — no network access
- Input size capped at 500 MB to prevent memory exhaustion
- JSON objects use BTreeMap (alphabetical key order) — tests must not assume insertion order
- Quality score: 0–100, letter grade A–F, based on issue severity penalties
- `--strict` exits non-zero only on high-severity issues

## Dependencies
- clap 4.6 — CLI parsing
- serde 1.0 + serde_json 1.0 — JSON serialization
- csv 1.4 — CSV/TSV parsing
- regex 1.13 — Pattern detection
- anyhow 1.0 — Error handling (dev only)

## Test Fixtures
- `tests/fixtures/sample.csv` — 10 rows, 6 columns (name, age, email, salary, active, join_date)
- `tests/fixtures/sample.json` — 3 objects
- `tests/fixtures/sample.jsonl` — 5 lines (includes nulls)
- `tests/fixtures/sample.tsv` — 3 rows
