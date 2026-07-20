use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ExtractedData {
    pub id:            usize,
    pub url:           String,
    pub title:         String,
    pub workspace:     String,
    pub pinned:        bool,
    pub pinned_entry:  Option<String>,
    pub pinned_id:     String,
    pub last_accessed: usize, // Unix timestamp -- VERY large number
    size:              usize,
}

impl Eq for ExtractedData {}

impl PartialEq<Self> for ExtractedData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd<Self> for ExtractedData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExtractedData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl ExtractedData {
    pub fn extract_fields(
        tab: &serde_json::Value, // serde_json::map::Map<String, Value>
    ) -> crate::Result<Self> {
        let mut s = ExtractedData::default();

        for key in tab.as_object().unwrap().keys() {
            // println!("Extraction - KEY: {} ||  VAL: {}", key, tab[key]);
            s.set_field(key.as_str(), tab[key].to_string());
            // let value = tab[key].to_string();
            // println!("Value: {}", value);
            s.add_size();
        }
        // println!("Values done");
        // std::thread::sleep(std::time::Duration::from_secs(10));

        // println!("Size: {}", s.get_size());

        Ok(s)
    }

    // pub fn get_size(&self) -> usize {
    //     self.size
    // }

    fn add_size(&mut self) {
        self.size += 1;
    }

    pub fn set_field(&mut self, field: &str, value: String) {
        match field.to_lowercase().as_str() {
            "id" => self.id = value.parse().unwrap(),
            "url" => self.url = value.to_string(),
            "title" => self.title = value,
            "workspace" => self.workspace = maybe_empty_string(&value),
            "pinned" => self.pinned = value.parse().unwrap(),
            "pinned_entry" => self.pinned_entry = Some(value),
            "pinned_id" => self.pinned_id = maybe_empty_string(&value),
            "last_accessed" => self.last_accessed = value.parse().unwrap(),
            _ => (),
        }
    }
}

fn maybe_empty_string(value: &str) -> String {
    match value {
        "null" => "".to_string(),
        v => {
            match !v.is_empty() {
                true => v.to_string(),
                false => "".to_string(),
            }
        }
    }
}
