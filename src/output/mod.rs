pub mod csv_output;
pub mod json_output;
pub mod text;

use crate::profile::DatasetProfile;

/// Output format enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

/// Render a profile in the specified format.
pub fn render(profile: &DatasetProfile, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => text::render_text(profile),
        OutputFormat::Json => json_output::render_json(profile),
        OutputFormat::Csv => csv_output::render_csv(profile),
    }
}
