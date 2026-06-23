use crate::theme::Theme;
use anyhow::Result;
use crossterm::style::{Attribute, ResetColor, SetAttribute, SetForegroundColor};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag};
use std::io::Write;
use terminal_size::{terminal_size, Width};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Visible character width of a string (strips ANSI, counts Unicode graphemes simply)
fn visible_len(s: &str) -> usize {
    // Strip ANSI escape sequences then count chars
    let mut len = 0usize;
    let mut in_esc = false;
    for c in s.chars() {
        if in_esc {
            if c == 'm' {
                in_esc = false;
            }
        } else if c == '\x1b' {
            in_esc = true;
        } else {
            len += 1;
        }
    }
    len
}

/// Pad a plain string to `width` visible chars with spaces on the right
fn pad_right(s: &str, width: usize) -> String {
    let vlen = visible_len(s);
    if vlen >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - vlen))
    }
}

// ── Renderer ─────────────────────────────────────────────────────────────────

pub struct Renderer<W: Write> {
    out: W,
    theme: Theme,
    term_width: u16,
    line_numbers: bool,
    word_wrap: bool,
    indent: usize,

    // list state
    list_depth: usize,
    ordered_counters: Vec<Option<u64>>, // None = unordered, Some(n) = ordered at n

    // paragraph / inline text buffer
    para_buf: String,
    in_blockquote: bool,

    // heading state
    heading_text: String,
    heading_level: HeadingLevel,
    in_heading: bool,

    // inline emphasis state (for para_buf coloring we pre-emit ANSI into para_buf)
    // We track nesting depth to know when to reset
    strong_depth: usize,
    em_depth: usize,
    strike_depth: usize,

    // link state
    in_link: bool,
    link_dest: String,
    link_text: String, // accumulate link display text separately

    // image state
    in_image: bool,
    image_alt: String,

    // code block state
    code_buf: Option<String>,
    code_lang: Option<String>,

    // table state — per table, rebuilt each time
    table_col_widths: Vec<usize>,
    table_rows: Vec<TableRow>,
    in_table: bool,
    in_table_head: bool,
    in_table_cell: bool,
    cur_cell_plain: String, // plain text for width measurement
    cur_cell_rich: String,  // ANSI-colored text for display
    cur_row_plain: Vec<String>,
    cur_row_rich: Vec<String>,
}

struct TableRow {
    plain: Vec<String>, // plain text (for measuring)
    rich: Vec<String>,  // ANSI-colored (for display)
    is_header: bool,
}

impl<W: Write> Renderer<W> {
    pub fn new(out: W, theme: Theme, line_numbers: bool, word_wrap: bool, indent: usize) -> Self {
        let term_width = terminal_size()
            .map(|(Width(w), _)| w)
            .unwrap_or(100)
            .min(200);
        Self {
            out,
            theme,
            term_width,
            line_numbers,
            word_wrap,
            indent,
            list_depth: 0,
            ordered_counters: Vec::new(),
            para_buf: String::new(),
            in_blockquote: false,
            heading_text: String::new(),
            heading_level: HeadingLevel::H1,
            in_heading: false,
            strong_depth: 0,
            em_depth: 0,
            strike_depth: 0,
            in_link: false,
            link_dest: String::new(),
            link_text: String::new(),
            in_image: false,
            image_alt: String::new(),
            code_buf: None,
            code_lang: None,
            table_col_widths: Vec::new(),
            table_rows: Vec::new(),
            in_table: false,
            in_table_head: false,
            in_table_cell: false,
            cur_cell_plain: String::new(),
            cur_cell_rich: String::new(),
            cur_row_plain: Vec::new(),
            cur_row_rich: Vec::new(),
        }
    }

    // ── low-level ANSI ───────────────────────────────────────────────────────

    fn fg(&mut self, c: &crate::theme::AnsiColor) -> Result<()> {
        write!(self.out, "{}", SetForegroundColor(c.to_crossterm()))?;
        Ok(())
    }
    fn reset_color(&mut self) -> Result<()> {
        write!(self.out, "{}", ResetColor)?;
        Ok(())
    }
    fn attr(&mut self, a: Attribute) -> Result<()> {
        write!(self.out, "{}", SetAttribute(a))?;
        Ok(())
    }
    fn reset_attr(&mut self) -> Result<()> {
        write!(self.out, "{}", SetAttribute(Attribute::Reset))?;
        Ok(())
    }

    // Versions that write to a String buffer instead of self.out
    fn fg_str(c: &crate::theme::AnsiColor) -> String {
        format!("{}", SetForegroundColor(c.to_crossterm()))
    }
    fn reset_str() -> String {
        format!("{}{}", ResetColor, SetAttribute(Attribute::Reset))
    }
    fn bold_str() -> String {
        format!("{}", SetAttribute(Attribute::Bold))
    }
    fn italic_str() -> String {
        format!("{}", SetAttribute(Attribute::Italic))
    }
    fn strike_str() -> String {
        format!("{}", SetAttribute(Attribute::CrossedOut))
    }
    fn underline_str() -> String {
        format!("{}", SetAttribute(Attribute::Underlined))
    }

    // ── newline / horizontal rule ────────────────────────────────────────────

    fn newline(&mut self) -> Result<()> {
        writeln!(self.out)?;
        Ok(())
    }

    fn hr(&mut self) -> Result<()> {
        let c = self.theme.colors.rule.clone();
        self.fg(&c)?;
        writeln!(self.out, "{}", "─".repeat(self.term_width as usize))?;
        self.reset_color()
    }

    // ── heading ──────────────────────────────────────────────────────────────

    fn print_heading(&mut self, text: &str, level: HeadingLevel) -> Result<()> {
        self.newline()?;
        let (prefix, color) = match level {
            HeadingLevel::H1 => ("█ ", self.theme.colors.h1.clone()),
            HeadingLevel::H2 => ("▌ ", self.theme.colors.h2.clone()),
            HeadingLevel::H3 => ("◆ ", self.theme.colors.h3.clone()),
            _ => ("◇ ", self.theme.colors.h4.clone()),
        };
        self.attr(Attribute::Bold)?;
        self.fg(&color)?;
        write!(self.out, "{}{}", prefix, text)?;
        self.reset_attr()?;
        self.reset_color()?;
        self.newline()?;
        if matches!(level, HeadingLevel::H1 | HeadingLevel::H2) {
            let width =
                (prefix.chars().count() + text.chars().count()).min(self.term_width as usize);
            let under = if matches!(level, HeadingLevel::H1) {
                "═"
            } else {
                "─"
            };
            self.fg(&color)?;
            writeln!(self.out, "{}", under.repeat(width))?;
            self.reset_color()?;
        }
        Ok(())
    }

    // ── paragraph / word-wrap flush ──────────────────────────────────────────

    fn flush_para(&mut self) -> Result<()> {
        if self.para_buf.is_empty() {
            return Ok(());
        }
        let text = self.para_buf.clone();
        self.para_buf.clear();

        let indent_str = if self.in_blockquote {
            "  │ ".to_string()
        } else {
            " ".repeat(self.list_depth * self.indent)
        };

        // We need to word-wrap the visible text but preserve ANSI codes.
        // Strategy: split by whitespace on PLAIN text positions, emit rich text.
        // Since we mixed ANSI into para_buf already, we do a simpler approach:
        // split on space/newline boundaries in the raw string (ANSI codes never
        // contain spaces), then wrap by visible width.
        let avail = (self.term_width as usize)
            .saturating_sub(indent_str.len())
            .max(20);

        if self.word_wrap {
            // Split into tokens (words + their trailing ANSI) at whitespace
            let mut col = 0usize;
            let mut line_started = false;

            // Tokenise: split on whitespace, but preserve ANSI seqs attached to words
            // We iterate char-by-char building tokens
            let mut tokens: Vec<String> = Vec::new();
            let mut tok = String::new();
            let mut in_esc = false;
            for ch in text.chars() {
                if in_esc {
                    tok.push(ch);
                    if ch == 'm' {
                        in_esc = false;
                    }
                } else if ch == '\x1b' {
                    in_esc = true;
                    tok.push(ch);
                } else if ch == ' ' || ch == '\n' {
                    if !tok.is_empty() {
                        tokens.push(tok.clone());
                        tok.clear();
                    }
                } else {
                    tok.push(ch);
                }
            }
            if !tok.is_empty() {
                tokens.push(tok);
            }

            for word in &tokens {
                let wlen = visible_len(word);
                if !line_started {
                    write!(self.out, "{}", indent_str)?;
                    line_started = true;
                } else if col + 1 + wlen > avail {
                    writeln!(self.out)?;
                    write!(self.out, "{}", indent_str)?;
                    col = 0;
                } else {
                    write!(self.out, " ")?;
                    col += 1;
                }
                write!(self.out, "{}", word)?;
                col += wlen;
            }
            if line_started {
                // always reset color at end of paragraph
                write!(self.out, "{}", ResetColor)?;
                write!(self.out, "{}", SetAttribute(Attribute::Reset))?;
                writeln!(self.out)?;
            }
        } else {
            for line in text.split('\n') {
                writeln!(self.out, "{}{}", indent_str, line)?;
            }
            write!(self.out, "{}", ResetColor)?;
            write!(self.out, "{}", SetAttribute(Attribute::Reset))?;
        }
        Ok(())
    }

    // ── code block ───────────────────────────────────────────────────────────

    fn render_code_block(&mut self, code: &str, lang: Option<&str>) -> Result<()> {
        let border_color = self.theme.colors.table_border.clone();
        let cmt_color = self.theme.colors.code_comment.clone();
        let code_color = self.theme.colors.code_inline.clone();
        let kw_color = self.theme.colors.code_keyword.clone();

        // inner width = term_width - 2 border chars, capped at 98
        let inner = ((self.term_width as usize).min(100)).saturating_sub(2);

        // top border
        self.fg(&border_color)?;
        writeln!(self.out, "╭{}╮", "─".repeat(inner))?;

        // language label
        if let Some(l) = lang {
            if !l.is_empty() {
                let label = format!(" {} ", l);
                let pad = inner.saturating_sub(label.chars().count());
                write!(self.out, "│")?;
                self.attr(Attribute::Bold)?;
                self.fg(&kw_color)?;
                write!(self.out, "{}", label)?;
                self.reset_attr()?;
                self.fg(&border_color)?;
                writeln!(self.out, "{}│", " ".repeat(pad))?;
                writeln!(self.out, "├{}┤", "─".repeat(inner))?;
            }
        }
        self.reset_color()?;

        let comment_marker: &str = match lang {
            Some("python") | Some("py") | Some("ruby") | Some("rb") | Some("shell")
            | Some("bash") | Some("sh") | Some("zsh") | Some("toml") | Some("yaml")
            | Some("yml") | Some("bat") | Some("powershell") | Some("ps1") => "#",
            Some("sql") | Some("lua") | Some("haskell") | Some("hs") => "--",
            _ => "//", // rust, c, cpp, java, go, js, ts, swift …
        };

        // code lines — we need to pad to exactly `inner - line_num_width` chars
        let ln_width = if self.line_numbers {
            let total = code.lines().count();
            format!("{}", total).len() + 1 // e.g. "42 " = 3 chars
        } else {
            0
        };
        let content_width = inner.saturating_sub(1 + ln_width); // 1 for leading space

        for (i, line) in code.lines().enumerate() {
            self.fg(&border_color)?;
            write!(self.out, "│")?;

            if self.line_numbers {
                self.fg(&cmt_color)?;
                write!(self.out, "{:>width$} ", i + 1, width = ln_width - 1)?;
            } else {
                write!(self.out, " ")?;
            }

            // colorize
            let trimmed = line.trim_start();
            let is_comment = trimmed.starts_with(comment_marker)
                || (lang == Some("html") || lang == Some("xml")) && trimmed.starts_with("<!--");
            if is_comment {
                self.fg(&cmt_color)?;
                self.attr(Attribute::Italic)?;
            } else {
                self.fg(&code_color)?;
            }

            // Measure visible line length and pad to content_width
            let vlen = line.chars().count();
            write!(self.out, "{}", line)?;
            let pad = content_width.saturating_sub(vlen);
            write!(self.out, "{}", " ".repeat(pad))?;

            self.reset_attr()?;
            self.reset_color()?;
            self.fg(&border_color)?;
            writeln!(self.out, "│")?;
        }

        // bottom border
        self.fg(&border_color)?;
        writeln!(self.out, "╰{}╯", "─".repeat(inner))?;
        self.reset_color()?;
        self.newline()?;
        Ok(())
    }

    // ── table ────────────────────────────────────────────────────────────────

    /// Compute column widths from accumulated rows (must be called before flush_table)
    fn compute_col_widths(&mut self) {
        let mut widths: Vec<usize> = Vec::new();
        for row in &self.table_rows {
            for (ci, cell) in row.plain.iter().enumerate() {
                let w = cell.chars().count();
                if ci < widths.len() {
                    if w > widths[ci] {
                        widths[ci] = w;
                    }
                } else {
                    widths.push(w.max(1));
                }
            }
        }
        self.table_col_widths = widths;
    }

    fn flush_table(&mut self) -> Result<()> {
        if self.table_rows.is_empty() {
            return Ok(());
        }

        self.compute_col_widths();
        let widths = self.table_col_widths.clone();
        let cols = widths.len();

        let border = self.theme.colors.table_border.clone();
        let header_color = self.theme.colors.table_header.clone();
        let fg = self.theme.colors.foreground.clone();

        // ── top border ────────────────────────────────────────────────────────
        self.fg(&border)?;
        write!(self.out, "┌")?;
        for (i, &w) in widths.iter().enumerate() {
            write!(self.out, "{}", "─".repeat(w + 2))?;
            if i + 1 < cols {
                write!(self.out, "┬")?;
            }
        }
        writeln!(self.out, "┐")?;

        let rows = std::mem::take(&mut self.table_rows);

        let mut prev_was_header = false;
        for (ri, row) in rows.iter().enumerate() {
            // ── header/body separator ─────────────────────────────────────────
            if ri > 0 && prev_was_header && !row.is_header {
                self.fg(&border)?;
                write!(self.out, "├")?;
                for (i, &w) in widths.iter().enumerate() {
                    write!(self.out, "{}", "═".repeat(w + 2))?;
                    if i + 1 < cols {
                        write!(self.out, "╪")?;
                    }
                }
                writeln!(self.out, "┤")?;
            }

            // ── row ──────────────────────────────────────────────────────────
            self.fg(&border)?;
            write!(self.out, "│")?;
            for (ci, cell_plain) in row.plain.iter().enumerate() {
                let w = widths.get(ci).copied().unwrap_or(1);
                write!(self.out, " ")?;
                if row.is_header {
                    self.attr(Attribute::Bold)?;
                    self.fg(&header_color)?;
                } else {
                    self.fg(&fg)?;
                }
                // Use the rich (colored) version for display but pad by plain width
                let rich = row.rich.get(ci).map(String::as_str).unwrap_or(cell_plain);
                let plain_len = cell_plain.chars().count();
                write!(self.out, "{}", rich)?;
                // pad using plain length so we don't count ANSI escape bytes
                let pad = w.saturating_sub(plain_len);
                write!(self.out, "{}", " ".repeat(pad))?;
                self.reset_attr()?;
                self.fg(&border)?;
                write!(self.out, " │")?;
            }
            writeln!(self.out)?;

            prev_was_header = row.is_header;
        }

        // ── bottom border ─────────────────────────────────────────────────────
        self.fg(&border)?;
        write!(self.out, "└")?;
        for (i, &w) in widths.iter().enumerate() {
            write!(self.out, "{}", "─".repeat(w + 2))?;
            if i + 1 < cols {
                write!(self.out, "┴")?;
            }
        }
        writeln!(self.out, "┘")?;
        self.reset_color()?;
        self.newline()?;

        self.table_col_widths.clear();
        Ok(())
    }

    // ── inline helpers (write into para_buf / cell_rich) ────────────────────

    /// Push text into whatever inline context is active
    fn push_inline_text(&mut self, text: &str) {
        if self.in_heading {
            self.heading_text.push_str(text);
        } else if self.in_image {
            self.image_alt.push_str(text);
        } else if self.in_link {
            self.link_text.push_str(text);
            // also push plain text for cell width measurement
            if self.in_table_cell {
                self.cur_cell_plain.push_str(text);
                self.cur_cell_rich.push_str(text);
            } else {
                // rich text goes to para_buf (ANSI already set before the link start)
                self.para_buf.push_str(text);
            }
        } else if self.in_table_cell {
            self.cur_cell_plain.push_str(text);
            self.cur_cell_rich.push_str(text);
        } else if self.code_buf.is_some() {
            if let Some(ref mut buf) = self.code_buf {
                buf.push_str(text);
            }
        } else {
            self.para_buf.push_str(text);
        }
    }

    /// Push ANSI into the rich side only (not plain/heading/code)
    fn push_ansi(&mut self, ansi: &str) {
        if self.in_table_cell {
            self.cur_cell_rich.push_str(ansi);
        } else if !self.in_heading && !self.in_image && self.code_buf.is_none() {
            self.para_buf.push_str(ansi);
        }
    }

    // ── main render ──────────────────────────────────────────────────────────

    pub fn render(&mut self, markdown: &str) -> Result<()> {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_FOOTNOTES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);

        let events: Vec<Event> = Parser::new_ext(markdown, opts).collect();

        for event in events {
            match event {
                // ── Headings ──────────────────────────────────────────────────
                Event::Start(Tag::Heading(level, _, _)) => {
                    self.flush_para()?;
                    self.heading_text.clear();
                    self.heading_level = level;
                    self.in_heading = true;
                }
                Event::End(Tag::Heading(_, _, _)) => {
                    self.in_heading = false;
                    let text = self.heading_text.clone();
                    self.print_heading(&text, self.heading_level)?;
                }

                // ── Paragraph ─────────────────────────────────────────────────
                Event::Start(Tag::Paragraph) => {}
                Event::End(Tag::Paragraph) => {
                    self.flush_para()?;
                    self.newline()?;
                }

                // ── Strong ────────────────────────────────────────────────────
                Event::Start(Tag::Strong) => {
                    self.strong_depth += 1;
                    let ansi = format!(
                        "{}{}",
                        Self::fg_str(&self.theme.colors.bold.clone()),
                        Self::bold_str()
                    );
                    self.push_ansi(&ansi);
                }
                Event::End(Tag::Strong) => {
                    self.strong_depth = self.strong_depth.saturating_sub(1);
                    if self.strong_depth == 0 && self.em_depth == 0 && self.strike_depth == 0 {
                        let ansi = format!(
                            "{}{}",
                            Self::reset_str(),
                            Self::fg_str(&self.theme.colors.foreground.clone())
                        );
                        self.push_ansi(&ansi);
                    }
                }

                // ── Emphasis ──────────────────────────────────────────────────
                Event::Start(Tag::Emphasis) => {
                    self.em_depth += 1;
                    let ansi = format!(
                        "{}{}",
                        Self::fg_str(&self.theme.colors.italic.clone()),
                        Self::italic_str()
                    );
                    self.push_ansi(&ansi);
                }
                Event::End(Tag::Emphasis) => {
                    self.em_depth = self.em_depth.saturating_sub(1);
                    if self.strong_depth == 0 && self.em_depth == 0 && self.strike_depth == 0 {
                        let ansi = format!(
                            "{}{}",
                            Self::reset_str(),
                            Self::fg_str(&self.theme.colors.foreground.clone())
                        );
                        self.push_ansi(&ansi);
                    }
                }

                // ── Strikethrough ─────────────────────────────────────────────
                Event::Start(Tag::Strikethrough) => {
                    self.strike_depth += 1;
                    let ansi = format!(
                        "{}{}",
                        Self::fg_str(&self.theme.colors.strikethrough.clone()),
                        Self::strike_str()
                    );
                    self.push_ansi(&ansi);
                }
                Event::End(Tag::Strikethrough) => {
                    self.strike_depth = self.strike_depth.saturating_sub(1);
                    if self.strong_depth == 0 && self.em_depth == 0 && self.strike_depth == 0 {
                        let ansi = format!(
                            "{}{}",
                            Self::reset_str(),
                            Self::fg_str(&self.theme.colors.foreground.clone())
                        );
                        self.push_ansi(&ansi);
                    }
                }

                // ── Inline code ───────────────────────────────────────────────
                Event::Code(text) => {
                    if self.in_heading {
                        // show without backticks in headings
                        self.heading_text.push_str(&text);
                    } else if self.in_table_cell {
                        // backtick-wrapped in cells, plain for width
                        let displayed = format!("`{}`", text);
                        self.cur_cell_plain.push_str(&displayed);
                        let ansi = format!(
                            "{}{}{}",
                            Self::fg_str(&self.theme.colors.code_inline.clone()),
                            displayed,
                            Self::reset_str()
                        );
                        self.cur_cell_rich.push_str(&ansi);
                    } else {
                        let displayed = format!("`{}`", text);
                        let ansi = format!(
                            "{}{}{}{}",
                            Self::fg_str(&self.theme.colors.code_inline.clone()),
                            displayed,
                            Self::reset_str(),
                            Self::fg_str(&self.theme.colors.foreground.clone())
                        );
                        self.para_buf.push_str(&ansi);
                    }
                }

                // ── Code blocks ───────────────────────────────────────────────
                Event::Start(Tag::CodeBlock(kind)) => {
                    self.flush_para()?;
                    self.code_buf = Some(String::new());
                    self.code_lang = match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
                        _ => None,
                    };
                }
                Event::End(Tag::CodeBlock(_)) => {
                    if let Some(code) = self.code_buf.take() {
                        let lang = self.code_lang.take();
                        // trim trailing newline that pulldown-cmark always appends
                        let code = code.trim_end_matches('\n').to_string();
                        self.render_code_block(&code, lang.as_deref())?;
                    }
                }

                // ── Links ─────────────────────────────────────────────────────
                Event::Start(Tag::Link(_, dest, _)) => {
                    self.in_link = true;
                    self.link_dest = dest.to_string();
                    self.link_text.clear();
                    let ansi = format!(
                        "{}{}",
                        Self::fg_str(&self.theme.colors.link.clone()),
                        Self::underline_str()
                    );
                    self.push_ansi(&ansi);
                }
                Event::End(Tag::Link(_, _, _)) => {
                    self.in_link = false;
                    // append URL in dimmer color
                    let url_part = format!(" ({})", self.link_dest);
                    let ansi = format!(
                        "{}{}{}{}",
                        Self::reset_str(),
                        Self::fg_str(&self.theme.colors.link_url.clone()),
                        url_part,
                        Self::reset_str()
                    );
                    if self.in_table_cell {
                        self.cur_cell_plain.push_str(&url_part);
                        self.cur_cell_rich.push_str(&ansi);
                    } else {
                        self.para_buf.push_str(&ansi);
                    }
                    self.link_dest.clear();
                    self.link_text.clear();
                }

                // ── Images ────────────────────────────────────────────────────
                Event::Start(Tag::Image(_, _, _)) => {
                    self.in_image = true;
                    self.image_alt.clear();
                }
                Event::End(Tag::Image(_, _dest, _)) => {
                    self.in_image = false;
                    // Show as [🖼 alt](url) — but skip URL-only images (alt == dest)
                    let alt = if self.image_alt.is_empty() {
                        "image".to_string()
                    } else {
                        self.image_alt.clone()
                    };
                    let displayed = format!("[🖼  {}]", alt);
                    let ansi = format!(
                        "{}{}{}",
                        Self::fg_str(&self.theme.colors.italic.clone()),
                        displayed,
                        Self::reset_str()
                    );
                    if self.in_table_cell {
                        self.cur_cell_plain.push_str(&displayed);
                        self.cur_cell_rich.push_str(&ansi);
                    } else {
                        self.para_buf.push_str(&ansi);
                    }
                    self.image_alt.clear();
                }

                // ── Lists ─────────────────────────────────────────────────────
                Event::Start(Tag::List(start)) => {
                    self.flush_para()?;
                    self.ordered_counters.push(start.map(|n| n));
                    self.list_depth += 1;
                }
                Event::End(Tag::List(_)) => {
                    self.list_depth = self.list_depth.saturating_sub(1);
                    self.ordered_counters.pop();
                    if self.list_depth == 0 {
                        self.newline()?;
                    }
                }
                Event::Start(Tag::Item) => {
                    self.flush_para()?;
                    let indent = " ".repeat(self.list_depth.saturating_sub(1) * self.indent);
                    let mc = self.theme.colors.list_marker.clone();
                    self.fg(&mc)?;
                    match self.ordered_counters.last_mut() {
                        Some(Some(n)) => {
                            write!(self.out, "{}{}. ", indent, n)?;
                            *n += 1;
                        }
                        _ => {
                            let bullet = match self.list_depth {
                                1 => "●",
                                2 => "○",
                                _ => "▸",
                            };
                            write!(self.out, "{}{} ", indent, bullet)?;
                        }
                    }
                    self.reset_color()?;
                }
                Event::End(Tag::Item) => {
                    self.flush_para()?;
                }

                // ── Blockquote ────────────────────────────────────────────────
                Event::Start(Tag::BlockQuote) => {
                    self.flush_para()?;
                    self.in_blockquote = true;
                    let ansi = format!(
                        "{}{}",
                        Self::fg_str(&self.theme.colors.blockquote.clone()),
                        Self::italic_str()
                    );
                    self.para_buf.push_str(&ansi);
                }
                Event::End(Tag::BlockQuote) => {
                    self.flush_para()?;
                    self.in_blockquote = false;
                    self.reset_attr()?;
                    self.reset_color()?;
                    self.newline()?;
                }

                // ── Tables ────────────────────────────────────────────────────
                Event::Start(Tag::Table(_)) => {
                    self.flush_para()?;
                    self.in_table = true;
                    self.table_rows.clear();
                    self.table_col_widths.clear();
                }
                Event::End(Tag::Table(_)) => {
                    self.in_table = false;
                    self.flush_table()?;
                }
                Event::Start(Tag::TableHead) => {
                    self.in_table_head = true;
                }
                Event::End(Tag::TableHead) => {
                    self.in_table_head = false;
                }
                Event::Start(Tag::TableRow) => {
                    self.cur_row_plain.clear();
                    self.cur_row_rich.clear();
                }
                Event::End(Tag::TableRow) => {
                    let plain = std::mem::take(&mut self.cur_row_plain);
                    let rich = std::mem::take(&mut self.cur_row_rich);
                    self.table_rows.push(TableRow {
                        plain,
                        rich,
                        is_header: self.in_table_head,
                    });
                }
                Event::Start(Tag::TableCell) => {
                    self.cur_cell_plain.clear();
                    self.cur_cell_rich.clear();
                    self.in_table_cell = true;
                }
                Event::End(Tag::TableCell) => {
                    self.in_table_cell = false;
                    // reset colors at end of cell
                    self.cur_cell_rich.push_str(&Self::reset_str());
                    let plain = std::mem::take(&mut self.cur_cell_plain);
                    let rich = std::mem::take(&mut self.cur_cell_rich);
                    self.cur_row_plain.push(plain);
                    self.cur_row_rich.push(rich);
                }

                // ── Horizontal rule ───────────────────────────────────────────
                Event::Rule => {
                    self.flush_para()?;
                    self.hr()?;
                    self.newline()?;
                }

                // ── Line breaks ───────────────────────────────────────────────
                Event::HardBreak => {
                    self.flush_para()?;
                }
                Event::SoftBreak => {
                    // treat as a space in the inline buffer
                    if self.in_table_cell {
                        self.cur_cell_plain.push(' ');
                        self.cur_cell_rich.push(' ');
                    } else if !self.in_heading && self.code_buf.is_none() {
                        self.para_buf.push(' ');
                    }
                }

                // ── Task list markers ─────────────────────────────────────────
                Event::TaskListMarker(checked) => {
                    let mark = if checked { "☑ " } else { "☐ " };
                    let mc = self.theme.colors.list_marker.clone();
                    self.fg(&mc)?;
                    write!(self.out, "{}", mark)?;
                    self.reset_color()?;
                }

                // ── Text ──────────────────────────────────────────────────────
                Event::Text(text) => {
                    self.push_inline_text(&text);
                }

                // ── HTML (block) ──────────────────────────────────────────────
                Event::Html(html) => {
                    // Strip tags; show plain content dimmed
                    let mut stripped = String::new();
                    let mut in_tag = false;
                    for c in html.chars() {
                        match c {
                            '<' => in_tag = true,
                            '>' => in_tag = false,
                            _ if !in_tag => stripped.push(c),
                            _ => {}
                        }
                    }
                    let stripped = stripped.trim();
                    if !stripped.is_empty() {
                        let ansi = format!(
                            "{}{}{}",
                            Self::fg_str(&self.theme.colors.code_comment.clone()),
                            stripped,
                            Self::reset_str()
                        );
                        self.para_buf.push_str(&ansi);
                    }
                }

                // ── Footnotes ─────────────────────────────────────────────────
                Event::FootnoteReference(name) => {
                    let ansi = format!(
                        "{}[^{}]{}",
                        Self::fg_str(&self.theme.colors.link_url.clone()),
                        name,
                        Self::reset_str()
                    );
                    self.para_buf.push_str(&ansi);
                }
                Event::Start(Tag::FootnoteDefinition(name)) => {
                    self.flush_para()?;
                    let ansi = format!(
                        "{}[^{}]: {}",
                        Self::fg_str(&self.theme.colors.link_url.clone()),
                        name,
                        Self::reset_str()
                    );
                    self.para_buf.push_str(&ansi);
                }
                Event::End(Tag::FootnoteDefinition(_)) => {
                    self.flush_para()?;
                    self.newline()?;
                }

                #[allow(unreachable_patterns)]
                _ => {}
            }
        }

        self.flush_para()?;
        self.reset_color()?;
        Ok(())
    }
}

/// Render markdown string to an ANSI-colored String
pub fn render_to_string(
    markdown: &str,
    theme: &Theme,
    line_numbers: bool,
    word_wrap: bool,
    indent: usize,
) -> Result<String> {
    let mut buf: Vec<u8> = Vec::new();
    let mut r = Renderer::new(&mut buf, theme.clone(), line_numbers, word_wrap, indent);
    r.render(markdown)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}
