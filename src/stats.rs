/// Statistical calculations for numeric columns.
use std::cmp::Ordering;

/// Summary statistics for a numeric column.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NumericStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub q1: f64,
    pub q3: f64,
    pub sum: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outlier_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outlier_percentage: Option<f64>,
}

/// Compute numeric statistics from a list of f64 values.
/// Returns None if the list is empty.
pub fn compute_numeric_stats(mut values: Vec<f64>) -> Option<NumericStats> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let count = values.len();
    let min = values[0];
    let max = values[count - 1];
    let sum: f64 = values.iter().sum();
    let mean = sum / count as f64;

    let median = percentile_sorted(&values, 50.0);
    let q1 = percentile_sorted(&values, 25.0);
    let q3 = percentile_sorted(&values, 75.0);

    let std_dev = if count > 1 {
        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
        variance.sqrt()
    } else {
        0.0
    };

    // Compute outlier info
    let iqr = q3 - q1;
    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;
    let outlier_indices: Vec<usize> = values
        .iter()
        .enumerate()
        .filter(|(_, v)| **v < lower_bound || **v > upper_bound)
        .map(|(i, _)| i)
        .collect();
    let outlier_count = outlier_indices.len();

    Some(NumericStats {
        count,
        min,
        max,
        mean,
        median,
        std_dev,
        q1,
        q3,
        sum,
        outlier_count: Some(outlier_count),
        outlier_percentage: Some(if count > 0 {
            (outlier_count as f64 / count as f64) * 100.0
        } else {
            0.0
        }),
    })
}

/// Compute a percentile from a sorted slice using linear interpolation.
fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }

    let rank = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;

    if lower == upper {
        return sorted[lower];
    }

    let frac = rank - lower as f64;
    sorted[lower] * (1.0 - frac) + sorted[upper] * frac
}

/// Detect outliers using the IQR method.
/// Returns indices of values that are below Q1 - 1.5*IQR or above Q3 + 1.5*IQR.
pub fn detect_outliers(values: &[f64]) -> Vec<usize> {
    if values.len() < 4 {
        return Vec::new();
    }

    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let q1 = percentile_sorted(&sorted, 25.0);
    let q3 = percentile_sorted(&sorted, 75.0);
    let iqr = q3 - q1;
    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;

    values
        .iter()
        .enumerate()
        .filter(|(_, v)| **v < lower_bound || **v > upper_bound)
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = compute_numeric_stats(values).unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.sum, 15.0);
    }

    #[test]
    fn test_empty() {
        assert!(compute_numeric_stats(vec![]).is_none());
    }

    #[test]
    fn test_single_value() {
        let stats = compute_numeric_stats(vec![42.0]).unwrap();
        assert_eq!(stats.std_dev, 0.0);
        assert_eq!(stats.min, 42.0);
        assert_eq!(stats.max, 42.0);
    }

    #[test]
    fn test_outliers() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let outliers = detect_outliers(&values);
        assert!(outliers.contains(&5)); // 100.0 is an outlier
    }

    #[test]
    fn test_no_outliers() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let outliers = detect_outliers(&values);
        assert!(outliers.is_empty());
    }

    #[test]
    fn test_quartiles() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let stats = compute_numeric_stats(values).unwrap();
        // Q1 should be around 2.75, Q3 around 6.25
        assert!(stats.q1 > 2.0 && stats.q1 < 3.0);
        assert!(stats.q3 > 6.0 && stats.q3 < 7.0);
    }
}
