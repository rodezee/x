use serde_json::Value;
use std::collections::HashMap;
use tl::Attributes;

pub mod file;
pub mod sqlite; // 1. Expose our new file module!

pub trait DataProvider {
    fn prefix(&self) -> &'static str;
    fn fetch(&self, arg: &str) -> Option<Value>;
}

pub struct DataManager {
    providers: HashMap<&'static str, Box<dyn DataProvider>>,
}

impl DataManager {
    pub fn new() -> Self {
        let mut providers: HashMap<&'static str, Box<dyn DataProvider>> = HashMap::new();
        
        // Register File Provider
        let file_prov = file::FileProvider;
        providers.insert(file_prov.prefix(), Box::new(file_prov));
        
        // 2. Register SQLite Provider
        let sqlite_prov = sqlite::SqliteProvider;
        providers.insert(sqlite_prov.prefix(), Box::new(sqlite_prov));
        
        Self { providers }
    }

    /// Scans a tag's attributes to see if any registered data provider matches
    pub fn extract_scope(&self, attributes: &Attributes) -> Option<Value> {
        for (key, val) in attributes.iter() {
            // key is a Cow<str>, so we can reference it directly as a &str slice
            let key_str = key.as_ref();
            
            if let Some(provider) = self.providers.get(key_str) {
                if let Some(attr_val) = val {
                    // attr_val is Option<Cow<str>>, map it cleanly to a string slice
                    return provider.fetch(attr_val.as_ref());
                }
            }
        }
        None
    }
}
