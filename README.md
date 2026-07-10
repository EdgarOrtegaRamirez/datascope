# DataScope

**Fast data profiling CLI for CSV, TSV, JSON, and JSONL files — type inference, statistics, quality checks, and pattern detection.**

[![CI](https://github.com/EdgarOrtegaRamirez/datascope/actions/workflows/ci.yml/badge.svg)](https://github.com/EdgarOrtegaRamirez/datascope/actions/workflows/ci.yml)

## What It Does

DataScope analyzes your data files and gives you a comprehensive profile of each column:

- **Type inference** — Detects whether each column is integer, float, boolean, date, datetime, or string
- **Statistics** — Computes min, max, mean, median, standard deviation, quartiles, and sum for numeric columns
- **Null analysis** — Counts null/empty values and calculates percentages
- **Cardinality** — Unique value counts and percentages
- **Top values** — Most frequent values with counts and percentages
- **Pattern detection** — Identifies emails, URLs, IPv4/IPv6 addresses, UUIDs, phone numbers, credit cards, and zip codes
- **Quality scoring** — Grades your data A–F based on detected issues (high null rates, mixed types, outliers, constant columns, high cardinality)
- **CI-friendly** — `--strict` flag exits non-zero on high-severity quality issues

## Quick Start

```bash
# Profile a CSV file
datascope data.csv

# Output as JSON for piping to other tools
datascope data.csv -o json | jq '.columns[] | .name'

# Profile from stdin
cat data.csv | datascope - -f csv

# Summary only (compact table)
datascope data.csv --summary-only

# Fail CI on quality issues
datascope data.csv --strict

# Profile JSONL data
datascope events.jsonl -o json
```

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
git clone https://github.com/EdgarOrtegaRamirez/datascope.git
cd datascope
cargo build --release
# Binary at target/release/datascope
```

## Supported Formats

| Format | Extension | `--format` flag |
|--------|-----------|-----------------|
| CSV    | `.csv`    | `csv`           |
| TSV    | `.tsv`, `.tab` | `tsv`     |
| JSON   | `.json`   | `json`          |
| JSONL  | `.jsonl`, `.ndjson` | `jsonl` |

Format is auto-detected from the file extension. Use `--format` to override or when reading from stdin.

## CLI Options

```
datascope [OPTIONS] <FILE>

Arguments:
  <FILE>                    Input file path, or "-" for stdin

Options:
  -f, --format <FORMAT>     Input format: csv, tsv, json, jsonl (auto-detected from extension)
  -o, --output <FORMAT>     Output format: text, json, csv (default: text)
  -n, --top <N>             Number of top values per column (default: 10)
      --no-patterns         Disable pattern detection
      --no-quality          Disable quality checks
      --strict              Exit non-zero if high-severity quality issues found
      --summary-only        Print only the summary table (text output only)
  -h, --help                Print help
  -V, --version             Print version
```

## Output Formats

### Text (default)

Human-readable report with per-column details, statistics, top values, and quality issues.

### JSON

Structured JSON output suitable for programmatic use, piping to `jq`, or integration with other tools.

```bash
datascope data.csv -o json | jq '.columns[] | {name, type: .data_type, nulls: .null_percentage}'
```

### CSV

Compact CSV summary — one row per column with key metrics. Ideal for comparing multiple datasets.

```bash
datascope data1.csv -o csv > profile1.csv
datascope data2.csv -o csv > profile2.csv
```

## Data Quality Checks

DataScope detects and reports the following quality issues:

| Issue | Severity | Description |
|-------|----------|-------------|
| `high_null_rate` | high | >50% of values are null/empty |
| `moderate_null_rate` | medium | >20% of values are null/empty |
| `mixed_types` | medium | Column has mixed numeric and string values |
| `outliers` | medium | Statistical outliers detected (IQR method) |
| `constant_column` | low | All non-null values are identical |
| `high_cardinality` | low | All values are unique (possible ID column) |
| `empty_column` | high | Column has no values at all |

The overall quality score (0–100) is computed from these issues and mapped to a letter grade (A–F).

## Architecture

```
src/
├── main.rs          # CLI entry point, I/O, orchestration
├── cli.rs           # Argument parsing (clap)
├── error.rs         # Error types
├── format/          # Input format readers
│   ├── mod.rs       # Format detection, DataSource struct
│   ├── csv_reader.rs # CSV/TSV reader
│   └── json_reader.rs # JSON/JSONL reader
├── infer.rs         # Type inference engine
├── stats.rs         # Statistical calculations (min, max, mean, median, std dev, quartiles, outliers)
├── pattern.rs       # Pattern detection (email, URL, IP, UUID, phone, etc.)
├── profile.rs       # Profiling engine, quality checks, scoring
└── output/          # Output renderers
    ├── mod.rs       # Output format dispatch
    ├── text.rs      # Human-readable text output
    ├── json_output.rs # JSON output
    └── csv_output.rs  # CSV summary output
```

## Security

- No network access — all processing is local
- Input file size capped at 500 MB to prevent memory exhaustion
- No code execution or eval — pure data parsing
- No secrets or credentials stored or transmitted
- See [SECURITY.md](SECURITY.md) for responsible disclosure

## License

MIT
