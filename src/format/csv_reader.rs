use crate::error::{DatascopeError, Result};
use crate::format::{DataSource, Row};
use std::io::Read;

/// Read CSV (or TSV) data from a reader.
pub fn read_csv(reader: &mut dyn Read, delimiter: u8) -> Result<DataSource> {
    let mut builder = csv::ReaderBuilder::new();
    builder
        .delimiter(delimiter)
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::Fields);

    let mut rdr = builder.from_reader(reader);

    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();

    if headers.is_empty() {
        return Err(DatascopeError::InvalidInput(
            "no headers found in CSV input".to_string(),
        ));
    }

    let mut rows: Vec<Row> = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let row: Row = record.iter().map(|f| f.to_string()).collect();
        rows.push(row);
    }

    if rows.is_empty() {
        return Err(DatascopeError::NoData);
    }

    Ok(DataSource { headers, rows })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        let csv_data = "name,age,city\nAlice,30,NYC\nBob,25,LA\n";
        let mut reader = std::io::Cursor::new(csv_data);
        let ds = read_csv(&mut reader, b',').unwrap();
        assert_eq!(ds.headers, vec!["name", "age", "city"]);
        assert_eq!(ds.rows.len(), 2);
        assert_eq!(ds.rows[0], vec!["Alice", "30", "NYC"]);
    }

    #[test]
    fn test_read_tsv() {
        let tsv_data = "name\tage\nAlice\t30\nBob\t25\n";
        let mut reader = std::io::Cursor::new(tsv_data);
        let ds = read_csv(&mut reader, b'\t').unwrap();
        assert_eq!(ds.headers, vec!["name", "age"]);
        assert_eq!(ds.rows.len(), 2);
    }

    #[test]
    fn test_empty_data() {
        let csv_data = "name,age\n";
        let mut reader = std::io::Cursor::new(csv_data);
        assert!(read_csv(&mut reader, b',').is_err());
    }
}
