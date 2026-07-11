use crate::format::DataSource;
use crate::infer::{self, DataType};
use serde::Serialize;
use std::collections::BTreeMap;

/// A JSON Schema property definition.
#[derive(Debug, Clone, Serialize)]
pub struct SchemaProperty {
    #[serde(rename = "type")]
    prop_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Box<SchemaProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<BTreeMap<String, SchemaProperty>>,
}

/// A complete JSON Schema document (Draft 2020-12).
#[derive(Debug, Clone, Serialize)]
pub struct JsonSchema {
    #[serde(rename = "$schema")]
    schema_url: String,
    #[serde(rename = "type")]
    root_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    properties: BTreeMap<String, SchemaProperty>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    required: Vec<String>,
}

/// Generate a JSON Schema (Draft 2020-12) from a dataset.
///
/// Reads the column headers and infers types from the data values,
/// producing a JSON Schema with `type: "object"`, a `properties`
/// map with per-column schemas, and a `required` array for
/// columns that have no null/empty values.
pub fn generate_schema(data: &DataSource) -> JsonSchema {
    let mut properties: BTreeMap<String, SchemaProperty> = BTreeMap::new();
    let mut required: Vec<String> = Vec::new();

    for (col_idx, header) in data.headers.iter().enumerate() {
        // Collect all values for this column
        let values: Vec<&str> = data
            .rows
            .iter()
            .filter_map(|row| row.get(col_idx).map(|s| s.as_str()))
            .collect();

        let (prop, col_has_nulls) = infer_column_schema(&values, header);
        properties.insert(header.clone(), prop);

        if !col_has_nulls {
            required.push(header.clone());
        }
    }

    // Sort required fields alphabetically for deterministic output
    required.sort();

    JsonSchema {
        schema_url: "https://json-schema.org/draft/2020-12/schema".to_string(),
        root_type: "object".to_string(),
        title: Some("Data Profile Schema".to_string()),
        description: Some(format!(
            "Auto-generated JSON Schema for dataset with {} column(s) and {} row(s)",
            data.headers.len(),
            data.rows.len()
        )),
        properties,
        required,
    }
}

/// Infer a JSON Schema property for a single column from its values.
fn infer_column_schema(values: &[&str], column_name: &str) -> (SchemaProperty, bool) {
    if values.is_empty() {
        return (
            SchemaProperty {
                prop_type: "null".to_string(),
                description: Some(format!("Column '{}' has no values", column_name)),
                items: None,
                properties: None,
            },
            true,
        );
    }

    // Count types
    let mut type_counts: std::collections::HashMap<DataType, usize> =
        std::collections::HashMap::new();
    let mut has_nulls = false;

    for v in values {
        let t = infer::infer_value_type(v);
        if matches!(t, DataType::Null | DataType::Empty) {
            has_nulls = true;
        } else {
            *type_counts.entry(t).or_insert(0) += 1;
        }
    }

    let dominant = infer::dominant_type(&type_counts);

    // Determine JSON Schema type from inferred type
    let json_type = match dominant {
        DataType::Integer => "integer".to_string(),
        DataType::Float => "number".to_string(),
        DataType::Boolean => "boolean".to_string(),
        DataType::Date | DataType::DateTime => "string".to_string(),
        DataType::String => "string".to_string(),
        DataType::Empty | DataType::Null => "null".to_string(),
    };

    let description = if has_nulls {
        Some(format!(
            "Column '{}' (inferred type: {}, with null values)",
            column_name, dominant
        ))
    } else {
        Some(format!(
            "Column '{}' (inferred type: {})",
            column_name, dominant
        ))
    };

    (
        SchemaProperty {
            prop_type: json_type,
            description,
            items: None,
            properties: None,
        },
        has_nulls,
    )
}

impl std::fmt::Display for JsonSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(json) => write!(f, "{}", json),
            Err(e) => write!(f, "{{ \"error\": \"failed to serialize schema: {}\" }}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{DataSource, Row};

    fn make_data(headers: Vec<String>, rows: Vec<Row>) -> DataSource {
        DataSource { headers, rows }
    }

    #[test]
    fn test_generate_schema_basic() {
        let data = make_data(
            vec!["name".to_string(), "age".to_string(), "active".to_string()],
            vec![
                vec![
                    "Alice".to_string(),
                    "30".to_string(),
                    "true".to_string(),
                ],
                vec![
                    "Bob".to_string(),
                    "25".to_string(),
                    "false".to_string(),
                ],
                vec![
                    "Charlie".to_string(),
                    "35".to_string(),
                    "true".to_string(),
                ],
            ],
        );

        let schema = generate_schema(&data);

        // Check schema version
        assert_eq!(
            schema.schema_url,
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema.root_type, "object");

        // Check properties exist
        assert!(schema.properties.contains_key("name"));
        assert!(schema.properties.contains_key("age"));
        assert!(schema.properties.contains_key("active"));

        // Check types
        assert_eq!(schema.properties["name"].prop_type, "string");
        assert_eq!(schema.properties["age"].prop_type, "integer");
        assert_eq!(schema.properties["active"].prop_type, "boolean");

        // All columns are non-null, so all should be required
        assert_eq!(schema.required.len(), 3);
        assert!(schema.required.contains(&"name".to_string()));
        assert!(schema.required.contains(&"age".to_string()));
        assert!(schema.required.contains(&"active".to_string()));
    }

    #[test]
    fn test_generate_schema_with_nulls() {
        let data = make_data(
            vec!["name".to_string(), "age".to_string()],
            vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "null".to_string()],
                vec!["Charlie".to_string(), "35".to_string()],
            ],
        );

        let schema = generate_schema(&data);

        // Age has null values, so it should not be required
        assert_eq!(schema.required.len(), 1);
        assert!(schema.required.contains(&"name".to_string()));

        // Age should still be inferred as integer (majority)
        assert_eq!(schema.properties["age"].prop_type, "integer");

        // Description should mention nulls
        let age_desc = schema.properties["age"]
            .description
            .as_deref()
            .unwrap_or("");
        assert!(age_desc.contains("null"));
    }

    #[test]
    fn test_generate_schema_float() {
        let data = make_data(
            vec!["salary".to_string()],
            vec![
                vec!["50000.50".to_string()],
                vec!["60000.75".to_string()],
                vec!["75000.00".to_string()],
            ],
        );

        let schema = generate_schema(&data);
        assert_eq!(schema.properties["salary"].prop_type, "number");
    }

    #[test]
    fn test_generate_schema_empty_dataset() {
        let data = make_data(
            vec!["col1".to_string(), "col2".to_string()],
            Vec::new(),
        );

        let schema = generate_schema(&data);
        assert_eq!(schema.root_type, "object");
        assert!(schema.properties.contains_key("col1"));
        assert!(schema.properties.contains_key("col2"));
        // With no rows, all columns default to null type and are non-required
        assert_eq!(schema.properties["col1"].prop_type, "null");
        assert!(schema.required.is_empty());
    }

    #[test]
    fn test_schema_serialization() {
        let data = make_data(
            vec!["name".to_string(), "age".to_string()],
            vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
            ],
        );

        let schema = generate_schema(&data);
        let json = schema.to_string();
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("should be valid JSON");

        assert_eq!(parsed["$schema"], "https://json-schema.org/draft/2020-12/schema");
        assert_eq!(parsed["type"], "object");
        assert!(parsed["properties"].is_object());
        assert!(parsed["required"].is_array());
    }
}
