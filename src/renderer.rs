use crate::theme::Theme;
use anyhow::Result;
use crossterm::style::{Attribute, ResetColor, SetAttribute, SetForegroundColor};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag};
use std::io::Write;
use terminal_size::{terminal_size, Width};

pub struct Renderer<W: Write> {
    out: W,
    theme: Theme,
    term_width: u16,
    line_numbers: bool,
    word_wrap: bool,
    indent: usize,
    list_depth: usize,
    ordered_counters: Vec<u64>,
    para_buf: String,
    in_blockquote: bool,
    in_table_head: bool,
    table_row: Vec<String>,
    table_col_widths: Vec<usize>,
    table_rows: Vec<Vec<String>>,
    table_is_header: Vec<bool>,
    code_buf: Option<String>,
    code_lang: Option<String>,
    heading_text: String,
    heading_level: HeadingLevel,
    in_heading: bool,
    in_cell: bool,
    cur_cell: String,
    link_dest: String,
    image_alt: String,
    in_image: bool,
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
            in_table_head: false,
            table_row: Vec::new(),
            table_col_widths: Vec::new(),
            table_rows: Vec::new(),
            table_is_header: Vec::new(),
            code_buf: None,
            code_lang: None,
            heading_text: String::new(),
            heading_level: HeadingLevel::H1,
            in_heading: false,
            in_cell: false,
            cur_cell: String::new(),
            link_dest: String::new(),
            image_alt: String::new(),
            in_image: false,
        }
    }

    fn set_fg(&mut self, c: &crate::theme::AnsiColor) -> Result<()> {
        write!(self.out, "{}", SetForegroundColor(c.to_crossterm()))?;
        Ok(())
    }
    fn reset(&mut self) -> Result<()> {
        write!(self.out, "{}", ResetColor)?;
        Ok(())
    }
    fn bold_on(&mut self) -> Result<()> {
        write!(self.out, "{}", SetAttribute(Attribute::Bold))?;
        Ok(())
    }
    fn italic_on(&mut self) -> Result<()> {
        write!(self.out, "{}", SetAttribute(Attribute::Italic))?;
        Ok(())
    }
    fn underline_on(&mut self) -> Result<()> {
        write!(self.out, "{}", SetAttribute(Attribute::Underlined))?;
        Ok(())
    }
    fn strike_on(&mut self) -> Result<()> {
        write!(self.out, "{}", SetAttribute(Attribute::CrossedOut))?;
        Ok(())
    }
    fn reset_attr(&mut self) -> Result<()> {
        write!(self.out, "{}", SetAttribute(Attribute::Reset))?;
        Ok(())
    }
    fn newline(&mut self) -> Result<()> {
        writeln!(self.out)?;
        Ok(())
    }

    fn hr(&mut self) -> Result<()> {
        let c = self.theme.colors.rule.clone();
        self.set_fg(&c)?;
        writeln!(self.out, "{}", "─".repeat(self.term_width as usize))?;
        self.reset()
    }

    fn print_heading(&mut self, text: &str, level: HeadingLevel) -> Result<()> {
        self.newline()?;
        let (prefix, color) = match level {
            HeadingLevel::H1 => ("█ ", self.theme.colors.h1.clone()),
            HeadingLevel::H2 => ("▌ ", self.theme.colors.h2.clone()),
            HeadingLevel::H3 => ("◆ ", self.theme.colors.h3.clone()),
            _ => ("◇ ", self.theme.colors.h4.clone()),
        };
        self.bold_on()?;
        self.set_fg(&color)?;
        write!(self.out, "{}{}", prefix, text)?;
        self.reset_attr()?;
        self.reset()?;
        self.newline()?;
        if matches!(level, HeadingLevel::H1 | HeadingLevel::H2) {
            let width =
                (prefix.chars().count() + text.chars().count()).min(self.term_width as usize);
            let under = if matches!(level, HeadingLevel::H1) {
                "═"
            } else {
                "─"
            };
            self.set_fg(&color)?;
            writeln!(self.out, "{}", under.repeat(width))?;
            self.reset()?;
        }
        Ok(())
    }

    fn flush_para(&mut self) -> Result<()> {
        if self.para_buf.is_empty() {
            return Ok(());
        }
        let text = self.para_buf.trim_end().to_string();
        self.para_buf.clear();
        let indent_str = if self.in_blockquote {
            "  │ ".to_string()
        } else {
            " ".repeat(self.list_depth * self.indent)
        };
        let avail = (self.term_width as usize).saturating_sub(indent_str.len());
        if self.word_wrap && avail > 20 {
            let mut col = 0usize;
            let mut first_word = true;
            for word in text.split_whitespace() {
                let wlen = word.chars().count();
                if col == 0 {
                    write!(self.out, "{}", indent_str)?;
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
                first_word = false;
            }
            if !first_word {
                writeln!(self.out)?;
            }
        } else {
            for line in text.lines() {
                writeln!(self.out, "{}{}", indent_str, line)?;
            }
        }
        Ok(())
    }

    fn render_code_block(&mut self, code: &str, lang: Option<&str>) -> Result<()> {
        let border_color = self.theme.colors.table_border.clone();
        let width = (self.term_width as usize).min(100);
        self.set_fg(&border_color)?;
        writeln!(self.out, "╭{}╮", "─".repeat(width - 2))?;
        if let Some(l) = lang {
            let label = format!(" {} ", l);
            let pad = (width - 2).saturating_sub(label.len());
            let kw = self.theme.colors.code_keyword.clone();
            write!(self.out, "│")?;
            self.set_fg(&kw)?;
            self.bold_on()?;
            write!(self.out, "{}", label)?;
            self.reset_attr()?;
            self.set_fg(&border_color)?;
            writeln!(self.out, "{}│", " ".repeat(pad))?;
            writeln!(self.out, "├{}┤", "─".repeat(width - 2))?;
        }
        self.reset()?;

        let cmt_color = self.theme.colors.code_comment.clone();
        let code_color = self.theme.colors.code_inline.clone();
        let comment_prefix: Option<&str> = match lang {
            Some("python") | Some("py") | Some("ruby") | Some("rb") | Some("shell")
            | Some("bash") | Some("sh") | Some("zsh") | Some("toml") | Some("yaml")
            | Some("yml") => Some("#"),
            Some("rust") | Some("rs") | Some("c") | Some("cpp") | Some("c++") | Some("java")
            | Some("go") | Some("swift") | Some("kotlin") | Some("javascript") | Some("js")
            | Some("typescript") | Some("ts") => Some("//"),
            Some("sql") => Some("--"),
            Some("css") | Some("html") | Some("xml") => Some("/*"),
            _ => Some("//"),
        };

        for (i, line) in code.lines().enumerate() {
            self.set_fg(&border_color)?;
            write!(self.out, "│")?;
            if self.line_numbers {
                self.set_fg(&cmt_color)?;
                write!(self.out, "{:>4} ", i + 1)?;
            } else {
                write!(self.out, " ")?;
            }
            let trimmed = line.trim_start();
            let is_comment = comment_prefix
                .map(|p| trimmed.starts_with(p))
                .unwrap_or(false);
            if is_comment {
                self.set_fg(&cmt_color)?;
            } else {
                self.set_fg(&code_color)?;
            }
            write!(self.out, "{}", line)?;

            let visible = line.chars().count() + if self.line_numbers { 5 } else { 1 };
            let pad = (width - 2).saturating_sub(visible);
            self.reset()?;
            self.set_fg(&border_color)?;
            writeln!(self.out, "{}│", " ".repeat(pad))?;
        }
        self.set_fg(&border_color)?;
        writeln!(self.out, "╰{}╯", "─".repeat(width - 2))?;
        self.reset()?;
        self.newline()?;
        Ok(())
    }

    fn flush_table(&mut self) -> Result<()> {
        if self.table_rows.is_empty() {
            return Ok(());
        }
        let cols = self.table_col_widths.len();
        let border = self.theme.colors.table_border.clone();
        let header = self.theme.colors.table_header.clone();
        let fg = self.theme.colors.foreground.clone();

        self.set_fg(&border)?;
        write!(self.out, "┌")?;
        for (i, w) in self.table_col_widths.iter().enumerate() {
            write!(self.out, "{}", "─".repeat(*w + 2))?;
            if i + 1 < cols {
                write!(self.out, "┬")?;
            }
        }
        writeln!(self.out, "┐")?;

        let rows = std::mem::take(&mut self.table_rows);
        let is_hdr = std::mem::take(&mut self.table_is_header);

        for (ri, row) in rows.iter().enumerate() {
            self.set_fg(&border)?;
            write!(self.out, "│")?;
            for (ci, cell) in row.iter().enumerate() {
                let w = self.table_col_widths.get(ci).copied().unwrap_or(6);
                write!(self.out, " ")?;
                if is_hdr[ri] {
                    self.bold_on()?;
                    self.set_fg(&header)?;
                } else {
                    self.set_fg(&fg)?;
                }
                write!(self.out, "{:<width$}", cell, width = w)?;
                self.reset_attr()?;
                self.set_fg(&border)?;
                write!(self.out, " │")?;
            }
            writeln!(self.out)?;
            if is_hdr[ri] {
                write!(self.out, "├")?;
                for (i, w) in self.table_col_widths.iter().enumerate() {
                    write!(self.out, "{}", "═".repeat(*w + 2))?;
                    if i + 1 < cols {
                        write!(self.out, "╪")?;
                    }
                }
                writeln!(self.out, "┤")?;
            }
        }
        write!(self.out, "└")?;
        for (i, w) in self.table_col_widths.iter().enumerate() {
            write!(self.out, "{}", "─".repeat(*w + 2))?;
            if i + 1 < cols {
                write!(self.out, "┴")?;
            }
        }
        writeln!(self.out, "┘")?;
        self.reset()?;
        self.newline()?;
        self.table_col_widths.clear();
        Ok(())
    }

    pub fn render(&mut self, markdown: &str) -> Result<()> {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_FOOTNOTES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);

        // First pass: collect table column widths
        {
            let mut in_table = false;
            let mut col_idx = 0usize;
            let mut widths: Vec<usize> = Vec::new();
            let mut cur = String::new();
            for ev in Parser::new_ext(markdown, opts) {
                match ev {
                    Event::Start(Tag::Table(_)) => {
                        in_table = true;
                        widths.clear();
                    }
                    Event::End(Tag::Table(_)) => {
                        in_table = false;
                        self.table_col_widths = widths.clone();
                    }
                    Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => {
                        col_idx = 0;
                    }
                    Event::Start(Tag::TableCell) => {
                        cur.clear();
                    }
                    Event::End(Tag::TableCell) if in_table => {
                        let w = cur.chars().count();
                        if col_idx < widths.len() {
                            widths[col_idx] = widths[col_idx].max(w);
                        } else {
                            widths.push(w.max(4));
                        }
                        col_idx += 1;
                    }
                    Event::Text(t) if in_table => {
                        cur.push_str(&t);
                    }
                    _ => {}
                }
            }
        }

        // Second pass: render
        let events: Vec<Event> = Parser::new_ext(markdown, opts).collect();
        for event in events {
            match event {
                // ── Headings ───────────────────────────────────────────────────────
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

                // ── Paragraph ──────────────────────────────────────────────────────
                Event::Start(Tag::Paragraph) => {}
                Event::End(Tag::Paragraph) => {
                    self.flush_para()?;
                    self.newline()?;
                }

                // ── Strong ─────────────────────────────────────────────────────────
                Event::Start(Tag::Strong) => {
                    if !self.in_heading && !self.in_cell {
                        let c = self.theme.colors.bold.clone();
                        self.set_fg(&c)?;
                        self.bold_on()?;
                    }
                }
                Event::End(Tag::Strong) => {
                    if !self.in_heading && !self.in_cell {
                        self.reset_attr()?;
                        self.reset()?;
                    }
                }

                // ── Emphasis ───────────────────────────────────────────────────────
                Event::Start(Tag::Emphasis) => {
                    if !self.in_heading && !self.in_cell {
                        let c = self.theme.colors.italic.clone();
                        self.set_fg(&c)?;
                        self.italic_on()?;
                    }
                }
                Event::End(Tag::Emphasis) => {
                    if !self.in_heading && !self.in_cell {
                        self.reset_attr()?;
                        self.reset()?;
                    }
                }

                // ── Strikethrough ──────────────────────────────────────────────────
                Event::Start(Tag::Strikethrough) => {
                    if !self.in_cell {
                        let c = self.theme.colors.strikethrough.clone();
                        self.set_fg(&c)?;
                        self.strike_on()?;
                    }
                }
                Event::End(Tag::Strikethrough) => {
                    if !self.in_cell {
                        self.reset_attr()?;
                        self.reset()?;
                    }
                }

                // ── Inline code ────────────────────────────────────────────────────
                Event::Code(text) => {
                    if self.in_cell {
                        self.cur_cell.push('`');
                        self.cur_cell.push_str(&text);
                        self.cur_cell.push('`');
                    } else if self.in_heading {
                        self.heading_text.push_str(&text);
                    } else {
                        let c = self.theme.colors.code_inline.clone();
                        self.set_fg(&c)?;
                        self.para_buf.push_str(&text);
                        self.reset()?;
                    }
                }

                // ── Code blocks ────────────────────────────────────────────────────
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
                        self.render_code_block(&code, lang.as_deref())?;
                    }
                }

                // ── Links ──────────────────────────────────────────────────────────
                Event::Start(Tag::Link(_, dest, _)) => {
                    self.link_dest = dest.to_string();
                    if !self.in_cell {
                        let c = self.theme.colors.link.clone();
                        self.set_fg(&c)?;
                        self.underline_on()?;
                    }
                }
                Event::End(Tag::Link(_, _, _)) => {
                    if !self.in_cell {
                        self.reset_attr()?;
                        let c = self.theme.colors.link_url.clone();
                        self.set_fg(&c)?;
                        let dest = format!(" ({})", self.link_dest);
                        self.para_buf.push_str(&dest);
                        self.reset()?;
                    }
                    self.link_dest.clear();
                }

                // ── Images ─────────────────────────────────────────────────────────
                Event::Start(Tag::Image(_, _, alt)) => {
                    self.in_image = true;
                    self.image_alt = alt.to_string();
                }
                Event::End(Tag::Image(_, _, _)) => {
                    self.in_image = false;
                    let c = self.theme.colors.italic.clone();
                    self.set_fg(&c)?;
                    let txt = format!("[🖼  {}]", self.image_alt);
                    self.para_buf.push_str(&txt);
                    self.reset()?;
                    self.image_alt.clear();
                }

                // ── Lists ──────────────────────────────────────────────────────────
                Event::Start(Tag::List(start)) => {
                    self.flush_para()?;
                    self.ordered_counters.push(start.unwrap_or(0));
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
                    let indent = " ".repeat((self.list_depth.saturating_sub(1)) * self.indent);
                    let mc = self.theme.colors.list_marker.clone();
                    self.set_fg(&mc)?;
                    let counter = self.ordered_counters.last_mut();
                    if let Some(n) = counter {
                        if *n > 0 {
                            write!(self.out, "{}{}. ", indent, n)?;
                            *n += 1;
                        } else {
                            let bullet = match self.list_depth {
                                1 => "●",
                                2 => "○",
                                _ => "▸",
                            };
                            write!(self.out, "{}{} ", indent, bullet)?;
                        }
                    }
                    self.reset()?;
                }
                Event::End(Tag::Item) => {
                    self.flush_para()?;
                }

                // ── Blockquote ─────────────────────────────────────────────────────
                Event::Start(Tag::BlockQuote) => {
                    self.flush_para()?;
                    self.in_blockquote = true;
                    let c = self.theme.colors.blockquote.clone();
                    self.set_fg(&c)?;
                    self.italic_on()?;
                }
                Event::End(Tag::BlockQuote) => {
                    self.flush_para()?;
                    self.in_blockquote = false;
                    self.reset_attr()?;
                    self.reset()?;
                    self.newline()?;
                }

                // ── Tables ─────────────────────────────────────────────────────────
                Event::Start(Tag::Table(_)) => {
                    self.flush_para()?;
                }
                Event::End(Tag::Table(_)) => {
                    self.flush_table()?;
                }
                Event::Start(Tag::TableHead) => {
                    self.in_table_head = true;
                }
                Event::End(Tag::TableHead) => {
                    self.in_table_head = false;
                }
                Event::Start(Tag::TableRow) => {
                    self.table_row.clear();
                }
                Event::End(Tag::TableRow) => {
                    let row = std::mem::take(&mut self.table_row);
                    self.table_rows.push(row);
                    self.table_is_header.push(self.in_table_head);
                }
                Event::Start(Tag::TableCell) => {
                    self.cur_cell.clear();
                    self.in_cell = true;
                }
                Event::End(Tag::TableCell) => {
                    self.in_cell = false;
                    let cell = self.cur_cell.clone();
                    self.table_row.push(cell);
                }

                // ── Horizontal rule ────────────────────────────────────────────────
                Event::Rule => {
                    self.flush_para()?;
                    self.hr()?;
                    self.newline()?;
                }

                // ── Line breaks ────────────────────────────────────────────────────
                Event::HardBreak => {
                    self.flush_para()?;
                }
                Event::SoftBreak => {
                    if !self.para_buf.is_empty() {
                        self.para_buf.push(' ');
                    }
                }

                // ── Task list ──────────────────────────────────────────────────────
                Event::TaskListMarker(checked) => {
                    let mark = if checked { "☑ " } else { "☐ " };
                    let c = self.theme.colors.list_marker.clone();
                    self.set_fg(&c)?;
                    write!(self.out, "{}", mark)?;
                    self.reset()?;
                }

                // ── Text ───────────────────────────────────────────────────────────
                Event::Text(text) => {
                    if self.in_heading {
                        self.heading_text.push_str(&text);
                    } else if let Some(ref mut buf) = self.code_buf {
                        buf.push_str(&text);
                    } else if self.in_cell {
                        self.cur_cell.push_str(&text);
                    } else if self.in_image {
                        // skip; alt already captured from Tag::Image
                    } else {
                        self.para_buf.push_str(&text);
                    }
                }

                Event::Html(html) => {
                    // Strip angle brackets, show stripped content dimmed
                    let plain: String = html.chars().filter(|&c| c != '<' && c != '>').collect();
                    let plain = plain.trim();
                    if !plain.is_empty() {
                        let c = self.theme.colors.code_comment.clone();
                        self.set_fg(&c)?;
                        self.para_buf.push_str(plain);
                        self.reset()?;
                    }
                }

                Event::FootnoteReference(name) => {
                    let c = self.theme.colors.link_url.clone();
                    self.set_fg(&c)?;
                    write!(self.out, "[^{}]", name)?;
                    self.reset()?;
                }

                Event::Start(Tag::FootnoteDefinition(name)) => {
                    self.flush_para()?;
                    let c = self.theme.colors.link_url.clone();
                    self.set_fg(&c)?;
                    write!(self.out, "[^{}]: ", name)?;
                    self.reset()?;
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
        self.reset()?;
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
