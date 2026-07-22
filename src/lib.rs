//! zwse library -- sessionstore decompress + tab export helpers.

mod cli;
mod error;
mod extract;
mod json_ptr;
mod prelude;

use std::fs::File;
use std::io::Write;
use std::path::Path;

pub use cli::{Cli, DEFAULT_OUTPUT, DEFAULT_SESSION_NAME, PATH_AUTO, VerbosityLevel, discover_zen_profile, get_styles};
pub use error::{Error, Result};
pub use extract::ExtractedData;
pub use json_ptr::FileLocations;
use serde_json::Value;

/// Mozilla `sessionstore.jsonlz4` magic prefix (`mozLz40` + NUL).
pub const MAGIC_HEADER: &[u8] = b"mozLz40\0";

/// Walk session JSON -> per-tab entry arrays.
pub fn parse_session_data(json_data: Value) -> Result<Vec<Vec<Value>>> {
    let windows = window_data(json_data)?;
    let tabs = tab_data(windows)?;
    let entry_data = entry_data(tabs)?;
    Ok(entry_data)
}

fn window_data(json_data: Value) -> Result<Vec<Value>> {
    let jd = json_data
        .get("windows")
        .and_then(Value::as_array)
        .ok_or(Error::WindowsArrayNotFound)?
        .to_owned();
    Ok(jd)
}

fn tab_data(windows: Vec<Value>) -> Result<Vec<Vec<Value>>> {
    let wd = windows
        .into_iter()
        .map_while(|window| {
            let tabs = window
                .get("tabs")
                .and_then(Value::as_array)
                .ok_or(Error::TabsArrayNotFound)
                .unwrap();
            Some(tabs.to_owned())
        })
        .collect::<Vec<_>>();
    Ok(wd)
}

fn entry_data(tabs: Vec<Vec<Value>>) -> Result<Vec<Vec<Value>>> {
    let ed = tabs
        .into_iter()
        .flat_map(|tab| {
            tab.into_iter()
                .map_while(|tab| {
                    let entries = tab
                        .get("entries")
                        .and_then(Value::as_array)
                        .ok_or(Error::FieldNotFound("entries".to_string()))
                        .unwrap();
                    Some(entries.to_owned())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    Ok(ed)
}

/// Write pipe-delimited rows for each extracted entry.
pub fn export_urls(urls: Vec<ExtractedData>, output_file: impl AsRef<Path>) -> Result<()> {
    let mut file =
        File::create(output_file).map_err(|e| Error::UrlExportError(format!("Could not create file: {e}")))?;
    for url in urls.iter() {
        writeln!(
            file,
            "| {} | {} | {} | {} |{} | {} | {} | {} |",
            url.id,
            url.title,
            url.workspace,
            url.pinned,
            url.pinned_entry.as_ref().unwrap_or(&"None".to_string()),
            url.pinned_id,
            url.last_accessed,
            url.url,
        )
        .map_err(|e| Error::UrlExportError(format!("Could not write to file: {e}")))?;
    }
    Ok(())
}

/// Full pipeline: decompress session path -> extract -> sort by id -> write output.
pub fn run_export(session_path: &Path, output_path: &Path) -> Result<usize> {
    let file_locations = FileLocations::open(session_path)?;
    let data: Value = serde_json::from_str(&file_locations.session_json).map_err(Error::JsonParseError)?;
    let ext_data = parse_session_data(data)?;

    let mut entries = vec![];
    for tab in ext_data.into_iter() {
        for key in tab.into_iter() {
            entries.push(ExtractedData::extract_fields(&key)?);
        }
    }
    entries.sort_by_key(|a| a.id);

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(Error::IoError)?;
    }

    let n = entries.len();
    export_urls(entries, output_path).map_err(|e| Error::UrlExportError(e.to_string()))?;
    Ok(n)
}
