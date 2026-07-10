use crate::error::{DatascopeError, Result};
use crate::format::{DataSource, Row};
use serde_json::Value;
use std::io::Read;

/// Read a JSON array of objects.
pub fn read_json(reader: &mut dyn Read) -> Result<DataSource> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let value: Value = serde_json::from_str(&buf)?;

    let array = match &value {
        Value::Array(arr) => arr,
        _ => {
            return Err(DatascopeError::InvalidInput(
                "JSON input must be an array of objects".to_string(),
            ));
        }
    };

    if array.is_empty() {
        return Err(DatascopeError::NoData);
    }

    // Collect all unique keys across all objects as headers
    let mut headers: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for obj in array {
        if let Value::Object(map) = obj {
            for key in map.keys() {
                if seen.insert(key.clone()) {
                    headers.push(key.clone());
                }
            }
        }
    }

    if headers.is_empty() {
        return Err(DatascopeError::InvalidInput(
            "JSON objects have no keys".to_string(),
        ));
    }

    let mut rows: Vec<Row> = Vec::with_capacity(array.len());
    for obj in array {
        let row = json_object_to_row(obj, &headers);
        rows.push(row);
    }

    Ok(DataSource { headers, rows })
}

/// Read JSONL (JSON Lines) — one JSON object per line.
pub fn read_jsonl(reader: &mut dyn Read) -> Result<DataSource> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let lines: Vec<&str> = buf.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.is_empty() {
        return Err(DatascopeError::NoData);
    }

    // Parse all objects and collect headers
    let mut objects: Vec<Value> = Vec::with_capacity(lines.len());
    let mut headers: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for line in &lines {
        let obj: Value = serde_json::from_str(line)
            .map_err(|e| DatascopeError::Json(format!("failed to parse JSONL line: {e}")))?;
        if let Value::Object(map) = &obj {
            for key in map.keys() {
                if seen.insert(key.clone()) {
                    headers.push(key.clone());
                }
            }
        }
        objects.push(obj);
    }

    if headers.is_empty() {
        return Err(DatascopeError::InvalidInput(
            "JSONL objects have no keys".to_string(),
        ));
    }

    let mut rows: Vec<Row> = Vec::with_capacity(objects.len());
    for obj in objects {
        let row = json_object_to_row(&obj, &headers);
        rows.push(row);
    }

    Ok(DataSource { headers, rows })
}

/// Convert a JSON value (object) to a row of strings.
fn json_object_to_row(obj: &Value, headers: &[String]) -> Row {
    let map = match obj {
        Value::Object(m) => m,
        _ => return vec![String::new(); headers.len()],
    };

    headers
        .iter()
        .map(|h| match map.get(h) {
            Some(Value::Null) | None => String::new(),
            Some(Value::String(s)) => s.clone(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Array(_)) | Some(Value::Object(_)) => {
                serde_json::to_string(map.get(h).unwrap()).unwrap_or_default()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_json() {
        let json = r#"[{"name":"Alice","age":30},{"name":"Bob","age":25}]"#;
        let mut reader = std::io::Cursor::new(json);
        let ds = read_json(&mut reader).unwrap();
        // Keys may be in any order (serde_json uses BTreeMap)
        assert!(ds.headers.contains(&"name".to_string()));
        assert!(ds.headers.contains(&"age".to_string()));
        assert_eq!(ds.rows.len(), 2);
        let name_idx = ds.headers.iter().position(|h| h == "name").unwrap();
        let age_idx = ds.headers.iter().position(|h| h == "age").unwrap();
        assert_eq!(ds.rows[0][name_idx], "Alice");
        assert_eq!(ds.rows[0][age_idx], "30");
    }

    #[test]
    fn test_read_jsonl() {
        let jsonl = "{\"name\":\"Alice\",\"age\":30}\n{\"name\":\"Bob\",\"age\":25}\n";
        let mut reader = std::io::Cursor::new(jsonl);
        let ds = read_jsonl(&mut reader).unwrap();
        assert!(ds.headers.contains(&"name".to_string()));
        assert!(ds.headers.contains(&"age".to_string()));
        assert_eq!(ds.rows.len(), 2);
        let name_idx = ds.headers.iter().position(|h| h == "name").unwrap();
        assert_eq!(ds.rows[0][name_idx], "Alice");
    }

    #[test]
    fn test_json_with_missing_keys() {
        let json = r#"[{"a":1,"b":2},{"a":3}]"#;
        let mut reader = std::io::Cursor::new(json);
        let ds = read_json(&mut reader).unwrap();
        assert_eq!(ds.rows[1], vec!["3", ""]);
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{"not": "an array"}"#;
        let mut reader = std::io::Cursor::new(json);
        assert!(read_json(&mut reader).is_err());
    }
}
