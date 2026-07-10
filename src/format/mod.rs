pub mod csv_reader;
pub mod json_reader;

use crate::error::Result;

/// Supported input formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    Csv,
    Tsv,
    Json,
    Jsonl,
}

impl InputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputFormat::Csv => "csv",
            InputFormat::Tsv => "tsv",
            InputFormat::Json => "json",
            InputFormat::Jsonl => "jsonl",
        }
    }
}

/// Detect format from file extension.
pub fn detect_format(filename: &str) -> Option<InputFormat> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".csv") {
        Some(InputFormat::Csv)
    } else if lower.ends_with(".tsv") || lower.ends_with(".tab") {
        Some(InputFormat::Tsv)
    } else if lower.ends_with(".jsonl") || lower.ends_with(".ndjson") {
        Some(InputFormat::Jsonl)
    } else if lower.ends_with(".json") {
        Some(InputFormat::Json)
    } else {
        None
    }
}

/// A row of data: column name → value.
pub type Row = Vec<String>;

/// A data source that yields rows with headers.
pub struct DataSource {
    pub headers: Vec<String>,
    pub rows: Vec<Row>,
}

/// Read data from a reader in the specified format.
pub fn read_data(reader: &mut dyn std::io::Read, format: InputFormat) -> Result<DataSource> {
    match format {
        InputFormat::Csv => csv_reader::read_csv(reader, b','),
        InputFormat::Tsv => csv_reader::read_csv(reader, b'\t'),
        InputFormat::Json => json_reader::read_json(reader),
        InputFormat::Jsonl => json_reader::read_jsonl(reader),
    }
}
