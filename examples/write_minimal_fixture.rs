// Build a tiny mozLz40 sessionstore fixture for tests.
// Format: magic(8) + uncompressed_size_le(u32) + lz4_block(payload)

use std::fs;
use std::path::PathBuf;

use lz4_flex::block::compress_prepend_size;
use zwse::MAGIC_HEADER;

fn main() {
    let json = r#"{"windows":[{"tabs":[{"entries":[
      {"id":2,"url":"https://example.com/b","title":"B","workspace":null,"pinned":false,"pinned_id":null,"last_accessed":200},
      {"id":1,"url":"https://example.com/a","title":"A","workspace":"ws1","pinned":true,"pinned_id":"p1","last_accessed":100}
    ]}]}]}"#;

    // compress_prepend_size already prepends u32 LE size -- matches FileLocations reader
    let compressed = compress_prepend_size(json.as_bytes());
    let mut out = Vec::with_capacity(MAGIC_HEADER.len() + compressed.len());
    out.extend_from_slice(MAGIC_HEADER);
    out.extend_from_slice(&compressed);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dest = root.join("tests/fixtures/minimal.jsonlz4");
    fs::create_dir_all(dest.parent().unwrap()).unwrap();
    fs::write(&dest, &out).unwrap();
    println!("wrote {} ({} bytes)", dest.display(), out.len());
}
