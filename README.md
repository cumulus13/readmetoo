# readmetoo 📖

[![Build](https://github.com/cumulus13/readmetoo/actions/workflows/build.yml/badge.svg)](https://github.com/cumulus13/readmetoo/actions/workflows/build.yml)
[![Crates.io](https://img.shields.io/crates/v/readmetoo.svg)](https://crates.io/crates/readmetoo)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> Beautiful, themed Markdown reader for your terminal.

`readmetoo` renders Markdown files directly in your terminal with rich formatting, syntax-highlighted code blocks, Unicode table rendering, and **30+ Pygments-inspired color themes**.

---

## Features

- 🎨 **30+ built-in themes** — all Pygments classics: `fruity` (default), `monokai`, `dracula`, `nord`, `github-dark`, `gruvbox-dark`, `one-dark`, `solarized-dark`, `material`, `zenburn`, and many more
- 🌙 **Dark & light themes** — works great on any terminal background
- 📟 **Built-in pager** — scroll long documents with keyboard navigation
- 🗂 **Custom theme files** — drop your own `.theme.toml` in `~/.config/readmetoo/themes/`
- ⚙️ **Config file support** — via [`config-get`](https://github.com/cumulus13/config-get-rs), auto-discovered from OS-standard locations
- 🧩 **Syntax highlighting** — fenced code blocks with language labels and line numbers
- 📊 **Tables** — full Unicode box-drawing table rendering
- 🔗 **Links, images, footnotes, task lists** — all CommonMark + GFM extensions
- 📦 **Single binary** — statically linked musl build, zero runtime dependencies
- 🖥️ **Cross-platform** — Linux, macOS, Windows

---

## Installation

### From crates.io

```bash
cargo install readmetoo
```

### From GitHub Releases

Download the pre-built binary for your platform from the [Releases](https://github.com/cumulus13/readmetoo/releases) page.

### From source

```bash
git clone https://github.com/cumulus13/readmetoo
cd readmetoo
cargo build --release
sudo install -m755 target/release/readmetoo /usr/local/bin/
```

### Bin name

```bash
readme
```

or

```bash
readmetoo
```

---

## Usage

```
readmetoo [OPTIONS] [FILE]...
```

### Examples

```bash
# Read a file with the default theme (fruity)
readmetoo README.md

# Use a different theme
readmetoo --theme dracula README.md

# Enable pager for long files
readmetoo --pager --theme nord CHANGELOG.md

# Show with line numbers in code blocks
readmetoo --line-numbers src/main.rs.md

# Read from stdin
cat README.md | readmetoo -

# List all available themes
readmetoo --list-themes

# Preview a theme with sample content
readmetoo --preview-theme monokai

# Print default config to stdout (save it to configure)
readmetoo --print-config > ~/.config/readmetoo/readmetoo.toml

# Show config file location
readmetoo --config-path
```

---

## Themes

Run `readmetoo --list-themes` to see all themes:

| Theme | Style | Based on |
|-------|-------|----------|
| `fruity` ⭐ | dark | Pygments fruity |
| `monokai` | dark | Monokai |
| `dracula` | dark | Dracula |
| `nord` | dark | Nord |
| `github-dark` | dark | GitHub Dark |
| `gruvbox-dark` | dark | Gruvbox |
| `one-dark` | dark | Atom One Dark |
| `solarized-dark` | dark | Solarized |
| `material` | dark | Material |
| `native` | dark | Pygments native |
| `vim` | dark | Vim default |
| `zenburn` | dark | Zenburn |
| `inkpot` | dark | InkPot |
| `default` | light | Pygments default |
| `emacs` | light | Emacs |
| `friendly` | light | Pygments friendly |
| `solarized-light` | light | Solarized |
| `tango` | light | GNOME Tango |
| `autumn` | light | Pygments autumn |
| `bw` | light | Black & white |

Preview any theme:

```bash
readmetoo --preview-theme gruvbox-dark
```

### Custom Themes

Create a TOML file in `~/.config/readmetoo/themes/mytheme.theme.toml`:

```toml
name = "mytheme"
description = "My custom theme"
dark = true

[colors]
foreground = { r = 220, g = 220, b = 204 }
h1 = { r = 255, g = 100, b = 100 }
h2 = { r = 255, g = 160, b = 60 }
# ... etc.
```

---

## Configuration

`readmetoo` uses [`config-get`](https://github.com/cumulus13/config-get-rs) to auto-discover config files from standard OS locations:

| Platform | Locations searched |
|----------|--------------------|
| Linux | `$XDG_CONFIG_HOME/readmetoo/readmetoo.{toml,ini,yaml,json,.env}`, `/etc/readmetoo/` |
| macOS | `$HOME/Library/Application Support/readmetoo/`, `/etc/readmetoo/` |
| Windows | `%APPDATA%\readmetoo\readmetoo.{toml,ini,yaml,json}` |

Generate a documented default config:

```bash
readmetoo --print-config > "$(readmetoo --config-path)"
```

### Config options

```toml
# Theme name (run `readmetoo --list-themes` for options)
theme = "fruity"

# Enable pager by default
pager = false

# Custom pager command
# pager_cmd = "less -R"

# Terminal width override (0 = auto)
# width = 0

# Line numbers in code blocks
line_numbers = false

# Custom themes directory
# themes_dir = "~/.config/readmetoo/themes"

# Show filename header when reading multiple files
show_header = true

# Word-wrap paragraphs to terminal width
word_wrap = true

# List indent (spaces)
indent = 2
```

Environment variables override config file settings:

| Variable | Effect |
|----------|--------|
| `READMETOO_THEME` | Set theme |
| `PAGER` | External pager command |

---

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)

---

## License

MIT © [Hadi Cahyadi](mailto:cumulus13@gmail.com) — <https://github.com/cumulus13/readmetoo>
