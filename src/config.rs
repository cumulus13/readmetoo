use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub pager: bool,
    pub pager_cmd: Option<String>,
    pub width: Option<u16>,
    pub line_numbers: bool,
    pub themes_dir: Option<String>,
    pub show_header: bool,
    pub word_wrap: bool,
    pub indent: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "fruity".into(),
            pager: false,
            pager_cmd: None,
            width: None,
            line_numbers: false,
            themes_dir: None,
            show_header: true,
            word_wrap: true,
            indent: 2,
        }
    }
}

/// Load config using config-get builder API
pub fn load_config() -> Result<AppConfig> {
    use config_get::ConfigGet;

    // Build a ConfigGet that searches for "readmetoo" config in OS standard dirs
    let cfg_result = ConfigGet::builder("readmetoo")
        .config_dir("readmetoo")
        .build();

    match cfg_result {
        Ok(cg) => {
            // Read individual keys and build AppConfig
            let mut cfg = AppConfig::default();

            if let Some(v) = cg.get("theme") {
                cfg.theme = v.to_string();
            }
            if let Ok(v) = cg.parse::<bool>("pager") {
                cfg.pager = v;
            }
            if let Some(v) = cg.get("pager_cmd") {
                cfg.pager_cmd = Some(v.to_string());
            }
            if let Ok(v) = cg.parse::<u16>("width") {
                cfg.width = Some(v);
            }
            if let Ok(v) = cg.parse::<bool>("line_numbers") {
                cfg.line_numbers = v;
            }
            if let Some(v) = cg.get("themes_dir") {
                cfg.themes_dir = Some(v.to_string());
            }
            if let Ok(v) = cg.parse::<bool>("show_header") {
                cfg.show_header = v;
            }
            if let Ok(v) = cg.parse::<bool>("word_wrap") {
                cfg.word_wrap = v;
            }
            if let Ok(v) = cg.parse::<usize>("indent") {
                cfg.indent = v;
            }

            Ok(cfg)
        }
        Err(_) => Ok(AppConfig::default()),
    }
}

/// Return the default config file path for display purposes
pub fn config_path() -> PathBuf {
    if let Some(cfg_dir) = dirs::config_dir() {
        cfg_dir.join("readmetoo").join("readmetoo.toml")
    } else {
        PathBuf::from("~/.config/readmetoo/readmetoo.toml")
    }
}

pub fn default_config_toml() -> &'static str {
    r#"# readmetoo configuration
# Save this to the path shown by: readmetoo --config-path
#
# Platform defaults:
#   Linux/macOS : ~/.config/readmetoo/readmetoo.toml
#   Windows     : %APPDATA%\readmetoo\readmetoo.toml

# Theme name.  Run `readmetoo --list-themes` to see all choices.
# Built-in themes (Pygments-inspired):
#   dark  : fruity (default), monokai, dracula, nord, github-dark,
#            gruvbox-dark, one-dark, solarized-dark, material,
#            native, vim, zenburn, inkpot
#   light : default, emacs, friendly, solarized-light, tango, autumn, bw
theme = "fruity"

# Enable the built-in pager automatically for long files
pager = false

# Use a specific external pager command instead of the built-in one.
# The command receives the rendered output on stdin.
# pager_cmd = "less -R"

# Hard-code terminal width for line-wrapping (omit to auto-detect)
# width = 100

# Show line numbers in fenced code blocks
line_numbers = false

# Directory containing custom *.theme.toml files
# themes_dir = "~/.config/readmetoo/themes"

# Show a filename banner when reading multiple files at once
show_header = true

# Word-wrap long paragraphs to terminal width
word_wrap = true

# Number of spaces per indent level in nested lists
indent = 2
"#
}
