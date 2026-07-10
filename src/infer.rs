use std::collections::HashMap;

/// Inferred data type for a column or value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Integer,
    Float,
    Boolean,
    Date,
    DateTime,
    String,
    Empty,
    Null,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Integer => write!(f, "integer"),
            DataType::Float => write!(f, "float"),
            DataType::Boolean => write!(f, "boolean"),
            DataType::Date => write!(f, "date"),
            DataType::DateTime => write!(f, "datetime"),
            DataType::String => write!(f, "string"),
            DataType::Empty => write!(f, "empty"),
            DataType::Null => write!(f, "null"),
        }
    }
}

impl DataType {
    /// Serialize for JSON output.
    pub fn as_str(&self) -> &'static str {
        match self {
            DataType::Integer => "integer",
            DataType::Float => "float",
            DataType::Boolean => "boolean",
            DataType::Date => "date",
            DataType::DateTime => "datetime",
            DataType::String => "string",
            DataType::Empty => "empty",
            DataType::Null => "null",
        }
    }

    /// Returns true if this type is numeric (integer or float).
    pub fn is_numeric(&self) -> bool {
        matches!(self, DataType::Integer | DataType::Float)
    }
}

/// Infer the data type of a single string value.
pub fn infer_value_type(value: &str) -> DataType {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return DataType::Empty;
    }

    // Null-like values
    let lower = trimmed.to_lowercase();
    if matches!(
        lower.as_str(),
        "null" | "none" | "nil" | "na" | "n/a" | "nan"
    ) {
        return DataType::Null;
    }

    // Boolean
    if matches!(lower.as_str(), "true" | "false" | "yes" | "no" | "0" | "1") {
        // Only treat as boolean if exactly "true"/"false"/"yes"/"no"
        if matches!(lower.as_str(), "true" | "false" | "yes" | "no") {
            return DataType::Boolean;
        }
    }

    // Integer (including negative, with optional thousands separators)
    if is_integer(trimmed) {
        return DataType::Integer;
    }

    // Float
    if is_float(trimmed) {
        return DataType::Float;
    }

    // Date / DateTime
    if let Some(dt) = detect_date_time(trimmed) {
        return dt;
    }

    DataType::String
}

/// Check if a string represents an integer.
fn is_integer(s: &str) -> bool {
    // Allow leading + or -
    let s = s.strip_prefix('+').unwrap_or(s);
    let s = s.strip_prefix('-').unwrap_or(s);

    if s.is_empty() {
        return false;
    }

    // Allow digits with optional thousand separators (commas)
    let without_seps = s.replace(',', "");
    if without_seps.is_empty() {
        return false;
    }
    without_seps.chars().all(|c| c.is_ascii_digit())
}

/// Check if a string represents a floating-point number.
fn is_float(s: &str) -> bool {
    let s = s.trim();

    // Must contain a decimal point or exponent to be a float
    if !s.contains('.') && !s.contains('e') && !s.contains('E') {
        return false;
    }

    // Try parsing as f64
    s.parse::<f64>().is_ok()
}

/// Detect date or datetime formats. Returns Some(Date) or Some(DateTime) if matched.
fn detect_date_time(s: &str) -> Option<DataType> {
    // ISO 8601 date: YYYY-MM-DD
    if is_iso_date(s) {
        // Check if there's a time component
        if s.len() > 10 {
            return Some(DataType::DateTime);
        }
        return Some(DataType::Date);
    }

    // Common date formats: MM/DD/YYYY, DD/MM/YYYY, MM-DD-YYYY, DD-MM-YYYY
    if is_slash_date(s) {
        return Some(DataType::Date);
    }

    // US format: Mon DD, YYYY or Month DD, YYYY
    if is_named_month_date(s) {
        return Some(DataType::Date);
    }

    None
}

fn is_iso_date(s: &str) -> bool {
    // YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS...
    let bytes = s.as_bytes();
    if bytes.len() < 10 {
        return false;
    }
    // Check YYYY-MM-DD pattern
    if bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
    {
        return true;
    }
    false
}

fn is_slash_date(s: &str) -> bool {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return false;
    }
    // Each part should be numeric (2-4 digits)
    parts
        .iter()
        .all(|p| !p.is_empty() && p.len() <= 4 && p.chars().all(|c| c.is_ascii_digit()))
}

fn is_named_month_date(s: &str) -> bool {
    let months = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    months.iter().any(|m| s.starts_with(m))
}

/// Determine the dominant data type from a collection of inferred types.
/// Uses a priority system: if all non-empty/non-null values are the same type, that's the type.
/// If mixed integer and float, it's float. Otherwise, string.
pub fn dominant_type(type_counts: &HashMap<DataType, usize>) -> DataType {
    let total: usize = type_counts.values().sum();
    if total == 0 {
        return DataType::Null;
    }

    let non_empty: usize = type_counts
        .iter()
        .filter(|(t, _)| !matches!(t, DataType::Empty | DataType::Null))
        .map(|(_, c)| *c)
        .sum();

    if non_empty == 0 {
        // All values are empty or null
        if *type_counts.get(&DataType::Empty).unwrap_or(&0) > 0 {
            return DataType::Empty;
        }
        return DataType::Null;
    }

    // Check for single dominant type (excluding empty/null)
    let mut non_empty_types: Vec<(DataType, usize)> = type_counts
        .iter()
        .filter(|(t, _)| !matches!(t, DataType::Empty | DataType::Null))
        .map(|(t, c)| (*t, *c))
        .collect();

    non_empty_types.sort_by_key(|b| std::cmp::Reverse(b.1));

    // If all non-empty values are the same type
    if non_empty_types.len() == 1 {
        return non_empty_types[0].0;
    }

    // If integer + float, return float
    let has_int = non_empty_types.iter().any(|(t, _)| *t == DataType::Integer);
    let has_float = non_empty_types.iter().any(|(t, _)| *t == DataType::Float);
    let only_numeric = non_empty_types.iter().all(|(t, _)| t.is_numeric());
    if has_int && has_float && only_numeric {
        return DataType::Float;
    }

    // Mixed types → string
    DataType::String
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_integer() {
        assert_eq!(infer_value_type("42"), DataType::Integer);
        assert_eq!(infer_value_type("-17"), DataType::Integer);
        assert_eq!(infer_value_type("+100"), DataType::Integer);
        assert_eq!(infer_value_type("1,000"), DataType::Integer);
    }

    #[test]
    fn test_infer_float() {
        assert_eq!(infer_value_type("3.14"), DataType::Float);
        assert_eq!(infer_value_type("-0.5"), DataType::Float);
        assert_eq!(infer_value_type("1e10"), DataType::Float);
        assert_eq!(infer_value_type("1.5E-3"), DataType::Float);
    }

    #[test]
    fn test_infer_boolean() {
        assert_eq!(infer_value_type("true"), DataType::Boolean);
        assert_eq!(infer_value_type("false"), DataType::Boolean);
        assert_eq!(infer_value_type("yes"), DataType::Boolean);
        assert_eq!(infer_value_type("no"), DataType::Boolean);
    }

    #[test]
    fn test_infer_null_empty() {
        assert_eq!(infer_value_type(""), DataType::Empty);
        assert_eq!(infer_value_type("   "), DataType::Empty);
        assert_eq!(infer_value_type("null"), DataType::Null);
        assert_eq!(infer_value_type("NA"), DataType::Null);
        assert_eq!(infer_value_type("n/a"), DataType::Null);
    }

    #[test]
    fn test_infer_date() {
        assert_eq!(infer_value_type("2024-01-15"), DataType::Date);
        assert_eq!(infer_value_type("2024-01-15T10:30:00"), DataType::DateTime);
        assert_eq!(infer_value_type("01/15/2024"), DataType::Date);
    }

    #[test]
    fn test_infer_string() {
        assert_eq!(infer_value_type("hello world"), DataType::String);
        assert_eq!(infer_value_type("test@example.com"), DataType::String);
    }

    #[test]
    fn test_dominant_type() {
        let mut counts = HashMap::new();
        counts.insert(DataType::Integer, 100);
        counts.insert(DataType::Empty, 5);
        assert_eq!(dominant_type(&counts), DataType::Integer);

        counts.clear();
        counts.insert(DataType::Integer, 50);
        counts.insert(DataType::Float, 50);
        assert_eq!(dominant_type(&counts), DataType::Float);

        counts.clear();
        counts.insert(DataType::Integer, 50);
        counts.insert(DataType::String, 50);
        assert_eq!(dominant_type(&counts), DataType::String);
    }
}
