use crate::error::Result;
use crate::format::DataSource;
use crate::infer::{self, DataType};
use crate::pattern;
use crate::stats;
use std::collections::HashMap;

/// Profile of a single column.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnProfile {
    pub name: String,
    pub data_type: String,
    pub count: usize,
    pub null_count: usize,
    pub null_percentage: f64,
    pub unique_count: usize,
    pub unique_percentage: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric_stats: Option<stats::NumericStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_values: Option<Vec<ValueCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<String>,
    pub quality_issues: Vec<QualityIssue>,
}

/// A value and its frequency count.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValueCount {
    pub value: String,
    pub count: usize,
    pub percentage: f64,
}

/// A data quality issue found in a column.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QualityIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: String,
}

/// Overall profile of a dataset.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatasetProfile {
    pub format: String,
    pub row_count: usize,
    pub column_count: usize,
    pub columns: Vec<ColumnProfile>,
    pub quality_score: f64,
    pub quality_grade: String,
}

/// Options for profiling.
#[derive(Debug, Clone)]
pub struct ProfileOptions {
    pub top_values_limit: usize,
    pub detect_patterns: bool,
    pub check_quality: bool,
}

impl Default for ProfileOptions {
    fn default() -> Self {
        Self {
            top_values_limit: 10,
            detect_patterns: true,
            check_quality: true,
        }
    }
}

/// Profile a dataset.
pub fn profile_dataset(
    data: &DataSource,
    format_name: &str,
    options: &ProfileOptions,
) -> Result<DatasetProfile> {
    let row_count = data.rows.len();
    let column_count = data.headers.len();

    let mut columns: Vec<ColumnProfile> = Vec::with_capacity(column_count);

    for (col_idx, header) in data.headers.iter().enumerate() {
        let col_profile = profile_column(data, col_idx, header, options);
        columns.push(col_profile);
    }

    // Compute overall quality score
    let quality_score = compute_quality_score(&columns);
    let quality_grade = quality_grade(quality_score);

    Ok(DatasetProfile {
        format: format_name.to_string(),
        row_count,
        column_count,
        columns,
        quality_score,
        quality_grade,
    })
}

/// Profile a single column.
fn profile_column(
    data: &DataSource,
    col_idx: usize,
    header: &str,
    options: &ProfileOptions,
) -> ColumnProfile {
    // Collect all values for this column
    let values: Vec<&str> = data
        .rows
        .iter()
        .filter_map(|row| row.get(col_idx).map(|s| s.as_str()))
        .collect();

    let count = values.len();
    if count == 0 {
        return ColumnProfile {
            name: header.to_string(),
            data_type: "null".to_string(),
            count: 0,
            null_count: 0,
            null_percentage: 0.0,
            unique_count: 0,
            unique_percentage: 0.0,
            numeric_stats: None,
            top_values: None,
            detected_pattern: None,
            min_value: None,
            max_value: None,
            quality_issues: vec![QualityIssue {
                issue_type: "empty_column".to_string(),
                description: "column has no values".to_string(),
                severity: "high".to_string(),
            }],
        };
    }

    // Count nulls and empties
    let null_count = values
        .iter()
        .filter(|v| {
            let t = infer::infer_value_type(v);
            matches!(t, DataType::Null | DataType::Empty)
        })
        .count();

    // Infer types
    let mut type_counts: HashMap<DataType, usize> = HashMap::new();
    for v in &values {
        let t = infer::infer_value_type(v);
        *type_counts.entry(t).or_insert(0) += 1;
    }
    let dominant = infer::dominant_type(&type_counts);

    // Unique count
    let unique_count = values
        .iter()
        .filter(|v| {
            let t = infer::infer_value_type(v);
            !matches!(t, DataType::Null | DataType::Empty)
        })
        .collect::<std::collections::HashSet<&&str>>()
        .len();

    // Top values
    let top_values = compute_top_values(&values, options.top_values_limit);

    // Numeric stats if numeric
    let numeric_stats = if dominant.is_numeric() {
        let nums: Vec<f64> = values
            .iter()
            .filter_map(|v| {
                let t = infer::infer_value_type(v);
                if t.is_numeric() {
                    v.trim().replace(',', "").parse::<f64>().ok()
                } else {
                    None
                }
            })
            .collect();
        stats::compute_numeric_stats(nums)
    } else {
        None
    };

    // Pattern detection
    let detected_pattern = if options.detect_patterns && dominant == DataType::String {
        detect_dominant_pattern(&values)
    } else {
        None
    };

    // Min/max values (for non-numeric, use lexicographic)
    let (min_value, max_value) = compute_min_max(&values, dominant);

    // Quality issues
    let quality_issues = if options.check_quality {
        check_quality(&values, dominant, null_count, count, unique_count)
    } else {
        Vec::new()
    };

    ColumnProfile {
        name: header.to_string(),
        data_type: dominant.as_str().to_string(),
        count,
        null_count,
        null_percentage: percentage(null_count, count),
        unique_count,
        unique_percentage: percentage(unique_count, count),
        numeric_stats,
        top_values: Some(top_values),
        detected_pattern: detected_pattern.map(|p| p.as_str().to_string()),
        min_value,
        max_value,
        quality_issues,
    }
}

/// Compute the top N most frequent values.
fn compute_top_values(values: &[&str], limit: usize) -> Vec<ValueCount> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for v in values {
        let t = infer::infer_value_type(v);
        if matches!(t, DataType::Null | DataType::Empty) {
            continue;
        }
        *counts.entry(v.trim().to_string()).or_insert(0) += 1;
    }

    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let total: usize = values
        .iter()
        .filter(|v| {
            let t = infer::infer_value_type(v);
            !matches!(t, DataType::Null | DataType::Empty)
        })
        .count();

    sorted
        .into_iter()
        .take(limit)
        .map(|(value, count)| ValueCount {
            value,
            count,
            percentage: percentage(count, total),
        })
        .collect()
}

/// Detect the dominant pattern in string values.
fn detect_dominant_pattern(values: &[&str]) -> Option<pattern::Pattern> {
    let mut pattern_counts: HashMap<pattern::Pattern, usize> = HashMap::new();
    let mut total_strings = 0usize;

    for v in values {
        let t = infer::infer_value_type(v);
        if t != DataType::String {
            continue;
        }
        total_strings += 1;
        if let Some(p) = pattern::detect_pattern(v.trim()) {
            *pattern_counts.entry(p).or_insert(0) += 1;
        }
    }

    if total_strings == 0 {
        return None;
    }

    // Need at least 50% of string values to match a pattern
    let threshold = total_strings / 2;
    pattern_counts
        .into_iter()
        .filter(|(_, c)| *c >= threshold.max(1))
        .max_by_key(|(_, c)| *c)
        .map(|(p, _)| p)
}

/// Compute min and max values.
fn compute_min_max(values: &[&str], data_type: DataType) -> (Option<String>, Option<String>) {
    let non_null: Vec<&str> = values
        .iter()
        .filter(|v| {
            let t = infer::infer_value_type(v);
            !matches!(t, DataType::Null | DataType::Empty)
        })
        .copied()
        .collect();

    if non_null.is_empty() {
        return (None, None);
    }

    if data_type.is_numeric() {
        let nums: Vec<f64> = non_null
            .iter()
            .filter_map(|v| v.trim().replace(',', "").parse::<f64>().ok())
            .collect();
        if nums.is_empty() {
            return (None, None);
        }
        let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        (Some(format_number(min)), Some(format_number(max)))
    } else {
        let min = non_null.iter().min().copied();
        let max = non_null.iter().max().copied();
        (min.map(|s| s.to_string()), max.map(|s| s.to_string()))
    }
}

/// Format a number for display, removing unnecessary decimals.
fn format_number(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{n:.0}")
    } else {
        format!("{n:.4}")
    }
}

/// Check data quality issues for a column.
fn check_quality(
    values: &[&str],
    data_type: DataType,
    null_count: usize,
    total: usize,
    unique_count: usize,
) -> Vec<QualityIssue> {
    let mut issues = Vec::new();

    // High null percentage
    let null_pct = percentage(null_count, total);
    if null_pct > 50.0 {
        issues.push(QualityIssue {
            issue_type: "high_null_rate".to_string(),
            description: format!("{null_pct:.1}% of values are null/empty"),
            severity: "high".to_string(),
        });
    } else if null_pct > 20.0 {
        issues.push(QualityIssue {
            issue_type: "moderate_null_rate".to_string(),
            description: format!("{null_pct:.1}% of values are null/empty"),
            severity: "medium".to_string(),
        });
    }

    // Type mismatch: check if dominant type is string but many values were numeric
    if data_type == DataType::String {
        let mut type_counts: HashMap<DataType, usize> = HashMap::new();
        for v in values {
            let t = infer::infer_value_type(v);
            *type_counts.entry(t).or_insert(0) += 1;
        }
        let non_empty: usize = type_counts
            .iter()
            .filter(|(t, _)| !matches!(t, DataType::Null | DataType::Empty))
            .map(|(_, c)| *c)
            .sum();
        let int_count = *type_counts.get(&DataType::Integer).unwrap_or(&0);
        let float_count = *type_counts.get(&DataType::Float).unwrap_or(&0);

        if non_empty > 0 {
            let numeric_pct = percentage(int_count + float_count, non_empty);
            if numeric_pct > 50.0 {
                issues.push(QualityIssue {
                    issue_type: "mixed_types".to_string(),
                    description: format!(
                        "column appears mixed: {numeric_pct:.1}% numeric, rest string"
                    ),
                    severity: "medium".to_string(),
                });
            }
        }
    }

    // Low cardinality warning (potential categorical column with too few values)
    if total > 100 && unique_count == 1 {
        issues.push(QualityIssue {
            issue_type: "constant_column".to_string(),
            description: "all non-null values are identical".to_string(),
            severity: "low".to_string(),
        });
    }

    // High cardinality warning (potential ID column)
    if total > 10 && unique_count == total {
        issues.push(QualityIssue {
            issue_type: "high_cardinality".to_string(),
            description: "all values are unique (possible ID column)".to_string(),
            severity: "low".to_string(),
        });
    }

    // Outlier detection for numeric columns
    if data_type.is_numeric() {
        let nums: Vec<f64> = values
            .iter()
            .filter_map(|v| {
                let t = infer::infer_value_type(v);
                if t.is_numeric() {
                    v.trim().replace(',', "").parse::<f64>().ok()
                } else {
                    None
                }
            })
            .collect();
        let outliers = stats::detect_outliers(&nums);
        if !outliers.is_empty() && nums.len() > 3 {
            let pct = percentage(outliers.len(), nums.len());
            if pct > 5.0 {
                issues.push(QualityIssue {
                    issue_type: "outliers".to_string(),
                    description: format!(
                        "{} outlier(s) detected ({pct:.1}% of values)",
                        outliers.len()
                    ),
                    severity: "medium".to_string(),
                });
            }
        }
    }

    issues
}

/// Compute overall quality score (0-100, higher is better).
fn compute_quality_score(columns: &[ColumnProfile]) -> f64 {
    if columns.is_empty() {
        return 0.0;
    }

    let mut total_penalty = 0.0;
    for col in columns {
        for issue in &col.quality_issues {
            let penalty = match issue.severity.as_str() {
                "high" => 15.0,
                "medium" => 7.0,
                "low" => 2.0,
                _ => 0.0,
            };
            total_penalty += penalty;
        }
    }

    let max_penalty = columns.len() as f64 * 15.0;
    if max_penalty == 0.0 {
        return 100.0;
    }

    let score = 100.0 - (total_penalty / max_penalty * 100.0);
    score.clamp(0.0, 100.0)
}

/// Convert quality score to letter grade.
fn quality_grade(score: f64) -> String {
    match score {
        s if s >= 90.0 => "A".to_string(),
        s if s >= 80.0 => "B".to_string(),
        s if s >= 70.0 => "C".to_string(),
        s if s >= 60.0 => "D".to_string(),
        _ => "F".to_string(),
    }
}

/// Compute percentage safely.
fn percentage(part: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (part as f64 / total as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{DataSource, Row};

    fn make_data(headers: Vec<String>, rows: Vec<Row>) -> DataSource {
        DataSource { headers, rows }
    }

    #[test]
    fn test_profile_basic() {
        let data = make_data(
            vec!["name".to_string(), "age".to_string()],
            vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
                vec!["Charlie".to_string(), "35".to_string()],
            ],
        );
        let profile = profile_dataset(&data, "csv", &ProfileOptions::default()).unwrap();
        assert_eq!(profile.row_count, 3);
        assert_eq!(profile.column_count, 2);
        assert_eq!(profile.columns[0].data_type, "string");
        assert_eq!(profile.columns[1].data_type, "integer");
    }

    #[test]
    fn test_profile_with_nulls() {
        let data = make_data(
            vec!["val".to_string()],
            vec![
                vec!["1".to_string()],
                vec!["".to_string()],
                vec!["null".to_string()],
                vec!["3".to_string()],
            ],
        );
        let profile = profile_dataset(&data, "csv", &ProfileOptions::default()).unwrap();
        assert_eq!(profile.columns[0].null_count, 2);
        assert_eq!(profile.columns[0].count, 4);
    }

    #[test]
    fn test_quality_score() {
        let cols = vec![ColumnProfile {
            name: "test".to_string(),
            data_type: "integer".to_string(),
            count: 100,
            null_count: 0,
            null_percentage: 0.0,
            unique_count: 100,
            unique_percentage: 100.0,
            numeric_stats: None,
            top_values: None,
            detected_pattern: None,
            min_value: None,
            max_value: None,
            quality_issues: vec![],
        }];
        let score = compute_quality_score(&cols);
        assert_eq!(score, 100.0);
    }
}
