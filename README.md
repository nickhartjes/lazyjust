# lazyjust

[![CI](https://github.com/nickhartjes/lazyjust/actions/workflows/ci.yml/badge.svg)](https://github.com/nickhartjes/lazyjust/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/lazyjust.svg)](https://crates.io/crates/lazyjust)
[![license](https://img.shields.io/crates/l/lazyjust.svg)](#license)
[![MSRV](https://img.shields.io/badge/rustc-1.79+-blue.svg)](#build-from-source)

A fast, themed TUI for the [`just`](https://github.com/casey/just) command runner, written in Rust.

![lazyjust dashboard](docs/screenshots/dashboard.png)

_Screenshot pending — add asset under `docs/screenshots/` and the image renders here._

## Why lazyjust

`just --list` dumps recipes to stdout and forgets. You memorize names, retype flags, re-run failing recipes from scratch. lazyjust gives every justfile a live dashboard: fuzzy-filter recipes, see their dependencies and command preview before running, watch output stream in a side pane, keep a scrollback per run, and kill or close sessions with one keystroke. The sidebar updates as you `j`/`k` through recipes, so you always see either a preview or the latest run of whatever's under the cursor.

## Features

- Browse + run any recipe from any directory containing a justfile.
- Live session pane — vt100-emulated output, scrollback, PID, elapsed time.
- Fuzzy filter to search recipes instantly.
- Cursor-follow: preview or latest-run log updates as you move through the list.
- Built-in theme picker, live preview, saved to config on confirm.
- 11 bundled themes + user themes via simple TOML.
- Glyph styles: `round`, `ascii`, `none`.
- Subdirectory discovery of nested justfiles; dropdown switcher.
- Persistent session logs per run, size cap + retention policy.

## Install

### Homebrew (macOS + Linux)

```bash
brew install nickhartjes/tap/lazyjust
```

### Pre-built binaries

Grab a tarball for your platform from [Releases](https://github.com/nickhartjes/lazyjust/releases). Checksums (`.sha256`) ship alongside every archive.

### NixOS / Nix

Requires Nix with flakes enabled.

Run without installing:

```sh
nix run github:nickhartjes/lazyjust
```

Install to your user profile:

```sh
nix profile install github:nickhartjes/lazyjust
```

Use as a flake input on NixOS / home-manager. The overlay relies on `rust-overlay` for the pinned Rust toolchain, so apply both overlays in order:

```nix
# flake.nix
{
  inputs = {
    lazyjust.url = "github:nickhartjes/lazyjust";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, lazyjust, rust-overlay, ... }: {
    # ...
  };
}

# configuration.nix or home.nix
{ pkgs, inputs, ... }:
{
  nixpkgs.overlays = [
    (import inputs.rust-overlay)
    inputs.lazyjust.overlays.default
  ];
  environment.systemPackages = [ pkgs.lazyjust ];
}
```

Enter a development shell with the pinned Rust toolchain and project tooling:

```sh
nix develop
```

### Cargo

```bash
cargo install lazyjust
```

### Build from source

Requires Rust **1.79+**.

```bash
git clone https://github.com/nickhartjes/lazyjust
cd lazyjust
cargo install --path .
```

Requires [`just`](https://github.com/casey/just) on `PATH` at runtime.

## Platform status

- **macOS** — supported.
- **Linux** — supported.
- **Windows** — not yet. The binary builds and `cargo test` passes, but spawning recipes returns an explicit "not yet implemented" error. Windows PTY support is tracked separately.

## Usage

```bash
lazyjust                  # scan current directory
lazyjust [path]           # scan a specific directory
lazyjust --justfile FILE  # pin a specific justfile as root
```

## Configuration

| Platform | Default path |
|---|---|
| Linux | `$XDG_CONFIG_HOME/lazyjust/config.toml` or `~/.config/lazyjust/config.toml` |
| macOS | `~/Library/Application Support/lazyjust/config.toml` (or `$XDG_CONFIG_HOME/lazyjust/` if set) |

Override with `LAZYJUST_CONFIG_DIR=/path/to/dir`.

```bash
lazyjust config init   # writes a commented config.toml if missing
lazyjust config path   # prints the resolved config file path
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

Built-in:

`tokyo-night` (default) · `nord` · `dracula` · `catppuccin-mocha` · `catppuccin-macchiato` · `catppuccin-frappe` · `catppuccin-latte` · `gruvbox-dark` · `one-dark` · `solarized-dark` · `mono-amber`

Custom themes go in `<config_dir>/lazyjust/themes/<name>.toml`. A user theme with the same name as a built-in takes precedence.

Press `t` inside lazyjust to open the interactive picker; Enter saves to config, Esc reverts.

## Keybindings

### List focus

| Key | Action |
|---|---|
| `j` / `k`, `↓` / `↑` | Move cursor |
| `h` / `l`, `←` / `→` | Cycle previous runs for the focused recipe |
| `Enter` | Run selected recipe, or jump to its running session |
| `Shift+Enter` / `r` | Always spawn a new session (never reuse) |
| `/` | Fuzzy-filter recipes |
| `Esc` | Clear filter |
| `d` | Open justfile-switcher dropdown |
| `Tab` | Toggle focus between list and right pane |
| `K` | Kill focused session (confirms) |
| `x` | Close focused session (confirms) |
| `L` | Copy focused session's log path |
| `Ctrl+o` / `Ctrl+i` | Jump to next / previous session with unread output |
| `>` / `<` / `=` | Grow / shrink / reset the left pane width |
| `t` | Open theme picker |
| `e` | Open startup-errors modal |
| `F1` / `?` | Help |
| `q` | Quit (confirms if sessions are running) |

### Session focus

| Key | Action |
|---|---|
| `F12` / `Ctrl+g` | Return focus to the recipe list |
| `PgUp` / `PgDn` | Scroll session output |
| `Home` / `End` | Jump to top / bottom of scrollback |
| any other key | Forwarded to the running shell |

## Acknowledgments

- [`just`](https://github.com/casey/just) — the command runner that makes this whole thing possible.
- [`ratatui`](https://github.com/ratatui-org/ratatui) — the terminal UI library.
- [`vt100`](https://github.com/doy/vt100-rust) — the terminal emulator used for the session pane.
- Inspired by [`lazygit`](https://github.com/jesseduffield/lazygit).

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## Contributing

Issues and pull requests welcome at [github.com/nickhartjes/lazyjust](https://github.com/nickhartjes/lazyjust).
