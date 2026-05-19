use rusqlite::Connection;
use serde_json::{Map, Value};
use std::path::PathBuf;
use super::DataProvider;

pub struct SqliteProvider;

impl DataProvider for SqliteProvider {
    fn prefix(&self) -> &'static str {
        "x-data-sqlite"
    }

    fn fetch(&self, arg: &str) -> Option<Value> {
        // Syntax format: "path/to/db.sqlite3|SELECT ... "
        let parts: Vec<&str> = arg.splitn(2, '|').collect();
        if parts.len() != 2 {
            eprintln!("❌ [SQLite Error] Invalid syntax. Expected 'db_path|SQL_QUERY'");
            return None;
        }

        let db_name = parts[0].trim();
        let query = parts[1].trim();

        // Safely map db file path into public/data/
        let db_path = PathBuf::from("public").join("data").join(db_name);

        let conn = match Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ [SQLite Database Error] Could not open database at {:?}: {}", db_path, e);
                return None;
            }
        };

        let mut stmt = match conn.prepare(query) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ [SQLite Query Error] SQL compilation failed: {}\nQuery: {}", e, query);
                return None;
            }
        };

        // Get column names dynamically so we don't have to hardcode any schemas!
        let col_count = stmt.column_count();
        let mut col_names = Vec::with_capacity(col_count);
        for i in 0..col_count {
            col_names.push(stmt.column_name(i).unwrap_or_default().to_string());
        }

        // Map database rows into a JSON Array
        let mut rows_array = Vec::new();

        let mut rows = match stmt.query([]) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("❌ [SQLite Execution Error] Query execution failed: {}", e);
                return None;
            }
        };

        while let Ok(Some(row)) = rows.next() {
            let mut row_map = Map::new();

            for (idx, name) in col_names.iter().enumerate() {
                // Interrogate SQLite storage type natively to extract correct data types
                let val = match row.get_ref(idx) {
                    Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(i)) => Value::Number(i.into()),
                    Ok(rusqlite::types::ValueRef::Real(f)) => {
                        if let Some(num) = serde_json::Number::from_f64(f) {
                            Value::Number(num)
                        } else {
                            Value::Null
                        }
                    }
                    Ok(rusqlite::types::ValueRef::Text(t)) => {
                        let text_str = std::str::from_utf8(t).unwrap_or_default();
                        Value::String(text_str.to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(b)) => {
                        Value::String(String::from_utf8_lossy(b).into_owned())
                    }
                    Err(_) => Value::Null,
                };
                row_map.insert(name.clone(), val);
            }
            rows_array.push(Value::Object(row_map));
        }

        Some(Value::Array(rows_array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_provider_flow() {
        let provider = SqliteProvider;
        
        // Ensure test space exists
        let test_dir = PathBuf::from("public").join("data");
        let _ = std::fs::create_dir_all(&test_dir);
        
        let test_db_name = "test_metrics.db";
        let test_db_path = test_dir.join(test_db_name);
        let _ = std::fs::remove_file(&test_db_path); // Clear old test junk

        // 1. Seed a quick mock test table
        let conn = Connection::open(&test_db_path).unwrap();
        conn.execute("CREATE TABLE system_nodes (id INTEGER, label TEXT, hex TEXT, active INTEGER)", []).unwrap();
        conn.execute("INSERT INTO system_nodes VALUES (501, 'SQLite Alpha Node', '#ef4444', 1)", []).unwrap();
        conn.execute("INSERT INTO system_nodes VALUES (502, 'SQLite Inactive Node', '#000000', 0)", []).unwrap();

        // 2. Query our provider using our pipeline syntax
        let arg = format!("{} | SELECT id, label, hex FROM system_nodes WHERE active = 1", test_db_name);
        let result = provider.fetch(&arg);

        // 3. Verify assertions
        assert!(result.is_some());
        let data = result.unwrap();
        assert!(data.is_array());
        
        let arr = data.as_array().unwrap();
        assert_eq!(arr.len(), 1); // Only active node returned
        assert_eq!(arr[0]["id"], 501);
        assert_eq!(arr[0]["label"], "SQLite Alpha Node");
        assert_eq!(arr[0]["hex"], "#ef4444");

        // Clean up scratch file
        let _ = std::fs::remove_file(test_db_path);
    }
}
