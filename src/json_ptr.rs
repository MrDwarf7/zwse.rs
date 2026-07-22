use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, slice};

use lz4_flex::decompress;

use crate::MAGIC_HEADER;
use crate::prelude::*;

pub trait FillBuf<T, N> {
    fn fill_buf(&mut self, data: T) -> Self;
}

impl FillBuf<&str, &[u8]> for JsonBufPtr {
    fn fill_buf(&mut self, json: &str) -> Self {
        println!("Filling buffer via JsonBufPtr");

        let data_ref = json.as_bytes();
        let data_ptr = data_ref.as_ptr();
        let size = data_ref.len();

        self.data_ptr = data_ptr;
        self.size = size;

        self.to_owned()
    }
}

#[derive(Debug, Clone)]
struct JsonBufPtr {
    data_ptr: *const u8,
    size:     usize,
}

impl JsonBufPtr {
    pub fn new(json: &str) -> Self {
        let mut s = JsonBufPtr {
            data_ptr: std::ptr::null(),
            size:     0,
        };
        s.fill_buf(json)
    }

    pub fn peek(&self, char_len: usize) -> Result<&[u8]> {
        if self.data_ptr.is_null() {
            return Err(Error::NullPointer);
        }
        println!("Peeking into buffer - data_ptr is not null");

        if char_len == 0 {
            return Err(Error::Generic("Peek failed".to_string()));
        }

        println!("Peeking into buffer - char length is not zero");

        let len = std::cmp::min(char_len, self.size);
        // Safety: data_ptr is not null (checked above) and len <= size.
        #[allow(unsafe_code)]
        // SAFETY: pointer non-null and length bounded by allocation size.
        let slice = unsafe { slice::from_raw_parts(self.data_ptr, len) };
        Ok(slice)
    }
}

impl Default for JsonBufPtr {
    fn default() -> Self {
        JsonBufPtr {
            data_ptr: std::ptr::null(),
            size:     0,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct FileLocations {
    pub profile_dir:  PathBuf,
    pub session_file: PathBuf,
    pub session_json: String,
    json_buf_ptr:     JsonBufPtr,
}

impl FillBuf<&str, &[u8]> for FileLocations {
    fn fill_buf(&mut self, data: &str) -> Self {
        println!("Filling buffer via FileLocations");
        let json_ptr = self.json_buf_ptr.fill_buf(data);
        self.json_buf_ptr = json_ptr;
        self.clone()
    }
}

impl FileLocations {
    /// Open + decompress an explicit sessionstore path (CLI / fixtures).
    pub fn open(session_file: impl AsRef<Path>) -> Result<Self> {
        let session_file = session_file.as_ref().to_path_buf();
        if !session_file.exists() {
            return Err(Error::SessionFileNotFound);
        }

        let mut s = Self {
            profile_dir: session_file.parent().map(Path::to_path_buf).unwrap_or_default(),
            session_file,
            session_json: String::new(),
            json_buf_ptr: JsonBufPtr::default(),
        };

        s.setup_session_json()?;
        s.json_buf_ptr = JsonBufPtr::new(&s.session_json);
        Ok(s)
    }

    /// Legacy path: discover first Zen profile + hard-coded session name.
    /// Prefer `open` + `Cli::resolve_session_file` for new code.
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        let mut s = Self::default();

        s.setup_profile_dir()
            .and_then(|s| s.setup_session_file())
            .and_then(|s| s.setup_session_json())
            .map_err(|e| Error::Generic(e.to_string()))?;

        s.json_buf_ptr = JsonBufPtr::new(&s.session_json).fill_buf(&s.session_json);
        println!("Filled FileLocations json_buf_ptr -- calling INNER fill_buf");
        Ok(s)
    }

    pub fn peek(&self, char_len: usize) -> Result<&[u8]> {
        self.json_buf_ptr.peek(char_len)
    }
}

#[allow(dead_code)]
impl FileLocations {
    fn setup_profile_dir(&mut self) -> Result<&mut Self> {
        let home = dirs::home_dir()
            .ok_or("Could not determine home directory - OS: Windows")
            .map_err(|e| Error::Generic(e.to_string()))?;
        let base_path = match std::env::consts::OS {
            "windows" => {
                let arr = ["AppData", "Roaming", "zen", "Profiles"].join("\\\\");
                home.join(&arr)
            }
            "linux" => {
                let arr = [".zen", "Profiles"].join("/");
                home.join(&arr)
            }
            "macos" => {
                let arr = ["Library", "Application Support", "zen", "Profiles"].join("/");
                home.join(&arr)
            }
            _ => {
                return Err(Error::Generic("Unsupported OS".to_string()));
            }
        };

        if !base_path.exists() {
            return Err(Error::ProfileDirNotFound(base_path.to_string_lossy().to_string()));
        }

        let entries = fs::read_dir(&base_path)
            .map_err(|e| Error::ProfileDirNotFound(format!("Unable to read the profile directory: {e}")))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.profile_dir = path;
                return Ok(self);
            }
        }

        Err(Error::ProfileDirNotFound(base_path.to_string_lossy().to_string()))
    }

    fn setup_session_file(&mut self) -> Result<&mut Self> {
        // Legacy hard-code kept for FileLocations::new only.
        let session_file = self.profile_dir.join("sessionstore_TESTING.jsonlz4");
        println!("Session file: {:?}", session_file);

        if !session_file.exists() {
            return Err(Error::SessionFileNotFound);
        }
        println!("Session file exists");

        self.session_file = session_file;

        Ok(self)
    }

    fn setup_session_json(&mut self) -> Result<&mut Self> {
        let mut file = File::open(&self.session_file).map_err(Error::IoError)?;
        println!("Session file opened");

        let mut magic = [0u8; 8];
        let _ = file.read_exact(&mut magic).map_err(|_e| Error::MagicHeader);

        if magic != MAGIC_HEADER {
            return Err(Error::MagicHeaderInvalid);
        }

        let mut compressed_data = vec![];
        file.read_to_end(&mut compressed_data).map_err(Error::IoError)?;

        if compressed_data.len() < 4 {
            return Err(Error::CompressedDataTooShort);
        }

        let expected_size = /* size of decompressed data, first 4 bytes LE */
            (compressed_data[0] as u32)
                | ((compressed_data[1] as u32) << 8)
                | ((compressed_data[2] as u32) << 16)
                | ((compressed_data[3] as u32) << 24);

        let decompressed_data =
            decompress(&compressed_data[4..], expected_size as usize).map_err(|_e| Error::DecompressionFailed)?;

        println!("Decompressing the data!");

        let data = String::from_utf8(decompressed_data).map_err(|_e| Error::DecompressedDataInvalidUtf8);
        self.session_json = data?;
        Ok(self)
    }
}
