use serde_json::Value;
use std::{fs, path::PathBuf};
use super::DataProvider;

pub struct FileProvider;

impl DataProvider for FileProvider {
    fn prefix(&self) -> &'static str {
        "x-data-file"
    }

    fn fetch(&self, arg: &str) -> Option<Value> {
        // 1. Safely slice off literal prefixes if explicitly provided in HTML
        let mut clean_arg = arg;
        
        if let Some(stripped) = clean_arg.strip_prefix("./") {
            clean_arg = stripped;
        }
        
        if let Some(stripped) = clean_arg.strip_prefix("data/") {
            clean_arg = stripped;
        }

        // 2. Build absolute compiler path relative to your public asset hub
        let data_path = PathBuf::from("public").join("data").join(clean_arg);
        
        match fs::read_to_string(&data_path) {
            Ok(content) => {
                // Try parsing the JSON string, catching the explicit error if it fails
                match serde_json::from_str::<Value>(&content) {
                    Ok(json_value) => Some(json_value),
                    Err(json_err) => {
                        eprintln!("❌ [JSON Syntax Error] File is malformed: {:?}", data_path);
                        eprintln!("   Details: {} (Line: {}, Column: {})", json_err, json_err.line(), json_err.column());
                        None
                    }
                }
            }
            Err(_) => {
                eprintln!("⚠️ [Data File Error] Could not read asset at: {:?}", data_path);
                None
            }
        }
    }
}

// -----------------------------------------------------------------
// UNIT TESTS
// -----------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_provider_prefix() {
        let provider = FileProvider;
        assert_eq!(provider.prefix(), "x-data-file");
    }

    #[test]
    fn test_fetch_valid_json_file() {
        let provider = FileProvider;
        
        // 1. Ensure our test directory structure exists safely
        let test_dir = PathBuf::from("public").join("data");
        fs::create_dir_all(&test_dir).unwrap();
        
        // 2. Write a temporary mock JSON file onto the disk
        let test_filename = "test_mock_inventory.json";
        let test_file_path = test_dir.join(test_filename);
        let mock_data = json!([
            { "sku": "TEST-01", "status": "Online" },
            { "sku": "TEST-02", "status": "Offline" }
        ]);
        
        fs::write(&test_file_path, serde_json::to_string(&mock_data).unwrap()).unwrap();

        // 3. Run our provider evaluation logic
        let result = provider.fetch(test_filename);
        
        // 4. Assertions
        assert!(result.is_some(), "Expected to successfully parse the JSON data file.");
        let json_value = result.unwrap();
        assert!(json_value.is_array());
        assert_eq!(json_value[0]["sku"], "TEST-01");
        assert_eq!(json_value[1]["status"], "Offline");

        // 5. Clean up the disk scratch file after successful test verification
        let _ = fs::remove_file(test_file_path);
    }

    #[test]
    fn test_fetch_nonexistent_file_returns_none() {
        let provider = FileProvider;
        
        // Passing a file name that definitely does not exist on disk
        let result = provider.fetch("this_file_does_not_exist_anywhere.json");
        
        assert!(result.is_none(), "Expected a missing file to return None safely without panicking.");
    }

    #[test]
    fn test_fetch_malformed_json_returns_none() {
        let provider = FileProvider;
        
        let test_dir = PathBuf::from("public").join("data");
        fs::create_dir_all(&test_dir).unwrap();
        
        let test_filename = "test_broken.json";
        let test_file_path = test_dir.join(test_filename);
        
        // Write invalid, broken raw text data that isn't true JSON format
        fs::write(&test_file_path, "{ broken key: unquoted value ").unwrap();

        let result = provider.fetch(test_filename);
        
        assert!(result.is_none(), "Expected corrupted JSON structures to be caught and return None.");
        
        let _ = fs::remove_file(test_file_path);
    }
}
