default: check

check:
    cargo check --all-targets

build:
    cargo build

test:
    cargo test --all-targets

fmt:
    cargo fmt --all

lint:
    cargo clippy --all-targets -- -D warnings

# Fail if any hardcoded ratatui Color::X sneaks into src/ui/ chrome.
# session_pane.rs is allowed — it translates vt100 palette indices, not chrome.
color-gate:
    @ if rg --glob '!src/ui/session_pane.rs' -q 'Color::(Red|Green|Blue|Yellow|Cyan|Magenta|White|Black|LightRed|LightGreen|LightBlue|LightYellow|LightCyan|LightMagenta|LightGray|DarkGray|Gray|Rgb|Indexed)' src/ui/; then \
        echo "color-gate: hardcoded Color::X found in src/ui/ chrome — theme slots only"; \
        rg --glob '!src/ui/session_pane.rs' -n 'Color::(Red|Green|Blue|Yellow|Cyan|Magenta|White|Black|LightRed|LightGreen|LightBlue|LightYellow|LightCyan|LightMagenta|LightGray|DarkGray|Gray|Rgb|Indexed)' src/ui/; \
        exit 1; \
    fi

ci: fmt lint test color-gate