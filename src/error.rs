// in-crate result type
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic error handler: {0}")]
    Generic(String),

    #[error("No Session file available")]
    SessionFileNotFound,

    #[error("Unable to export URLs from request file: {0}")]
    UrlExportError(String),

    #[error("Could not find profile directory. Tried with: {0}")]
    ProfileDirNotFound(String),

    #[error("Null pointer in JsonBufPtr")]
    NullPointer,

    #[error("Compressed Data is too short, or is empty")]
    CompressedDataTooShort,

    #[error("Decompression failed")]
    DecompressionFailed,

    #[error("Couldn't convert data in decompressed data into valid UTF-8 / string")]
    DecompressedDataInvalidUtf8,

    #[error("Couldn't read magic header value from supplied file")]
    MagicHeader,

    #[error("Invalid magic header")]
    MagicHeaderInvalid,

    #[error("Could not read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Could not parse JSON data: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Windows array not found in JSON data")]
    WindowsArrayNotFound,

    #[error("No tabs array found in JSON data")]
    TabsArrayNotFound,

    #[error("Couldn't pull field from JSON data: {0}")]
    FieldConversionError(String),

    #[error("Field not found in JSON data: {0}")]
    FieldNotFound(String),

    #[error("Tab object not found in JSON data")]
    TabObjectNotFound,
}
