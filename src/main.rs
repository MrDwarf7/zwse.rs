// #![feature(portable_simd)]
// #![feature(stdarch_x86_avx512)]

mod cli;
mod error;
mod extract;
mod json_ptr;
mod prelude;
// mod sorting;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde_json::Value;

use crate::cli::{Cli, VerbosityLevel};
pub use crate::error::{Error, Result};
use crate::extract::ExtractedData;
use crate::json_ptr::FileLocations;
// use crate::sorting::*;

const MAGIC_HEADER: &[u8] = b"mozLz40\0";
const OUTPUT_WINDOW_CHAR_LEN: usize = 5000;

fn main() -> Result<()> {
    let cli = Cli::new();
    let level = cli.verbosity_level();

    let session_path = cli.resolve_session_file()?;
    if level.at_least(VerbosityLevel::Info) {
        println!("Session file: {}", session_path.display());
    }

    let file_locations = FileLocations::open(&session_path)?;
    if level.at_least(VerbosityLevel::Info) {
        println!("Profile dir: {}", file_locations.profile_dir.display());
    }

    if level.at_least(VerbosityLevel::Debug) {
        match file_locations.peek(OUTPUT_WINDOW_CHAR_LEN) {
            Ok(peek) => {
                let n = std::cmp::min(OUTPUT_WINDOW_CHAR_LEN, peek.len());
                let preview = String::from_utf8_lossy(&peek[..n]);
                println!("First {n} characters of session data: {preview}...");
            }
            Err(e) => eprintln!("Peek error: {e}"),
        }
    }

    let data: Value = serde_json::from_str(&file_locations.session_json).map_err(Error::JsonParseError)?;

    let ext_data = parse_session_data(data)?;

    let mut entries = vec![];

    let mut t = 0;
    let mut k = 0;
    for tab in ext_data.into_iter() {
        t += 1;
        for key in tab.into_iter() {
            k += 1;
            entries.push(ExtractedData::extract_fields(&key.to_owned())?);
        }
    }

    // entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries.sort_by_key(|a| a.id);
    // radix_sort(&mut entries);
    // radix_sort_simd(&mut entries);
    // radix_sort_avx(&mut entries);

    let output_file = cli.output_path();
    if let Some(parent) = output_file.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(Error::IoError)?;
        }
    }

    let entries_len = t; // outer loop count, not k (entries-in-tab)

    export_urls(entries, output_file).map_err(|e| Error::UrlExportError(e.to_string()))?;

    if level.at_least(VerbosityLevel::Info) {
        println!("T value: {t}");
        println!("K value: {k}");
        println!("Exported URLs to: {}", output_file.display());
        println!("Total URLs: {entries_len}");
    }

    Ok(())
}

fn parse_session_data(json_data: Value) -> Result<Vec<Vec<Value>>> {
    let windows = window_data(json_data)?;
    let tabs = tab_data(windows)?;
    let entry_data = entry_data(tabs)?;

    Ok(entry_data)
}

fn window_data(json_data: Value) -> Result<Vec<Value>> {
    let jd = json_data
        .to_owned()
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
                    // dbg!(&tab);
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

// TODO: Add TabWriter (BurntSushi) to write to file - will space out better and make it easier to read
fn export_urls(urls: Vec<ExtractedData>, output_file: impl AsRef<Path>) -> Result<()> {
    let mut file =
        File::create(output_file).map_err(|e| Error::UrlExportError(format!("Could not create file: {}", e)))?;
    for url in urls.iter() {
        writeln!(
            file,
            "| {} | {} | {} | {} |{} | {} | {} | {} |",
            url.id,
            url.title,
            url.workspace, // can be empty...
            url.pinned,
            url.pinned_entry.as_ref().unwrap_or(&"None".to_string()),
            url.pinned_id, // can be empty
            url.last_accessed,
            url.url,
        )
        .map_err(|e| Error::UrlExportError(format!("Could not write to file: {}", e)))?;
    }
    Ok(())
}

#[allow(dead_code)]
fn dump_all(data: &Value) {
    let file = File::create("dump.txt").unwrap();
    serde_json::to_writer_pretty(file, data).unwrap();
}

// fn extract_tab_url(tab: &Value) -> String {
//     let entries = match tab.get("entries").and_then(Value::as_array) {
//         Some(entries) if !entries.is_empty() => entries,
//         _ => return "about:blank".to_string(),
//     };
//
//     entries
//         .last()
//         .and_then(|entry| entry.get("url"))
//         .and_then(Value::as_str)
//         .unwrap_or("about:blank")
//         .to_string()
// }
