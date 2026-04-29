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

# Cut a release: bump Cargo.toml, refresh Cargo.lock, build a changelog
# from `git log <last-tag>..HEAD --oneline`, commit as `release: vVERSION`,
# and tag `vVERSION`. Does NOT push — push manually with
# `git push origin main --follow-tags`.
release VERSION:
    #!/usr/bin/env bash
    set -euo pipefail

    cleanup_on_fail() {
        local rc=$?
        if [[ $rc -ne 0 ]]; then
            echo "release aborted (exit $rc)." >&2
            echo "If Cargo.toml/Cargo.lock were already mutated, restore with:" >&2
            echo "  git checkout -- Cargo.toml Cargo.lock" >&2
        fi
    }
    trap cleanup_on_fail EXIT

    # 1. Validate VERSION shape.
    if ! [[ "{{VERSION}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "release: VERSION must be MAJOR.MINOR.PATCH (got: {{VERSION}})" >&2
        exit 1
    fi

    # 2. Verify clean main, up-to-date with origin.
    branch="$(git rev-parse --abbrev-ref HEAD)"
    if [[ "$branch" != "main" ]]; then
        echo "release: must run on main (currently on: $branch)" >&2
        exit 1
    fi
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "release: working tree is dirty; commit or stash first" >&2
        git status --short
        exit 1
    fi
    git fetch --quiet origin main
    if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
        echo "release: HEAD is not equal to origin/main; pull/push first" >&2
        echo "  local:  $(git rev-parse HEAD)" >&2
        echo "  origin: $(git rev-parse origin/main)" >&2
        exit 1
    fi

    # 3. Preflight: read-only checks (do not auto-rewrite the tree).
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test --all-targets
    just color-gate

    # 4. Bump Cargo.toml [package] version. perl -i -pe is identical on
    #    macOS and Linux (no BSD/GNU `sed -i` split).
    perl -i -pe 's/^version = "[0-9]+\.[0-9]+\.[0-9]+"$/version = "{{VERSION}}"/' Cargo.toml
    if ! grep -q "^version = \"{{VERSION}}\"$" Cargo.toml; then
        echo "release: failed to bump Cargo.toml version" >&2
        exit 1
    fi

    # 5. Refresh Cargo.lock by running a build.
    cargo build --quiet

    # 6. Build the changelog body from `git log <last-tag>..HEAD`.
    last_tag="$(git describe --tags --abbrev=0 2>/dev/null || true)"
    if [[ -n "$last_tag" ]]; then
        body="$(git log "$last_tag..HEAD" --oneline | sed 's/^/- /')"
        if [[ -z "$body" ]]; then
            body="No commits since $last_tag (re-tag of an existing commit?)."
        fi
    else
        body="Initial release."
    fi

    # 7. Commit.
    git add Cargo.toml Cargo.lock
    git commit -m "release: v{{VERSION}}" -m "$body"

    # 8. Tag.
    git tag "v{{VERSION}}"

    # 9. Tell the human how to push.
    echo
    echo "Done. Commit and tag created locally."
    echo "Push with: git push origin main --follow-tags"