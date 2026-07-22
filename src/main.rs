//! zwse CLI entrypoint.

use zwse::{Cli, Error, ExtractedData, FileLocations, Result, VerbosityLevel, export_urls, parse_session_data};

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
        match file_locations.peek(5000) {
            Ok(peek) => {
                let n = std::cmp::min(5000, peek.len());
                let preview = String::from_utf8_lossy(&peek[..n]);
                println!("First {n} characters of session data: {preview}...");
            }
            Err(e) => eprintln!("Peek error: {e}"),
        }
    }

    let data: serde_json::Value = serde_json::from_str(&file_locations.session_json).map_err(Error::JsonParseError)?;

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

    entries.sort_by_key(|a| a.id);

    let output_file = cli.output_path();
    if let Some(parent) = output_file.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(Error::IoError)?;
    }

    let entries_len = t;

    export_urls(entries, output_file).map_err(|e| Error::UrlExportError(e.to_string()))?;

    if level.at_least(VerbosityLevel::Info) {
        println!("T value: {t}");
        println!("K value: {k}");
        println!("Exported URLs to: {}", output_file.display());
        println!("Total URLs: {entries_len}");
    }

    Ok(())
}
