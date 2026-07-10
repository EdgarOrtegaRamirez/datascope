use crate::profile::DatasetProfile;

/// Render profile as CSV summary (one row per column).
pub fn render_csv(profile: &DatasetProfile) -> String {
    let mut out = String::new();

    // Header row
    out.push_str("column,data_type,count,null_count,null_pct,unique_count,unique_pct");
    out.push_str(",min,max,mean,median,std_dev,quality_issues\n");

    for col in &profile.columns {
        let mut row = vec![
            csv_escape(&col.name),
            col.data_type.clone(),
            col.count.to_string(),
            col.null_count.to_string(),
            format!("{:.1}", col.null_percentage),
            col.unique_count.to_string(),
            format!("{:.1}", col.unique_percentage),
        ];

        // Min/max
        row.push(csv_escape(col.min_value.as_deref().unwrap_or("")));
        row.push(csv_escape(col.max_value.as_deref().unwrap_or("")));

        // Numeric stats
        if let Some(ref stats) = col.numeric_stats {
            row.push(format_num(stats.mean));
            row.push(format_num(stats.median));
            row.push(format_num(stats.std_dev));
        } else {
            row.push(String::new());
            row.push(String::new());
            row.push(String::new());
        }

        // Quality issues count
        let issue_summary = if col.quality_issues.is_empty() {
            String::new()
        } else {
            col.quality_issues
                .iter()
                .map(|i| i.issue_type.clone())
                .collect::<Vec<_>>()
                .join(";")
        };
        row.push(csv_escape(&issue_summary));

        out.push_str(&row.join(","));
        out.push('\n');
    }

    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn format_num(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{n:.0}")
    } else {
        format!("{n:.4}")
    }
}
