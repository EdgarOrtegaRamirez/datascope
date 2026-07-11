use crate::profile::DatasetProfile;

/// Render profile as formatted text table.
pub fn render_text(profile: &DatasetProfile) -> String {
    let mut out = String::new();

    // Header
    out.push_str("╭─────────────────────────────────────────────╮\n");
    out.push_str("│            DataScope Profile Report          │\n");
    out.push_str("╰─────────────────────────────────────────────╯\n\n");

    // Dataset summary
    out.push_str("Dataset Summary\n");
    out.push_str(&format!("  Format:      {}\n", profile.format));
    out.push_str(&format!("  Rows:        {}\n", profile.row_count));
    out.push_str(&format!("  Columns:     {}\n", profile.column_count));
    out.push_str(&format!(
        "  Quality:     {}/100 ({})\n\n",
        profile.quality_score.round() as i64,
        profile.quality_grade
    ));

    // Column profiles
    for col in &profile.columns {
        out.push_str(&format!("── {} ", col.name));
        let dash_count = if col.name.len() < 60 {
            64 - col.name.len()
        } else {
            4
        };
        for _ in 0..dash_count {
            out.push('─');
        }
        out.push('\n');

        out.push_str(&format!("  Type:          {}\n", col.data_type));
        out.push_str(&format!("  Count:         {}\n", col.count));
        out.push_str(&format!(
            "  Null/Empty:    {} ({:.1}%)\n",
            col.null_count, col.null_percentage
        ));
        out.push_str(&format!(
            "  Unique:        {} ({:.1}%)\n",
            col.unique_count, col.unique_percentage
        ));

        if let Some(ref stats) = col.numeric_stats {
            out.push_str("  Statistics:\n");
            out.push_str(&format!("    Min:    {}\n", format_num(stats.min)));
            out.push_str(&format!("    Max:    {}\n", format_num(stats.max)));
            out.push_str(&format!("    Mean:   {}\n", format_num(stats.mean)));
            out.push_str(&format!("    Median: {}\n", format_num(stats.median)));
            out.push_str(&format!("    StdDev: {}\n", format_num(stats.std_dev)));
            out.push_str(&format!("    Q1:     {}\n", format_num(stats.q1)));
            out.push_str(&format!("    Q3:     {}\n", format_num(stats.q3)));
            out.push_str(&format!("    Sum:    {}\n", format_num(stats.sum)));
            if let Some(outlier_count) = stats.outlier_count {
                if outlier_count > 0 {
                    out.push_str(&format!(
                        "    Outliers: {} ({:.1}%)\n",
                        outlier_count,
                        stats.outlier_percentage.unwrap_or(0.0)
                    ));
                } else {
                    out.push_str("    Outliers: None\n");
                }
            }
        }

        if let Some(ref pattern) = col.detected_pattern {
            out.push_str(&format!("  Pattern:       {pattern}\n"));
        }

        if let Some(ref min) = col.min_value {
            out.push_str(&format!("  Min Value:     {min}\n"));
        }
        if let Some(ref max) = col.max_value {
            out.push_str(&format!("  Max Value:     {max}\n"));
        }

        if let Some(ref top) = col.top_values {
            if !top.is_empty() {
                out.push_str("  Top Values:\n");
                for tv in top.iter().take(5) {
                    let display_val = if tv.value.len() > 30 {
                        format!("{}...", &tv.value[..27])
                    } else {
                        tv.value.clone()
                    };
                    out.push_str(&format!(
                        "    {:<32} {} ({:.1}%)\n",
                        display_val, tv.count, tv.percentage
                    ));
                }
            }
        }

        if !col.quality_issues.is_empty() {
            out.push_str("  Quality Issues:\n");
            for issue in &col.quality_issues {
                let icon = match issue.severity.as_str() {
                    "high" => "✗",
                    "medium" => "⚠",
                    "low" => "ℹ",
                    _ => "•",
                };
                out.push_str(&format!(
                    "    {icon} [{}] {}\n",
                    issue.severity, issue.description
                ));
            }
        }

        out.push('\n');
    }

    out
}

fn format_num(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{n:.0}")
    } else {
        format!("{n:.4}")
    }
}
