use crate::config::{config_path, default_config_toml, load_config};
use crate::error::ReadmetooError;
use crate::pager;
use crate::renderer::render_to_string;
use crate::theme::{get_theme, list_themes};
use anyhow::{Context, Result};
use clap::{ArgAction, Parser};
use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// readmetoo — Beautiful Markdown reader for your terminal
///
/// Renders Markdown with rich formatting, Pygments-inspired color themes,
/// Unicode tables, syntax-highlighted code blocks, and an optional pager.
#[derive(Parser, Debug)]
#[command(
    name = "readmetoo",
    version,
    author = "Hadi Cahyadi <cumulus13@gmail.com>",
    about = "Beautiful Markdown reader for your terminal"
)]
pub struct Cli {
    /// Markdown file(s) to display (reads from stdin when omitted)
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Color theme name (overrides config file). Use --list-themes to browse.
    #[arg(short = 't', long, value_name = "THEME", env = "READMETOO_THEME")]
    pub theme: Option<String>,

    /// Enable the built-in pager for long output
    #[arg(short = 'p', long, action = ArgAction::SetTrue)]
    pub pager: bool,

    /// Disable the pager even when enabled in config
    #[arg(short = 'P', long = "no-pager", action = ArgAction::SetTrue)]
    pub no_pager: bool,

    /// External pager command to pipe output into (e.g. "less -R")
    #[arg(long, value_name = "CMD", env = "PAGER")]
    pub pager_cmd: Option<String>,

    /// Show line numbers inside code blocks
    #[arg(short = 'n', long, action = ArgAction::SetTrue)]
    pub line_numbers: bool,

    /// Hard-code terminal width for line-wrapping
    #[arg(short = 'w', long, value_name = "COLS")]
    pub width: Option<u16>,

    /// Disable paragraph word-wrapping
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_wrap: bool,

    /// List all available built-in themes and exit
    #[arg(short = 'l', long = "list-themes", action = ArgAction::SetTrue)]
    pub list_themes: bool,

    /// Print sample rendered output for the named theme, then exit
    #[arg(long, value_name = "THEME")]
    pub preview_theme: Option<String>,

    /// Print the config file path for this platform and exit
    #[arg(long, action = ArgAction::SetTrue)]
    pub config_path: bool,

    /// Print a documented default config file to stdout and exit
    #[arg(long, action = ArgAction::SetTrue)]
    pub print_config: bool,

    /// Suppress filename banners when reading multiple files
    #[arg(short = 'q', long, action = ArgAction::SetTrue)]
    pub quiet: bool,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        // ── informational exits ──────────────────────────────────────────────
        if self.list_themes {
            return self.do_list_themes();
        }
        if self.config_path {
            println!("{}", config_path().display().to_string().bright_cyan());
            return Ok(());
        }
        if self.print_config {
            print!("{}", default_config_toml());
            return Ok(());
        }

        // ── load config ──────────────────────────────────────────────────────
        let cfg = load_config().unwrap_or_default();

        // ── theme preview ────────────────────────────────────────────────────
        if let Some(ref name) = self.preview_theme {
            return self.do_preview(name);
        }

        // ── resolve settings ─────────────────────────────────────────────────
        let theme_name = self.theme.as_deref().unwrap_or(&cfg.theme);
        let theme =
            get_theme(theme_name).ok_or_else(|| ReadmetooError::theme_not_found(theme_name))?;

        let use_pager = !self.no_pager && (self.pager || cfg.pager);
        let line_numbers = self.line_numbers || cfg.line_numbers;
        let word_wrap = !self.no_wrap && cfg.word_wrap;
        let indent = cfg.indent;
        let show_header = !self.quiet && cfg.show_header;

        // ── collect & render ─────────────────────────────────────────────────
        let mut parts: Vec<(String, String)> = Vec::new();

        if self.files.is_empty() {
            let input = io::read_to_string(io::stdin()).context("Failed to read from stdin")?;
            let rendered = render_to_string(&input, &theme, line_numbers, word_wrap, indent)?;
            parts.push(("stdin".into(), rendered));
        } else {
            for path in &self.files {
                let content = fs::read_to_string(path)
                    .with_context(|| format!("Cannot read '{}'", path.display()))?;
                let rendered = render_to_string(&content, &theme, line_numbers, word_wrap, indent)?;
                parts.push((path.display().to_string(), rendered));
            }
        }

        // ── assemble output ──────────────────────────────────────────────────
        let mut output = String::new();
        for (title, body) in &parts {
            if show_header && parts.len() > 1 {
                output.push_str(&file_banner(title));
            }
            output.push_str(body);
        }

        // ── emit ─────────────────────────────────────────────────────────────
        if use_pager {
            let title = parts
                .first()
                .map(|(t, _)| t.as_str())
                .unwrap_or("readmetoo");
            if let Some(cmd) = self.pager_cmd.as_deref().or(cfg.pager_cmd.as_deref()) {
                pager::external_pager(&output, cmd)?;
            } else {
                pager::page_content(&output, Some(title))?;
            }
        } else {
            io::stdout().lock().write_all(output.as_bytes())?;
        }

        Ok(())
    }

    fn do_list_themes(&self) -> Result<()> {
        println!("{}", "Available themes".bright_yellow().bold());
        println!(
            "{}",
            "─────────────────────────────────────────────────────────────".bright_black()
        );
        let themes = list_themes();
        for name in &themes {
            if let Some(t) = crate::theme::get_theme(name) {
                let kind = if t.dark {
                    " dark ".on_bright_black().black()
                } else {
                    "light ".on_bright_white().black()
                };
                let star = if name.as_str() == "fruity" {
                    " ★ default".bright_green().to_string()
                } else {
                    String::new()
                };
                println!(
                    "  {:<20} [{}]  {}{}",
                    name.bright_cyan().bold(),
                    kind,
                    t.description.white(),
                    star,
                );
            }
        }
        println!();
        println!(
            "{}",
            format!("{} themes available", themes.len()).bright_black()
        );
        println!(
            "{}",
            "Usage: readmetoo --theme <name> <file.md>".bright_white()
        );
        Ok(())
    }

    fn do_preview(&self, name: &str) -> Result<()> {
        let theme = get_theme(name).ok_or_else(|| ReadmetooError::theme_not_found(name))?;
        let sample = sample_markdown(name);
        let rendered = render_to_string(&sample, &theme, false, true, 2)?;
        io::stdout().lock().write_all(rendered.as_bytes())?;
        Ok(())
    }
}

fn file_banner(title: &str) -> String {
    let line = "═".repeat(title.len() + 4);
    format!("\n╔{}╗\n║  {}  ║\n╚{}╝\n\n", line, title, line)
}

fn sample_markdown(name: &str) -> String {
    format!(
        r#"# {name} — Theme Preview

## Typography

This paragraph contains **bold text**, *italic text*, ~~strikethrough~~,
and `inline code`. A [link](https://github.com/cumulus13/readmetoo) is shown too.

## Code Block

```rust
fn main() {{
    // Hello from readmetoo!
    let theme = "{name}";
    println!("Active theme: {{theme}}");
    let answer: u32 = 42;
}}
```

## Lists

- First bullet item
- Second item with **bold**
  - Nested bullet
    - Deeply nested

1. Ordered first
2. Ordered second
3. Ordered third

## Task List

- [x] Implement themes
- [x] Add pager support
- [ ] Write more tests

## Blockquote

> *"Any color you like, as long as it's {name}."*

## Table

| Feature        | Status   | Notes             |
|----------------|----------|-------------------|
| Themes         | ✅ Done  | {name} active     |
| Pager          | ✅ Done  | minus + external  |
| Code highlight | ✅ Done  | Per-language      |
| Tables         | ✅ Done  | Unicode borders   |

---

~~Old style~~ → **New style** • Powered by `readmetoo`
"#
    )
}
