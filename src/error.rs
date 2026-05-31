use thiserror::Error;

/// Top-level errors for readmetoo.
///
/// These are used internally; most user-facing errors are surfaced via `anyhow`.
#[derive(Debug, Error)]
#[allow(dead_code)] // variants reserved for library / future CLI use
pub enum ReadmetooError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read file '{path}': {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Theme '{0}' not found. Run `readmetoo --list-themes` to see available themes.")]
    ThemeNotFound(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Rendering error: {0}")]
    RenderError(String),

    #[error("Pager error: {0}")]
    PagerError(String),
}

impl ReadmetooError {
    #[allow(dead_code)]
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound(path.into())
    }
    pub fn theme_not_found(name: impl Into<String>) -> Self {
        Self::ThemeNotFound(name.into())
    }
}
