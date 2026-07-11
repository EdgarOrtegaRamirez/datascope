mod cli;
mod error;
mod format;
mod infer;
mod output;
mod pattern;
mod profile;
mod schema;
mod stats;

use clap::Parser;
use std::io::{self, Read};
use std::process::ExitCode;

use cli::{Cli, FormatArg, OutputArg};
use error::DatascopeError;
use format::InputFormat;
use output::OutputFormat;
use profile::ProfileOptions;

fn main() -> ExitCode {
    let args = Cli::parse();

    match run(&args) {
        Ok(has_issues) => {
            if args.strict && has_issues {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Cli) -> Result<bool, DatascopeError> {
    // Determine input format
    let input_format = match args.format {
        Some(FormatArg::Csv) => InputFormat::Csv,
        Some(FormatArg::Tsv) => InputFormat::Tsv,
        Some(FormatArg::Json) => InputFormat::Json,
        Some(FormatArg::Jsonl) => InputFormat::Jsonl,
        None => {
            // Auto-detect from filename (not for stdin)
            if args.input == "-" {
                return Err(DatascopeError::InvalidInput(
                    "format is required when reading from stdin (use --format)".to_string(),
                ));
            }
            format::detect_format(&args.input).ok_or_else(|| {
                DatascopeError::InvalidInput(format!(
                    "could not detect format from filename '{}'. Use --format to specify.",
                    args.input
                ))
            })?
        }
    };

    // Read input
    let mut input_data: Vec<u8> = Vec::new();
    if args.input == "-" {
        io::stdin()
            .read_to_end(&mut input_data)
            .map_err(|e| DatascopeError::Io(e.to_string()))?;
    } else {
        std::fs::File::open(&args.input)
            .map_err(|e| DatascopeError::Io(format!("cannot open '{}': {e}", args.input)))?
            .read_to_end(&mut input_data)
            .map_err(|e| DatascopeError::Io(e.to_string()))?;
    }

    // Validate file size (prevent loading huge files into memory)
    const MAX_SIZE: usize = 500 * 1024 * 1024; // 500 MB
    if input_data.len() > MAX_SIZE {
        return Err(DatascopeError::InvalidInput(format!(
            "input file is {} MB, exceeds maximum of {} MB",
            input_data.len() / (1024 * 1024),
            MAX_SIZE / (1024 * 1024)
        )));
    }

    let mut reader = io::Cursor::new(input_data);
    let data = format::read_data(&mut reader, input_format)?;

    // If --schema is specified, generate and output JSON Schema
    if args.schema {
        let json_schema = schema::generate_schema(&data);
        println!("{json_schema}");
        return Ok(false);
    }

    // Profile
    let options = ProfileOptions {
        top_values_limit: args.top_values,
        detect_patterns: !args.no_patterns,
        check_quality: !args.no_quality,
    };

    let profile_result = profile::profile_dataset(&data, input_format.as_str(), &options)?;

    // Determine output format
    let output_format = match args.output {
        OutputArg::Text => OutputFormat::Text,
        OutputArg::Json => OutputFormat::Json,
        OutputArg::Csv => OutputFormat::Csv,
    };

    // Render output
    if args.summary_only && output_format == OutputFormat::Text {
        print_summary(&profile_result);
    } else {
        let rendered = output::render(&profile_result, output_format);
        println!("{rendered}");
    }

    // Check for quality issues (for --strict)
    let has_issues = profile_result
        .columns
        .iter()
        .any(|c| c.quality_issues.iter().any(|i| i.severity == "high"));

    Ok(has_issues)
}

fn print_summary(profile: &profile::DatasetProfile) {
    println!("Format:      {}", profile.format);
    println!("Rows:         {}", profile.row_count);
    println!("Columns:      {}", profile.column_count);
    println!(
        "Quality:     {}/100 ({})",
        profile.quality_score.round() as i64,
        profile.quality_grade
    );
    println!();
    println!(
        "{:<30} {:<12} {:>6} {:>8}",
        "Column", "Type", "Nulls", "Unique"
    );
    println!("{}", "─".repeat(60));
    for col in &profile.columns {
        println!(
            "{:<30} {:<12} {:>5.1}% {:>7.1}%",
            truncate(&col.name, 30),
            col.data_type,
            col.null_percentage,
            col.unique_percentage
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
