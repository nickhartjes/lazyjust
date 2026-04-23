# lazyjust

A terminal UI for [`just`](https://github.com/casey/just) — inspired by `lazygit`.

## Install

```bash
cargo install lazyjust
```

Requires the `just` binary on your `PATH`.

## Use

```bash
lazyjust [path]          # default: current directory
lazyjust --justfile FILE # override root justfile
```

## Keys

| Key | Action |
|---|---|
| `j` / `k` | Move cursor |
| `/` | Fuzzy filter |
| `Enter` | Run recipe (or focus its running session) |
| `Shift+Enter`, `r` | Always spawn a new run |
| `h` / `l` | Cycle run history |
| `d` | Switch justfile |
| `Tab` | List ↔ session focus |
| `K` | Kill focused session |
| `x` | Close focused session |
| `L` | Copy session log path |
| `>` `<` `=` | Resize panes |
| `F12` | Leave session pane |
| `PgUp` / `PgDn` | Scroll session output |
| `Home` / `End` | Top / bottom of scrollback |
| `e` | View startup load errors |
| `?` | Help |
| `q` | Quit |

## Spec

See `docs/superpowers/specs/2026-04-22-lazyjust-design.md`.
