use serde_json::Value;
use super::DataProvider;

pub struct ApiProvider;

impl DataProvider for ApiProvider {
    fn prefix(&self) -> &'static str {
        "x-data-api"
    }

    fn fetch(&self, url: &str) -> Option<Value> {
        let clean_url = url.trim();

        // Wrap the blocking operation so Tokio knows how to handle it safely inside an async context
        tokio::task::block_in_place(|| {
            // 1. Build the blocking client safely inside the block_in_place context
            let client = match reqwest::blocking::Client::builder()
                .user_agent("Project-X-Template-Engine/1.0")
                .build() 
            {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ [API Client Error] Failed to initialize HTTP client: {}", e);
                    return None;
                }
            };

            println!("📡 [API Provider] Fetching remote data from: {}", clean_url);

            // 2. Execute the request
            match client.get(clean_url).send() {
                Ok(response) => {
                    if !response.status().is_success() {
                        eprintln!("❌ [API Error] HTTP request failed with status: {}", response.status());
                        return None;
                    }
                    
                    match response.json::<Value>() {
                        Ok(json_value) => Some(json_value),
                        Err(e) => {
                            eprintln!("❌ [API JSON Error] Failed to parse response payload as JSON: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ [API Network Error] Connection failed for URL ({}): {}", clean_url, e);
                    None
                }
            }
        })
    }
}
