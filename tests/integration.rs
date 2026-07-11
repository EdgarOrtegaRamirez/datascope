use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn bin() -> Command {
    Command::cargo_bin("datascope").expect("failed to find datascope binary")
}

#[test]
fn test_csv_text_output() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--output").arg("text");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DataScope Profile Report"))
        .stdout(predicate::str::contains("Rows:        10"))
        .stdout(predicate::str::contains("Columns:     6"));
}

#[test]
fn test_csv_json_output() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--output").arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["row_count"], 10);
    assert_eq!(parsed["column_count"], 6);
    assert_eq!(parsed["format"], "csv");
}

#[test]
fn test_csv_csv_output() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--output").arg("csv");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("column,data_type,count"))
        .stdout(predicate::str::contains("name,string"))
        .stdout(predicate::str::contains("age,integer"));
}

#[test]
fn test_jsonl_input() {
    let jsonl_path = fixture_path("sample.jsonl");
    let mut cmd = bin();
    cmd.arg(&jsonl_path).arg("--output").arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["row_count"], 5);
    assert_eq!(parsed["format"], "jsonl");
}

#[test]
fn test_json_input() {
    let json_path = fixture_path("sample.json");
    let mut cmd = bin();
    cmd.arg(&json_path).arg("--output").arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["row_count"], 3);
    assert_eq!(parsed["format"], "json");
}

#[test]
fn test_tsv_input() {
    let tsv_path = fixture_path("sample.tsv");
    let mut cmd = bin();
    cmd.arg(&tsv_path).arg("--output").arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["row_count"], 3);
    assert_eq!(parsed["format"], "tsv");
}

#[test]
fn test_stdin_csv() {
    let csv_data = "name,age\nAlice,30\nBob,25\n";
    let mut cmd = bin();
    cmd.arg("-")
        .arg("--format")
        .arg("csv")
        .arg("--output")
        .arg("json");
    cmd.write_stdin(csv_data);
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["row_count"], 2);
}

#[test]
fn test_summary_only() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--summary-only");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Format:"))
        .stdout(predicate::str::contains("Rows:"))
        .stdout(predicate::str::contains("Column"));
}

#[test]
fn test_pattern_detection() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--output").arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    // The email column should have pattern "email"
    let email_col = parsed["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "email")
        .unwrap();
    assert_eq!(email_col["detected_pattern"], "email");
}

#[test]
fn test_quality_issues_with_nulls() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--output").arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    // The age column has a null value
    let age_col = parsed["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "age")
        .unwrap();
    assert!(age_col["null_count"].as_u64().unwrap() > 0);
}

#[test]
fn test_strict_mode_fails_on_issues() {
    // Create a CSV with high null rate (>50%) to trigger a high-severity issue
    let csv_data = "val\n1\n\n\n\n\n\n\n\n\n\n";
    let mut cmd = bin();
    cmd.arg("-").arg("--format").arg("csv").arg("--strict");
    cmd.write_stdin(csv_data);
    // The CSV reader may skip empty lines, so this might not trigger.
    // Use a two-column CSV where one column is mostly empty.
    let csv_data2 = "id,val\n1,10\n2,\n3,\n4,\n5,\n6,\n7,\n8,\n9,\n10,\n";
    let mut cmd2 = bin();
    cmd2.arg("-").arg("--format").arg("csv").arg("--strict");
    cmd2.write_stdin(csv_data2);
    cmd2.assert().failure();
}

#[test]
fn test_no_quality_flag() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path)
        .arg("--output")
        .arg("json")
        .arg("--no-quality");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    // With --no-quality, columns should have empty quality_issues
    for col in parsed["columns"].as_array().unwrap() {
        assert!(col["quality_issues"].as_array().unwrap().is_empty());
    }
}

#[test]
fn test_no_patterns_flag() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path)
        .arg("--output")
        .arg("json")
        .arg("--no-patterns");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let email_col = parsed["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "email")
        .unwrap();
    assert!(email_col["detected_pattern"].is_null());
}

#[test]
fn test_nonexistent_file() {
    let mut cmd = bin();
    cmd.arg("/nonexistent/file.csv");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_format_auto_detect() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--output").arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["format"], "csv");
}

#[test]
fn test_explicit_format_override() {
    // Use a .csv file but force JSON format
    let json_path = fixture_path("sample.json");
    let mut cmd = bin();
    cmd.arg(&json_path)
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["format"], "json");
}

#[test]
fn test_numeric_stats() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--output").arg("json");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let age_col = parsed["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "age")
        .unwrap();
    // Age column should have numeric stats
    assert!(age_col["numeric_stats"].is_object());
    assert!(age_col["numeric_stats"]["mean"].is_number());
    assert!(age_col["numeric_stats"]["median"].is_number());
    assert!(age_col["numeric_stats"]["std_dev"].is_number());
}

#[test]
fn test_top_values() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path)
        .arg("--output")
        .arg("json")
        .arg("--top")
        .arg("3");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let name_col = parsed["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "name")
        .unwrap();
    // With --top 3, should have at most 3 top values
    let top_values = name_col["top_values"].as_array().unwrap();
    assert!(top_values.len() <= 3);
}

#[test]
fn test_empty_file_error() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    let mut cmd = bin();
    cmd.arg(temp.path()).arg("--format").arg("csv");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_schema_generation() {
    let csv_path = fixture_path("sample.csv");
    let mut cmd = bin();
    cmd.arg(&csv_path).arg("--schema");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert_eq!(parsed["type"], "object");
    assert_eq!(parsed["properties"]["name"]["type"], "string");
    assert_eq!(parsed["properties"]["age"]["type"], "integer");
    assert!(parsed["required"].as_array().unwrap().contains(&serde_json::Value::String("name".to_string())));
}

#[test]
fn test_schema_jsonl_input() {
    let jsonl_path = fixture_path("sample.jsonl");
    let mut cmd = bin();
    cmd.arg(&jsonl_path).arg("--schema");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert!(parsed["properties"].is_object());
}

#[test]
fn test_schema_stdin() {
    let csv_data = "name,age\nAlice,30\nBob,25\n";
    let mut cmd = bin();
    cmd.arg("-")
        .arg("--format")
        .arg("csv")
        .arg("--schema");
    cmd.write_stdin(csv_data);
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["properties"]["name"]["type"], "string");
    assert_eq!(parsed["properties"]["age"]["type"], "integer");
}

#[test]
fn test_numeric_stats_with_outliers() {
    let csv_data = "val\n10\n12\n11\n13\n9\n100\n";
    let mut cmd = bin();
    cmd.arg("-")
        .arg("--format")
        .arg("csv")
        .arg("--output")
        .arg("json");
    cmd.write_stdin(csv_data);
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let val_col = parsed["columns"][0].as_object().unwrap();
    let ns = &val_col["numeric_stats"];
    assert_eq!(ns["outlier_count"], 1);
    assert!(ns["outlier_percentage"].as_f64().unwrap() > 15.0);
}

#[test]
fn test_stdin_requires_format() {
    let mut cmd = bin();
    cmd.arg("-").arg("--output").arg("json");
    cmd.write_stdin("name,age\nAlice,30\n");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("format is required"));
}
