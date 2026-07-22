//! Integration tests against a tiny in-repo mozLz40 fixture.

use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;
use zwse::{Error, FileLocations, MAGIC_HEADER, parse_session_data, run_export};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal.jsonlz4")
}

#[test]
fn fixture_exists_and_has_magic() {
    let p = fixture_path();
    assert!(p.is_file(), "missing fixture at {}", p.display());
    let bytes = std::fs::read(&p).expect("read fixture");
    assert!(bytes.len() > MAGIC_HEADER.len() + 4);
    assert_eq!(&bytes[..MAGIC_HEADER.len()], MAGIC_HEADER);
}

#[test]
fn open_missing_session_errors() {
    let err = FileLocations::open("/nonexistent/zwse-sessionstore.jsonlz4").unwrap_err();
    assert!(matches!(err, Error::SessionFileNotFound));
}

#[test]
fn open_bad_magic_errors() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("bad.jsonlz4");
    std::fs::write(&p, b"not-a-magic-header-plus-body").unwrap();
    let err = FileLocations::open(&p).unwrap_err();
    assert!(matches!(err, Error::MagicHeaderInvalid | Error::MagicHeader));
}

#[test]
fn open_and_parse_minimal_fixture() {
    let fl = FileLocations::open(fixture_path()).expect("open fixture");
    assert!(!fl.session_json.is_empty());
    let v: serde_json::Value = serde_json::from_str(&fl.session_json).expect("json");
    let tabs = parse_session_data(v).expect("parse");
    assert_eq!(tabs.len(), 1, "one tab of entries");
    assert_eq!(tabs[0].len(), 2, "two entries");
}

#[test]
fn run_export_writes_sorted_rows() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("export.txt");
    let n = run_export(&fixture_path(), &out).expect("export");
    assert_eq!(n, 2);
    let text = std::fs::read_to_string(&out).expect("read export");
    let lines: Vec<_> = text.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2);
    // sorted by id: 1 then 2
    assert!(lines[0].contains("| 1 |"), "first row id=1: {}", lines[0]);
    assert!(lines[1].contains("| 2 |"), "second row id=2: {}", lines[1]);
    assert!(lines[0].contains("example.com/a"));
    assert!(lines[1].contains("example.com/b"));
}

#[test]
fn cli_binary_exports_fixture() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("cli-out.txt");
    let bin = env!("CARGO_BIN_EXE_zwse");
    let status = Command::new(bin)
        .args([
            "-i",
            fixture_path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-l",
            "ERROR",
        ])
        .status()
        .expect("spawn zwse");
    assert!(status.success(), "zwse exit={status}");
    let text = std::fs::read_to_string(&out).expect("cli out");
    assert!(text.contains("example.com/a"));
    assert!(text.contains("example.com/b"));
}
