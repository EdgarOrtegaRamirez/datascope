use clap::{Parser, ValueEnum};

/// Fast data profiling CLI for CSV, TSV, JSON, and JSONL files.
///
/// Analyzes data files to infer column types, compute statistics,
/// detect data quality issues, and identify value patterns.
#[derive(Parser, Debug)]
#[command(
    name = "datascope",
    version,
    about,
    long_about = None,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Input file path. Use "-" for stdin.
    #[arg(value_name = "FILE")]
    pub input: String,

    /// Input format. Auto-detected from extension if omitted.
    #[arg(short = 'f', long = "format", value_enum)]
    pub format: Option<FormatArg>,

    /// Output format.
    #[arg(short = 'o', long = "output", value_enum, default_value = "text")]
    pub output: OutputArg,

    /// Number of top values to show per column.
    #[arg(short = 'n', long = "top", default_value = "10")]
    pub top_values: usize,

    /// Disable pattern detection.
    #[arg(long = "no-patterns")]
    pub no_patterns: bool,

    /// Disable quality checks.
    #[arg(long = "no-quality")]
    pub no_quality: bool,

    /// Exit with non-zero code if quality issues are found.
    #[arg(long = "strict")]
    pub strict: bool,

    /// Print only the summary (no per-column details).
    #[arg(long = "summary-only")]
    pub summary_only: bool,

    /// Generate a JSON Schema (Draft 2020-12) instead of a data profile.
    #[arg(long = "schema")]
    pub schema: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum FormatArg {
    Csv,
    Tsv,
    Json,
    Jsonl,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputArg {
    Text,
    Json,
    Csv,
}
