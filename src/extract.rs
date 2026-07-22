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
    ) -> crate::error::Result<Self> {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn extract_fields_from_entry_object() {
        let tab = json!({
            "id": 42,
            "url": "https://example.com/",
            "title": "Example",
            "workspace": null,
            "pinned": false,
            "pinned_id": null,
            "last_accessed": 1234567890
        });
        let row = ExtractedData::extract_fields(&tab).expect("extract");
        assert_eq!(row.id, 42);
        // serde_json Value::to_string quotes strings
        assert!(row.url.contains("example.com"), "url={}", row.url);
        assert!(row.title.contains("Example"), "title={}", row.title);
        assert!(!row.pinned);
        assert_eq!(row.last_accessed, 1234567890);
        assert_eq!(row.workspace, "");
        assert_eq!(row.pinned_id, "");
    }

    #[test]
    fn set_field_null_workspace_becomes_empty() {
        let mut s = ExtractedData::default();
        s.set_field("workspace", "null".into());
        assert_eq!(s.workspace, "");
    }

    #[test]
    fn ord_by_id() {
        let a = ExtractedData {
            id: 1,
            ..Default::default()
        };
        let b = ExtractedData {
            id: 2,
            ..Default::default()
        };
        assert!(a < b);
    }
}
