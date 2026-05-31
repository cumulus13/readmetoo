use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ANSI color codes used by themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl AnsiColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub fn to_crossterm(&self) -> crossterm::style::Color {
        crossterm::style::Color::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

/// Role-based color scheme mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// Overall background (None = terminal default)
    pub background: Option<AnsiColor>,
    /// Default foreground text
    pub foreground: AnsiColor,
    /// H1 heading
    pub h1: AnsiColor,
    /// H2 heading
    pub h2: AnsiColor,
    /// H3 heading
    pub h3: AnsiColor,
    /// H4-H6 headings
    pub h4: AnsiColor,
    /// Bold text
    pub bold: AnsiColor,
    /// Italic text
    pub italic: AnsiColor,
    /// Inline code
    pub code_inline: AnsiColor,
    /// Code block background keyword
    pub code_keyword: AnsiColor,
    /// Code block string
    pub code_string: AnsiColor,
    /// Code block comment
    pub code_comment: AnsiColor,
    /// Code block number
    pub code_number: AnsiColor,
    /// Code block function/class names
    pub code_function: AnsiColor,
    /// Code block operators
    pub code_operator: AnsiColor,
    /// Blockquote text
    pub blockquote: AnsiColor,
    /// Link text
    pub link: AnsiColor,
    /// Link URL
    pub link_url: AnsiColor,
    /// Horizontal rule
    pub rule: AnsiColor,
    /// List bullet/marker
    pub list_marker: AnsiColor,
    /// Table header
    pub table_header: AnsiColor,
    /// Table border
    pub table_border: AnsiColor,
    /// Strikethrough text
    pub strikethrough: AnsiColor,
    /// Syntax theme name for syntect (for code blocks)
    pub syntect_theme: String,
}

/// A complete named theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub dark: bool,
    pub colors: ThemeColors,
}

/// Load a theme by name. Falls back to "fruity" if not found.
pub fn get_theme(name: &str) -> Option<Theme> {
    builtin_themes().remove(name)
}

/// List all available theme names
pub fn list_themes() -> Vec<String> {
    let mut names: Vec<String> = builtin_themes().into_keys().collect();
    names.sort();
    names
}

/// All builtin themes modeled after Pygments styles
pub fn builtin_themes() -> HashMap<String, Theme> {
    let mut map = HashMap::new();

    // ── fruity (default, dark) ────────────────────────────────────────────────
    // Exact Pygments fruity palette (pygments/styles/fruity.py):
    //   background  #111111 | foreground (Token)  #ffffff
    //   keyword     #fb660a | string              #0086d2
    //   comment     #008800 | number              #0086f7
    //   func/attr   #ff0086 | keyword_type        #cdcaa9
    //   preproc     #ff0007 | whitespace          #888888
    map.insert(
        "fruity".into(),
        Theme {
            name: "fruity".into(),
            description: "Exact Pygments fruity — dark vim-inspired theme (bg #111111)".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(17, 17, 17)), // #111111
                foreground: AnsiColor::new(255, 255, 255),    // #ffffff  Token
                h1: AnsiColor::new(251, 102, 10),             // #fb660a  Keyword
                h2: AnsiColor::new(0, 134, 210),              // #0086d2  String
                h3: AnsiColor::new(255, 0, 134),              // #ff0086  Name.Function
                h4: AnsiColor::new(0, 134, 247),              // #0086f7  Number
                bold: AnsiColor::new(255, 255, 255),          // #ffffff
                italic: AnsiColor::new(205, 202, 169),        // #cdcaa9  Keyword.Type
                code_inline: AnsiColor::new(0, 134, 210),     // #0086d2  String
                code_keyword: AnsiColor::new(251, 102, 10),   // #fb660a  Keyword bold
                code_string: AnsiColor::new(0, 134, 210),     // #0086d2  String
                code_comment: AnsiColor::new(0, 136, 0),      // #008800  Comment
                code_number: AnsiColor::new(0, 134, 247),     // #0086f7  Number bold
                code_function: AnsiColor::new(255, 0, 134),   // #ff0086  Name.Function bold
                code_operator: AnsiColor::new(205, 202, 169), // #cdcaa9  neutral
                blockquote: AnsiColor::new(0, 136, 0),        // #008800  Comment green
                link: AnsiColor::new(0, 134, 247),            // #0086f7  Number blue
                link_url: AnsiColor::new(0, 134, 210),        // #0086d2  String blue
                rule: AnsiColor::new(136, 136, 136),          // #888888  Whitespace
                list_marker: AnsiColor::new(251, 102, 10),    // #fb660a  Keyword orange
                table_header: AnsiColor::new(255, 0, 134),    // #ff0086  Name.Function pink
                table_border: AnsiColor::new(68, 68, 68),     // #444444  Generic.Output fg
                strikethrough: AnsiColor::new(136, 136, 136), // #888888  Whitespace
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── monokai (dark) ────────────────────────────────────────────────────────
    map.insert(
        "monokai".into(),
        Theme {
            name: "monokai".into(),
            description: "Classic Monokai dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(39, 40, 34)),
                foreground: AnsiColor::new(248, 248, 242),
                h1: AnsiColor::new(249, 38, 114),
                h2: AnsiColor::new(166, 226, 46),
                h3: AnsiColor::new(230, 219, 116),
                h4: AnsiColor::new(102, 217, 239),
                bold: AnsiColor::new(249, 38, 114),
                italic: AnsiColor::new(166, 226, 46),
                code_inline: AnsiColor::new(166, 226, 46),
                code_keyword: AnsiColor::new(249, 38, 114),
                code_string: AnsiColor::new(230, 219, 116),
                code_comment: AnsiColor::new(117, 113, 94),
                code_number: AnsiColor::new(174, 129, 255),
                code_function: AnsiColor::new(166, 226, 46),
                code_operator: AnsiColor::new(249, 38, 114),
                blockquote: AnsiColor::new(117, 113, 94),
                link: AnsiColor::new(102, 217, 239),
                link_url: AnsiColor::new(81, 170, 194),
                rule: AnsiColor::new(80, 80, 80),
                list_marker: AnsiColor::new(249, 38, 114),
                table_header: AnsiColor::new(166, 226, 46),
                table_border: AnsiColor::new(80, 80, 80),
                strikethrough: AnsiColor::new(117, 113, 94),
                syntect_theme: "Monokai Extended".into(),
            },
        },
    );

    // ── dracula (dark) ────────────────────────────────────────────────────────
    map.insert(
        "dracula".into(),
        Theme {
            name: "dracula".into(),
            description: "Dracula dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(40, 42, 54)),
                foreground: AnsiColor::new(248, 248, 242),
                h1: AnsiColor::new(255, 85, 85),
                h2: AnsiColor::new(255, 184, 108),
                h3: AnsiColor::new(241, 250, 140),
                h4: AnsiColor::new(80, 250, 123),
                bold: AnsiColor::new(255, 85, 85),
                italic: AnsiColor::new(189, 147, 249),
                code_inline: AnsiColor::new(80, 250, 123),
                code_keyword: AnsiColor::new(255, 121, 198),
                code_string: AnsiColor::new(241, 250, 140),
                code_comment: AnsiColor::new(98, 114, 164),
                code_number: AnsiColor::new(189, 147, 249),
                code_function: AnsiColor::new(80, 250, 123),
                code_operator: AnsiColor::new(255, 121, 198),
                blockquote: AnsiColor::new(98, 114, 164),
                link: AnsiColor::new(139, 233, 253),
                link_url: AnsiColor::new(98, 114, 164),
                rule: AnsiColor::new(68, 71, 90),
                list_marker: AnsiColor::new(255, 184, 108),
                table_header: AnsiColor::new(255, 121, 198),
                table_border: AnsiColor::new(68, 71, 90),
                strikethrough: AnsiColor::new(98, 114, 164),
                syntect_theme: "Dracula".into(),
            },
        },
    );

    // ── nord (dark) ───────────────────────────────────────────────────────────
    map.insert(
        "nord".into(),
        Theme {
            name: "nord".into(),
            description: "Nord arctic dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(46, 52, 64)),
                foreground: AnsiColor::new(216, 222, 233),
                h1: AnsiColor::new(191, 97, 106),
                h2: AnsiColor::new(235, 203, 139),
                h3: AnsiColor::new(163, 190, 140),
                h4: AnsiColor::new(129, 161, 193),
                bold: AnsiColor::new(235, 203, 139),
                italic: AnsiColor::new(180, 142, 173),
                code_inline: AnsiColor::new(163, 190, 140),
                code_keyword: AnsiColor::new(129, 161, 193),
                code_string: AnsiColor::new(163, 190, 140),
                code_comment: AnsiColor::new(76, 86, 106),
                code_number: AnsiColor::new(180, 142, 173),
                code_function: AnsiColor::new(136, 192, 208),
                code_operator: AnsiColor::new(129, 161, 193),
                blockquote: AnsiColor::new(76, 86, 106),
                link: AnsiColor::new(136, 192, 208),
                link_url: AnsiColor::new(94, 129, 172),
                rule: AnsiColor::new(67, 76, 94),
                list_marker: AnsiColor::new(235, 203, 139),
                table_header: AnsiColor::new(136, 192, 208),
                table_border: AnsiColor::new(67, 76, 94),
                strikethrough: AnsiColor::new(76, 86, 106),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── github-dark (dark) ────────────────────────────────────────────────────
    map.insert(
        "github-dark".into(),
        Theme {
            name: "github-dark".into(),
            description: "GitHub dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(13, 17, 23)),
                foreground: AnsiColor::new(201, 209, 217),
                h1: AnsiColor::new(88, 166, 255),
                h2: AnsiColor::new(121, 192, 255),
                h3: AnsiColor::new(149, 213, 145),
                h4: AnsiColor::new(210, 153, 34),
                bold: AnsiColor::new(255, 255, 255),
                italic: AnsiColor::new(201, 209, 217),
                code_inline: AnsiColor::new(255, 123, 114),
                code_keyword: AnsiColor::new(255, 123, 114),
                code_string: AnsiColor::new(165, 214, 255),
                code_comment: AnsiColor::new(139, 148, 158),
                code_number: AnsiColor::new(121, 192, 255),
                code_function: AnsiColor::new(210, 168, 255),
                code_operator: AnsiColor::new(255, 123, 114),
                blockquote: AnsiColor::new(139, 148, 158),
                link: AnsiColor::new(88, 166, 255),
                link_url: AnsiColor::new(58, 125, 200),
                rule: AnsiColor::new(33, 38, 45),
                list_marker: AnsiColor::new(88, 166, 255),
                table_header: AnsiColor::new(88, 166, 255),
                table_border: AnsiColor::new(33, 38, 45),
                strikethrough: AnsiColor::new(139, 148, 158),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── gruvbox-dark (dark) ───────────────────────────────────────────────────
    map.insert(
        "gruvbox-dark".into(),
        Theme {
            name: "gruvbox-dark".into(),
            description: "Gruvbox dark retro theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(40, 40, 40)),
                foreground: AnsiColor::new(235, 219, 178),
                h1: AnsiColor::new(251, 73, 52),
                h2: AnsiColor::new(250, 189, 47),
                h3: AnsiColor::new(184, 187, 38),
                h4: AnsiColor::new(142, 192, 124),
                bold: AnsiColor::new(251, 73, 52),
                italic: AnsiColor::new(211, 134, 155),
                code_inline: AnsiColor::new(184, 187, 38),
                code_keyword: AnsiColor::new(251, 73, 52),
                code_string: AnsiColor::new(184, 187, 38),
                code_comment: AnsiColor::new(146, 131, 116),
                code_number: AnsiColor::new(211, 134, 155),
                code_function: AnsiColor::new(250, 189, 47),
                code_operator: AnsiColor::new(254, 128, 25),
                blockquote: AnsiColor::new(146, 131, 116),
                link: AnsiColor::new(131, 165, 152),
                link_url: AnsiColor::new(69, 133, 136),
                rule: AnsiColor::new(80, 73, 69),
                list_marker: AnsiColor::new(250, 189, 47),
                table_header: AnsiColor::new(251, 73, 52),
                table_border: AnsiColor::new(80, 73, 69),
                strikethrough: AnsiColor::new(146, 131, 116),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── one-dark (dark) ───────────────────────────────────────────────────────
    map.insert(
        "one-dark".into(),
        Theme {
            name: "one-dark".into(),
            description: "Atom One Dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(40, 44, 52)),
                foreground: AnsiColor::new(171, 178, 191),
                h1: AnsiColor::new(224, 108, 117),
                h2: AnsiColor::new(209, 154, 102),
                h3: AnsiColor::new(229, 192, 123),
                h4: AnsiColor::new(152, 195, 121),
                bold: AnsiColor::new(224, 108, 117),
                italic: AnsiColor::new(198, 120, 221),
                code_inline: AnsiColor::new(152, 195, 121),
                code_keyword: AnsiColor::new(198, 120, 221),
                code_string: AnsiColor::new(152, 195, 121),
                code_comment: AnsiColor::new(92, 99, 112),
                code_number: AnsiColor::new(209, 154, 102),
                code_function: AnsiColor::new(97, 175, 239),
                code_operator: AnsiColor::new(86, 182, 194),
                blockquote: AnsiColor::new(92, 99, 112),
                link: AnsiColor::new(97, 175, 239),
                link_url: AnsiColor::new(86, 182, 194),
                rule: AnsiColor::new(60, 65, 75),
                list_marker: AnsiColor::new(209, 154, 102),
                table_header: AnsiColor::new(224, 108, 117),
                table_border: AnsiColor::new(60, 65, 75),
                strikethrough: AnsiColor::new(92, 99, 112),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── solarized-dark (dark) ─────────────────────────────────────────────────
    map.insert(
        "solarized-dark".into(),
        Theme {
            name: "solarized-dark".into(),
            description: "Solarized dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(0, 43, 54)),
                foreground: AnsiColor::new(131, 148, 150),
                h1: AnsiColor::new(220, 50, 47),
                h2: AnsiColor::new(203, 75, 22),
                h3: AnsiColor::new(181, 137, 0),
                h4: AnsiColor::new(133, 153, 0),
                bold: AnsiColor::new(147, 161, 161),
                italic: AnsiColor::new(108, 113, 196),
                code_inline: AnsiColor::new(133, 153, 0),
                code_keyword: AnsiColor::new(108, 113, 196),
                code_string: AnsiColor::new(42, 161, 152),
                code_comment: AnsiColor::new(88, 110, 117),
                code_number: AnsiColor::new(211, 54, 130),
                code_function: AnsiColor::new(38, 139, 210),
                code_operator: AnsiColor::new(220, 50, 47),
                blockquote: AnsiColor::new(88, 110, 117),
                link: AnsiColor::new(38, 139, 210),
                link_url: AnsiColor::new(7, 54, 66),
                rule: AnsiColor::new(7, 54, 66),
                list_marker: AnsiColor::new(203, 75, 22),
                table_header: AnsiColor::new(38, 139, 210),
                table_border: AnsiColor::new(7, 54, 66),
                strikethrough: AnsiColor::new(88, 110, 117),
                syntect_theme: "Solarized (dark)".into(),
            },
        },
    );

    // ── material (dark) ───────────────────────────────────────────────────────
    map.insert(
        "material".into(),
        Theme {
            name: "material".into(),
            description: "Material dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(38, 50, 56)),
                foreground: AnsiColor::new(238, 255, 255),
                h1: AnsiColor::new(255, 83, 112),
                h2: AnsiColor::new(255, 203, 107),
                h3: AnsiColor::new(195, 232, 141),
                h4: AnsiColor::new(137, 221, 255),
                bold: AnsiColor::new(255, 83, 112),
                italic: AnsiColor::new(199, 146, 234),
                code_inline: AnsiColor::new(195, 232, 141),
                code_keyword: AnsiColor::new(199, 146, 234),
                code_string: AnsiColor::new(195, 232, 141),
                code_comment: AnsiColor::new(85, 98, 112),
                code_number: AnsiColor::new(247, 140, 108),
                code_function: AnsiColor::new(130, 170, 255),
                code_operator: AnsiColor::new(137, 221, 255),
                blockquote: AnsiColor::new(85, 98, 112),
                link: AnsiColor::new(130, 170, 255),
                link_url: AnsiColor::new(85, 98, 112),
                rule: AnsiColor::new(55, 70, 80),
                list_marker: AnsiColor::new(255, 203, 107),
                table_header: AnsiColor::new(130, 170, 255),
                table_border: AnsiColor::new(55, 70, 80),
                strikethrough: AnsiColor::new(85, 98, 112),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── native (dark) ─────────────────────────────────────────────────────────
    map.insert(
        "native".into(),
        Theme {
            name: "native".into(),
            description: "Pygments native dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(26, 26, 26)),
                foreground: AnsiColor::new(220, 220, 220),
                h1: AnsiColor::new(255, 102, 102),
                h2: AnsiColor::new(255, 170, 68),
                h3: AnsiColor::new(255, 204, 102),
                h4: AnsiColor::new(170, 255, 102),
                bold: AnsiColor::new(255, 255, 255),
                italic: AnsiColor::new(187, 187, 187),
                code_inline: AnsiColor::new(102, 255, 102),
                code_keyword: AnsiColor::new(102, 102, 255),
                code_string: AnsiColor::new(255, 102, 102),
                code_comment: AnsiColor::new(153, 153, 153),
                code_number: AnsiColor::new(0, 204, 153),
                code_function: AnsiColor::new(255, 170, 68),
                code_operator: AnsiColor::new(170, 170, 170),
                blockquote: AnsiColor::new(153, 153, 153),
                link: AnsiColor::new(102, 153, 255),
                link_url: AnsiColor::new(68, 102, 204),
                rule: AnsiColor::new(68, 68, 68),
                list_marker: AnsiColor::new(255, 170, 68),
                table_header: AnsiColor::new(102, 102, 255),
                table_border: AnsiColor::new(68, 68, 68),
                strikethrough: AnsiColor::new(153, 153, 153),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── vim (dark) ────────────────────────────────────────────────────────────
    map.insert(
        "vim".into(),
        Theme {
            name: "vim".into(),
            description: "Classic Vim color scheme (dark)".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(0, 0, 0)),
                foreground: AnsiColor::new(255, 255, 255),
                h1: AnsiColor::new(0, 175, 255),
                h2: AnsiColor::new(0, 215, 0),
                h3: AnsiColor::new(255, 175, 0),
                h4: AnsiColor::new(215, 95, 255),
                bold: AnsiColor::new(255, 255, 255),
                italic: AnsiColor::new(200, 200, 200),
                code_inline: AnsiColor::new(0, 215, 0),
                code_keyword: AnsiColor::new(0, 175, 255),
                code_string: AnsiColor::new(255, 135, 0),
                code_comment: AnsiColor::new(95, 95, 95),
                code_number: AnsiColor::new(215, 95, 255),
                code_function: AnsiColor::new(0, 215, 0),
                code_operator: AnsiColor::new(215, 215, 215),
                blockquote: AnsiColor::new(95, 95, 95),
                link: AnsiColor::new(0, 175, 255),
                link_url: AnsiColor::new(0, 95, 175),
                rule: AnsiColor::new(95, 95, 95),
                list_marker: AnsiColor::new(255, 175, 0),
                table_header: AnsiColor::new(0, 175, 255),
                table_border: AnsiColor::new(95, 95, 95),
                strikethrough: AnsiColor::new(95, 95, 95),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── zenburn (dark) ────────────────────────────────────────────────────────
    map.insert(
        "zenburn".into(),
        Theme {
            name: "zenburn".into(),
            description: "Zenburn low-contrast dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(63, 63, 63)),
                foreground: AnsiColor::new(220, 220, 204),
                h1: AnsiColor::new(220, 130, 130),
                h2: AnsiColor::new(220, 200, 130),
                h3: AnsiColor::new(180, 220, 130),
                h4: AnsiColor::new(130, 220, 200),
                bold: AnsiColor::new(240, 240, 224),
                italic: AnsiColor::new(204, 170, 204),
                code_inline: AnsiColor::new(115, 175, 113),
                code_keyword: AnsiColor::new(223, 175, 143),
                code_string: AnsiColor::new(204, 136, 88),
                code_comment: AnsiColor::new(120, 120, 120),
                code_number: AnsiColor::new(255, 160, 122),
                code_function: AnsiColor::new(223, 175, 143),
                code_operator: AnsiColor::new(200, 200, 180),
                blockquote: AnsiColor::new(120, 120, 120),
                link: AnsiColor::new(130, 180, 220),
                link_url: AnsiColor::new(90, 130, 170),
                rule: AnsiColor::new(90, 90, 90),
                list_marker: AnsiColor::new(220, 200, 130),
                table_header: AnsiColor::new(223, 175, 143),
                table_border: AnsiColor::new(90, 90, 90),
                strikethrough: AnsiColor::new(120, 120, 120),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── inkpot (dark) ─────────────────────────────────────────────────────────
    map.insert(
        "inkpot".into(),
        Theme {
            name: "inkpot".into(),
            description: "InkPot dark theme".into(),
            dark: true,
            colors: ThemeColors {
                background: Some(AnsiColor::new(29, 24, 36)),
                foreground: AnsiColor::new(205, 205, 205),
                h1: AnsiColor::new(207, 106, 76),
                h2: AnsiColor::new(255, 164, 79),
                h3: AnsiColor::new(192, 192, 64),
                h4: AnsiColor::new(95, 175, 175),
                bold: AnsiColor::new(255, 255, 255),
                italic: AnsiColor::new(173, 138, 190),
                code_inline: AnsiColor::new(95, 175, 175),
                code_keyword: AnsiColor::new(173, 138, 190),
                code_string: AnsiColor::new(207, 106, 76),
                code_comment: AnsiColor::new(118, 118, 118),
                code_number: AnsiColor::new(102, 204, 153),
                code_function: AnsiColor::new(95, 175, 175),
                code_operator: AnsiColor::new(200, 200, 200),
                blockquote: AnsiColor::new(118, 118, 118),
                link: AnsiColor::new(95, 175, 255),
                link_url: AnsiColor::new(60, 100, 160),
                rule: AnsiColor::new(60, 55, 70),
                list_marker: AnsiColor::new(255, 164, 79),
                table_header: AnsiColor::new(173, 138, 190),
                table_border: AnsiColor::new(60, 55, 70),
                strikethrough: AnsiColor::new(118, 118, 118),
                syntect_theme: "base16-ocean.dark".into(),
            },
        },
    );

    // ── default / light ───────────────────────────────────────────────────────
    map.insert(
        "default".into(),
        Theme {
            name: "default".into(),
            description: "Pygments default light theme".into(),
            dark: false,
            colors: ThemeColors {
                background: None,
                foreground: AnsiColor::new(40, 40, 40),
                h1: AnsiColor::new(0, 0, 200),
                h2: AnsiColor::new(0, 100, 0),
                h3: AnsiColor::new(150, 0, 150),
                h4: AnsiColor::new(0, 130, 130),
                bold: AnsiColor::new(0, 0, 0),
                italic: AnsiColor::new(80, 80, 80),
                code_inline: AnsiColor::new(0, 100, 0),
                code_keyword: AnsiColor::new(0, 0, 200),
                code_string: AnsiColor::new(187, 68, 0),
                code_comment: AnsiColor::new(64, 128, 128),
                code_number: AnsiColor::new(0, 64, 255),
                code_function: AnsiColor::new(0, 100, 0),
                code_operator: AnsiColor::new(100, 100, 100),
                blockquote: AnsiColor::new(100, 100, 100),
                link: AnsiColor::new(0, 100, 200),
                link_url: AnsiColor::new(100, 130, 180),
                rule: AnsiColor::new(180, 180, 180),
                list_marker: AnsiColor::new(0, 0, 200),
                table_header: AnsiColor::new(0, 0, 180),
                table_border: AnsiColor::new(180, 180, 180),
                strikethrough: AnsiColor::new(150, 150, 150),
                syntect_theme: "InspiredGitHub".into(),
            },
        },
    );

    // ── emacs (light) ─────────────────────────────────────────────────────────
    map.insert(
        "emacs".into(),
        Theme {
            name: "emacs".into(),
            description: "Emacs-like light syntax theme".into(),
            dark: false,
            colors: ThemeColors {
                background: None,
                foreground: AnsiColor::new(0, 0, 0),
                h1: AnsiColor::new(0, 0, 205),
                h2: AnsiColor::new(34, 139, 34),
                h3: AnsiColor::new(139, 0, 139),
                h4: AnsiColor::new(0, 128, 128),
                bold: AnsiColor::new(0, 0, 0),
                italic: AnsiColor::new(80, 80, 80),
                code_inline: AnsiColor::new(34, 139, 34),
                code_keyword: AnsiColor::new(170, 34, 255),
                code_string: AnsiColor::new(188, 122, 0),
                code_comment: AnsiColor::new(95, 125, 149),
                code_number: AnsiColor::new(102, 102, 204),
                code_function: AnsiColor::new(0, 100, 50),
                code_operator: AnsiColor::new(100, 100, 100),
                blockquote: AnsiColor::new(100, 100, 100),
                link: AnsiColor::new(0, 0, 205),
                link_url: AnsiColor::new(100, 130, 180),
                rule: AnsiColor::new(180, 180, 180),
                list_marker: AnsiColor::new(170, 34, 255),
                table_header: AnsiColor::new(0, 0, 205),
                table_border: AnsiColor::new(180, 180, 180),
                strikethrough: AnsiColor::new(150, 150, 150),
                syntect_theme: "InspiredGitHub".into(),
            },
        },
    );

    // ── friendly (light) ──────────────────────────────────────────────────────
    map.insert(
        "friendly".into(),
        Theme {
            name: "friendly".into(),
            description: "Friendly light theme".into(),
            dark: false,
            colors: ThemeColors {
                background: None,
                foreground: AnsiColor::new(50, 50, 50),
                h1: AnsiColor::new(0, 100, 200),
                h2: AnsiColor::new(0, 160, 80),
                h3: AnsiColor::new(160, 40, 160),
                h4: AnsiColor::new(0, 140, 140),
                bold: AnsiColor::new(20, 20, 20),
                italic: AnsiColor::new(90, 90, 90),
                code_inline: AnsiColor::new(0, 140, 0),
                code_keyword: AnsiColor::new(0, 100, 200),
                code_string: AnsiColor::new(200, 80, 0),
                code_comment: AnsiColor::new(80, 140, 140),
                code_number: AnsiColor::new(100, 80, 200),
                code_function: AnsiColor::new(0, 140, 0),
                code_operator: AnsiColor::new(100, 100, 100),
                blockquote: AnsiColor::new(100, 100, 100),
                link: AnsiColor::new(0, 100, 200),
                link_url: AnsiColor::new(80, 120, 180),
                rule: AnsiColor::new(200, 200, 200),
                list_marker: AnsiColor::new(0, 100, 200),
                table_header: AnsiColor::new(0, 100, 180),
                table_border: AnsiColor::new(180, 180, 180),
                strikethrough: AnsiColor::new(150, 150, 150),
                syntect_theme: "InspiredGitHub".into(),
            },
        },
    );

    // ── solarized-light (light) ───────────────────────────────────────────────
    map.insert(
        "solarized-light".into(),
        Theme {
            name: "solarized-light".into(),
            description: "Solarized light theme".into(),
            dark: false,
            colors: ThemeColors {
                background: None,
                foreground: AnsiColor::new(88, 110, 117),
                h1: AnsiColor::new(220, 50, 47),
                h2: AnsiColor::new(203, 75, 22),
                h3: AnsiColor::new(181, 137, 0),
                h4: AnsiColor::new(133, 153, 0),
                bold: AnsiColor::new(101, 123, 131),
                italic: AnsiColor::new(108, 113, 196),
                code_inline: AnsiColor::new(133, 153, 0),
                code_keyword: AnsiColor::new(108, 113, 196),
                code_string: AnsiColor::new(42, 161, 152),
                code_comment: AnsiColor::new(147, 161, 161),
                code_number: AnsiColor::new(211, 54, 130),
                code_function: AnsiColor::new(38, 139, 210),
                code_operator: AnsiColor::new(220, 50, 47),
                blockquote: AnsiColor::new(147, 161, 161),
                link: AnsiColor::new(38, 139, 210),
                link_url: AnsiColor::new(147, 161, 161),
                rule: AnsiColor::new(200, 215, 220),
                list_marker: AnsiColor::new(203, 75, 22),
                table_header: AnsiColor::new(38, 139, 210),
                table_border: AnsiColor::new(200, 215, 220),
                strikethrough: AnsiColor::new(147, 161, 161),
                syntect_theme: "Solarized (light)".into(),
            },
        },
    );

    // ── tango (light) ─────────────────────────────────────────────────────────
    map.insert(
        "tango".into(),
        Theme {
            name: "tango".into(),
            description: "GNOME Tango light theme".into(),
            dark: false,
            colors: ThemeColors {
                background: None,
                foreground: AnsiColor::new(0, 0, 0),
                h1: AnsiColor::new(52, 101, 164),
                h2: AnsiColor::new(78, 154, 6),
                h3: AnsiColor::new(117, 80, 123),
                h4: AnsiColor::new(6, 152, 154),
                bold: AnsiColor::new(0, 0, 0),
                italic: AnsiColor::new(80, 80, 80),
                code_inline: AnsiColor::new(78, 154, 6),
                code_keyword: AnsiColor::new(0, 0, 200),
                code_string: AnsiColor::new(196, 160, 0),
                code_comment: AnsiColor::new(136, 138, 133),
                code_number: AnsiColor::new(36, 36, 175),
                code_function: AnsiColor::new(78, 154, 6),
                code_operator: AnsiColor::new(100, 100, 100),
                blockquote: AnsiColor::new(136, 138, 133),
                link: AnsiColor::new(52, 101, 164),
                link_url: AnsiColor::new(100, 130, 180),
                rule: AnsiColor::new(200, 200, 200),
                list_marker: AnsiColor::new(52, 101, 164),
                table_header: AnsiColor::new(52, 101, 164),
                table_border: AnsiColor::new(200, 200, 200),
                strikethrough: AnsiColor::new(150, 150, 150),
                syntect_theme: "InspiredGitHub".into(),
            },
        },
    );

    // ── autumn (light) ────────────────────────────────────────────────────────
    map.insert(
        "autumn".into(),
        Theme {
            name: "autumn".into(),
            description: "Warm autumn-toned light theme".into(),
            dark: false,
            colors: ThemeColors {
                background: None,
                foreground: AnsiColor::new(30, 30, 30),
                h1: AnsiColor::new(180, 60, 0),
                h2: AnsiColor::new(200, 120, 0),
                h3: AnsiColor::new(160, 140, 0),
                h4: AnsiColor::new(0, 130, 80),
                bold: AnsiColor::new(30, 30, 30),
                italic: AnsiColor::new(100, 80, 80),
                code_inline: AnsiColor::new(150, 80, 0),
                code_keyword: AnsiColor::new(0, 112, 32),
                code_string: AnsiColor::new(186, 33, 33),
                code_comment: AnsiColor::new(153, 85, 17),
                code_number: AnsiColor::new(64, 160, 112),
                code_function: AnsiColor::new(64, 0, 128),
                code_operator: AnsiColor::new(100, 100, 100),
                blockquote: AnsiColor::new(153, 85, 17),
                link: AnsiColor::new(0, 80, 180),
                link_url: AnsiColor::new(80, 110, 160),
                rule: AnsiColor::new(200, 200, 200),
                list_marker: AnsiColor::new(200, 120, 0),
                table_header: AnsiColor::new(180, 60, 0),
                table_border: AnsiColor::new(200, 200, 200),
                strikethrough: AnsiColor::new(150, 150, 150),
                syntect_theme: "InspiredGitHub".into(),
            },
        },
    );

    // ── bw (black & white) ────────────────────────────────────────────────────
    map.insert(
        "bw".into(),
        Theme {
            name: "bw".into(),
            description: "Black and white minimal theme".into(),
            dark: false,
            colors: ThemeColors {
                background: None,
                foreground: AnsiColor::new(0, 0, 0),
                h1: AnsiColor::new(0, 0, 0),
                h2: AnsiColor::new(50, 50, 50),
                h3: AnsiColor::new(80, 80, 80),
                h4: AnsiColor::new(100, 100, 100),
                bold: AnsiColor::new(0, 0, 0),
                italic: AnsiColor::new(70, 70, 70),
                code_inline: AnsiColor::new(50, 50, 50),
                code_keyword: AnsiColor::new(0, 0, 0),
                code_string: AnsiColor::new(50, 50, 50),
                code_comment: AnsiColor::new(130, 130, 130),
                code_number: AnsiColor::new(50, 50, 50),
                code_function: AnsiColor::new(0, 0, 0),
                code_operator: AnsiColor::new(70, 70, 70),
                blockquote: AnsiColor::new(100, 100, 100),
                link: AnsiColor::new(0, 0, 200),
                link_url: AnsiColor::new(80, 80, 180),
                rule: AnsiColor::new(180, 180, 180),
                list_marker: AnsiColor::new(0, 0, 0),
                table_header: AnsiColor::new(0, 0, 0),
                table_border: AnsiColor::new(150, 150, 150),
                strikethrough: AnsiColor::new(150, 150, 150),
                syntect_theme: "InspiredGitHub".into(),
            },
        },
    );

    map
}
