# lazyrust

A fast, themed TUI for the `just` command runner, written in Rust.

## Features

- Browse and run `just` recipes from any directory with a justfile
- Live session pane — watch recipe output as it streams
- Fuzzy filter to search recipes instantly
- Built-in theme picker with several bundled themes (Tokyo Night, Nord, Dracula, Catppuccin variants, Gruvbox, One Dark, Solarized Dark, Mono Amber)
- User-defined themes via simple TOML files
- Configurable icon styles: `round`, `ascii`, or `none`
- Walks subdirectories for nested justfiles
- Persistent session logs per run

## Install

```bash
cargo install lazyrust
```

Requires the [`just`](https://github.com/casey/just) binary on your `PATH`.

## Usage

```bash
lazyrust                  # scan current directory
lazyrust [path]           # scan a specific directory
lazyrust --justfile FILE  # use a specific justfile as root
```

## Configuration

Config file location:

| Platform | Default path |
|---|---|
| Linux | `$XDG_CONFIG_HOME/lazyrust/config.toml` or `~/.config/lazyrust/config.toml` |
| macOS | `~/Library/Application Support/lazyrust/config.toml` |

Override with `LAZYRUST_CONFIG_DIR=/path/to/dir`.

Generate a commented example config:

```bash
lazyrust config init   # writes config.toml if not present
lazyrust config path   # print the config file path
```

### Key settings

```toml
[ui]
theme = "tokyo-night"        # built-in or user theme name
icon_style = "round"         # "round" | "ascii" | "none"
split_ratio = 0.30           # fraction of width for the recipe list

[paths]
# state_dir = "/absolute/path"
# sessions_log_dir = "/absolute/path"

[logging]
session_log_size_cap_mb = 10
session_log_retention_days = 7

[engine]
render_throttle_ms = 16
tick_interval_ms = 250
```

## Themes

Built-in themes:

- `tokyo-night` (default)
- `nord`
- `dracula`
- `catppuccin-mocha`
- `catppuccin-macchiato`
- `catppuccin-frappe`
- `catppuccin-latte`
- `gruvbox-dark`
- `one-dark`
- `solarized-dark`
- `mono-amber`

Custom themes go in `<config_dir>/lazyrust/themes/<name>.toml`. A user theme with the same name as a built-in takes precedence.

Press `t` inside lazyrust to open the interactive theme picker.

## Keybindings

| Key | Action |
|---|---|
| `j` / `k` | Move cursor up/down |
| `/` | Fuzzy filter recipes |
| `Esc` | Clear filter |
| `Enter` | Run selected recipe (or focus its running session) |
| `Tab` | Toggle focus between list and session pane |
| `t` | Open theme picker |
| `F12` / `Ctrl+g` | Return focus to recipe list |
| `PgUp` / `PgDn` | Scroll session output |
| `Home` / `End` | Jump to top / bottom of scrollback |
| `e` | View startup load errors |
| `?` | Help |
| `q` | Quit |

## Screenshots

Coming soon.

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## Contributing

Issues and pull requests are welcome at [github.com/nickhartjes/lazyrust](https://github.com/nickhartjes/lazyrust).
