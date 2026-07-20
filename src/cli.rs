use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{Parser, ValueEnum};

use crate::prelude::*;

/// Default export path relative to the process cwd.
pub const DEFAULT_OUTPUT: &str = "data/workspace_export_rs.txt";

/// Default sessionstore filename inside a Zen profile dir.
pub const DEFAULT_SESSION_NAME: &str = "sessionstore.jsonlz4";

/// Sentinel default for path args that mean "resolve automatically".
pub const PATH_AUTO: &str = "auto";

#[rustfmt::skip]
#[derive(Parser, Debug)]
#[command(
    name = clap::crate_name!(),
    author = clap::crate_authors!(),
    version = clap::crate_version!(),
    about = clap::crate_description!(),
    long_about = "\n\
Export tabs and workspaces from a Zen Browser (Firefox-style) sessionstore.jsonlz4.\n\
\n\
With no paths given, discovers the first Zen profile on this OS and reads\n\
sessionstore.jsonlz4 from it. Point --input at a fixture (e.g. data/sessionstore_operations.jsonlz4)\n\
to skip discovery. Use the literal value `auto` (default) for discoverable paths.\n",
    // All args have defaults -- bare `cargo run` should work.
    arg_required_else_help = false,
    // Custom -v / --version via Cli::new()
    disable_version_flag = true,
    styles = get_styles()
)]
pub struct Cli {
    /// Explicit path to a sessionstore.jsonlz4 (or fixture).
    /// `auto` (default) => discover via profile + --session-name.
    #[arg(
        short = 'i',
        long = "input",
        help = "Path to sessionstore.jsonlz4. Use 'auto' to discover via Zen profile.",
        value_hint = clap::ValueHint::FilePath,
        default_value = PATH_AUTO
    )]
    pub input: PathBuf,

    /// Zen profile directory. Only used when --input is auto.
    /// `auto` (default) => first profile under the OS Zen Profiles root.
    #[arg(
        short = 'p',
        long = "profile",
        help = "Zen profile directory. Use 'auto' for the first OS Zen profile.",
        value_hint = clap::ValueHint::DirPath,
        default_value = PATH_AUTO
    )]
    pub profile: PathBuf,

    /// Session filename inside the profile dir (ignored when --input is set).
    #[arg(
        short = 's',
        long = "session-name",
        help = "Session filename inside the profile directory.",
        value_hint = clap::ValueHint::Other,
        default_value = DEFAULT_SESSION_NAME
    )]
    pub session_name: String,

    /// Output path for the pipe-delimited export.
    #[arg(
        short = 'o',
        long = "output",
        help = "Output file path for the exported table.",
        value_hint = clap::ValueHint::FilePath,
        default_value = DEFAULT_OUTPUT
    )]
    pub output: PathBuf,

    /// Verbosity for diagnostic prints (no full tracing stack on this one-shot tool).
    /// 0=ERROR .. 4=TRACE. Default INFO.
    #[arg(
        value_enum,
        name = "level_verbosity",
        short = 'l',
        long = "level_verbosity",
        help = "Verbosity for diagnostic prints (ERROR|WARN|INFO|DEBUG|TRACE or 0-4).",
        default_value = "INFO",
        value_hint = clap::ValueHint::Other
    )]
    pub level_verbosity: VerbosityLevel,

    /// Print crate name + version and exit.
    #[arg(short = 'v', short_alias = 'V', long = "version", help = "Prints version information")]
    pub version: bool,
}

impl Cli {
    /// Parse argv. Keeps `Parser` / `::parse()` out of `main`.
    pub fn new() -> Self {
        let s = Self::parse();
        if s.version {
            println!("{} {}", clap::crate_name!(), clap::crate_version!());
            std::process::exit(0);
        }
        s
    }

    #[inline]
    pub fn verbosity_level(&self) -> VerbosityLevel {
        self.level_verbosity
    }

    /// Resolve the sessionstore path from --input, or profile + session-name.
    pub fn resolve_session_file(&self) -> Result<PathBuf> {
        if path_set(&self.input) {
            return Ok(self.input.clone());
        }

        let profile = if path_set(&self.profile) {
            self.profile.clone()
        } else {
            discover_zen_profile()?
        };

        Ok(profile.join(&self.session_name))
    }

    #[inline]
    pub fn output_path(&self) -> &Path {
        self.output.as_path()
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            input: PathBuf::from(PATH_AUTO),
            profile: PathBuf::from(PATH_AUTO),
            session_name: DEFAULT_SESSION_NAME.to_string(),
            output: PathBuf::from(DEFAULT_OUTPUT),
            level_verbosity: VerbosityLevel::Info,
            version: false,
        }
    }
}

/// Non-auto path = user supplied a real value (anti-Option sentinel).
#[inline]
fn path_set(p: &Path) -> bool {
    let s = p.as_os_str();
    !s.is_empty() && s != PATH_AUTO
}

/// First directory under the OS-specific Zen Profiles root.
pub fn discover_zen_profile() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        Error::Generic("Could not determine home directory".to_string())
    })?;

    // Zen has used both `.../zen/Profiles/<id>` and `.../zen/<id>` layouts.
    let candidates: Vec<PathBuf> = match std::env::consts::OS {
        "windows" => {
            let base = home.join("AppData").join("Roaming").join("zen");
            vec![base.join("Profiles"), base]
        }
        "linux" => {
            let base = home.join(".zen");
            vec![base.join("Profiles"), base]
        }
        "macos" => {
            let base = home
                .join("Library")
                .join("Application Support")
                .join("zen");
            vec![base.join("Profiles"), base]
        }
        _ => return Err(Error::Generic("Unsupported OS".to_string())),
    };

    let mut tried = Vec::new();
    for base_path in &candidates {
        tried.push(base_path.display().to_string());
        if !base_path.exists() {
            continue;
        }

        let entries = match std::fs::read_dir(base_path) {
            Ok(e) => e,
            Err(e) => {
                return Err(Error::ProfileDirNotFound(format!(
                    "Unable to read the profile directory {}: {e}",
                    base_path.display()
                )));
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            // Skip non-profile noise under ~/.zen (e.g. caches) by preferring
            // dirs that already contain a sessionstore*.jsonlz4.
            let has_session = ["sessionstore.jsonlz4", "sessionstore_TESTING.jsonlz4"]
                .iter()
                .any(|name| path.join(name).is_file());
            if has_session {
                return Ok(path);
            }
        }

        // Fallback: first subdir if none had a sessionstore (caller may still fail later).
        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    return Ok(path);
                }
            }
        }
    }

    Err(Error::ProfileDirNotFound(format!(
        "tried: {}",
        tried.join(", ")
    )))
}

/// Diagnostic verbosity. Defaults via clap -- no Option wrapping.
#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq, Default)]
#[clap(name = "VerbosityLevel", rename_all = "upper")]
pub enum VerbosityLevel {
    #[value(name = "ERROR", alias = "error", alias = "Error", alias = "0")]
    Error,
    #[value(name = "WARN", alias = "warn", alias = "Warn", alias = "1")]
    Warn,
    #[default]
    #[value(name = "INFO", alias = "info", alias = "Info", alias = "2")]
    Info,
    #[value(name = "DEBUG", alias = "debug", alias = "Debug", alias = "3")]
    Debug,
    #[value(name = "TRACE", alias = "trace", alias = "Trace", alias = "4")]
    Trace,
}

impl VerbosityLevel {
    /// Rank for simple `if level >= Debug` gates (no tracing crate).
    #[inline]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Error => 0,
            Self::Warn => 1,
            Self::Info => 2,
            Self::Debug => 3,
            Self::Trace => 4,
        }
    }

    #[inline]
    pub const fn at_least(self, other: Self) -> bool {
        self.rank() >= other.rank()
    }
}

impl From<u8> for VerbosityLevel {
    #[inline]
    fn from(level: u8) -> Self {
        match level {
            0 => Self::Error,
            1 => Self::Warn,
            2 => Self::Info,
            3 => Self::Debug,
            4 => Self::Trace,
            _ => Self::Info,
        }
    }
}

impl FromStr for VerbosityLevel {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "ERROR" | "0" => Ok(Self::Error),
            "WARN" | "1" => Ok(Self::Warn),
            "INFO" | "2" => Ok(Self::Info),
            "DEBUG" | "3" => Ok(Self::Debug),
            "TRACE" | "4" => Ok(Self::Trace),
            _ => Err(Error::Generic(format!("Verbosity level: {s} is not supported."))),
        }
    }
}

pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Blue))),
        )
        .literal(
            anstyle::Style::new()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightWhite))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)))
                .effects(anstyle::Effects::ITALIC),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
}
