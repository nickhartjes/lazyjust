# Discovery: always walk, with optional pin

**Date:** 2026-04-29
**Status:** Approved (design)
**Topic:** Make `lazyjust --justfile FILE` discover other justfiles in addition to FILE, and clean up the discovery module's split entry points.

## Problem

Today, `lazyjust --justfile FILE` skips the recursive walk and loads only
the pinned file. Users who pass `--justfile` expect to still see the rest
of their repo's justfiles in the dropdown switcher (`d`); the pin is meant
to anchor the *active* justfile, not to filter discovery.

The discovery module also has two near-identical public entry points
(`discover` / `discover_explicit`) that funnel into a private
`discover_inner` whose `Option<&Path>` second arg encodes "pinned vs
walked". The split has no callers that benefit from it — it's an artifact
of how the feature grew. One entry point is enough.

## Goal

A single discovery entry point that always walks, optionally pins a
specific file as the active one, and unions the walk roots when both a
positional `[PATH]` and `--justfile` are given.

## Behavior matrix

| Invocation | Walk roots |
|---|---|
| `lazyjust` | cwd |
| `lazyjust some/path` | `some/path` |
| `lazyjust --justfile F` | `parent_of(F)` |
| `lazyjust some/path --justfile F` | `some/path` ∪ `parent_of(F)` (dedup) |

Across all cases:

- The absolutized path of `F` (when `--justfile F` is set) is force-included
  in the discovered set, even if the walk skipped it because of `.gitignore`,
  the hardcoded skip list (`node_modules`, `target`, `dist`, `.git`), or
  because F sits outside any walked root.
- If `--justfile F` is set and F is in the result, F is the active
  justfile on launch. Otherwise the active justfile is the first sorted
  entry (current behavior).

## Non-goals

- Per-justfile root override.
- Async / progressive discovery, walk-progress UI.
- Changing `App::new`'s signature. The active index is post-assigned by
  the caller in `lib.rs`.
- `--justfile` accepting multiple files.
- Reworking the walker itself (`ignore::WalkBuilder`, hardcoded skip
  list, recognized filename patterns).

## Design

### CLI

`src/cli.rs`: drop the `default_value = "."` from `path` and change its
type to `Option<PathBuf>`. This lets the discovery layer tell apart "user
omitted PATH" from "user typed `.`". Both still walk cwd in practice when
nothing else is set, but the explicit/implicit distinction matters when
combined with `--justfile`.

```rust
#[arg(value_name = "PATH")]
pub path: Option<PathBuf>,
```

### Discovery API

`src/discovery/mod.rs` exposes a single public entry point:

```rust
pub struct DiscoverOptions<'a> {
    pub path: Option<&'a Path>,
    pub justfile: Option<&'a Path>,
}

pub struct DiscoveryResult {
    pub justfiles: Vec<Justfile>,
    pub errors: Vec<(PathBuf, String)>,
    /// Index into `justfiles` to pre-select. 0 when no pin or pin not found.
    pub active_index: usize,
}

pub fn discover(opts: DiscoverOptions) -> Result<DiscoveryResult>;
```

`discover_explicit` and the prior `discover(&Path)` are removed. The
private `discover_inner` is renamed/rewritten as the new `discover`.

Internal flow:

1. **Compute walk roots** — a small helper, e.g.
   `fn walk_roots(opts: &DiscoverOptions) -> Vec<PathBuf>`:
   - If `opts.path.is_some()` push it.
   - If `opts.justfile.is_some()` absolutize the file via
     `walk::absolutize`, take its parent, and push that. If the parent is
     somehow empty (e.g. the absolutized path is `/`), push `.` instead
     so the walker has a valid root.
   - Dedup the resulting list by absolutized path.
   - If the list is still empty (both `None`), return `vec![PathBuf::from(".")]`.
2. **Walk** each root via `walk::walk_justfiles`. Concatenate the results.
   Dedup paths by absolutized form.
3. **Pin merge** — if `opts.justfile` is set, push the absolutized pin
   into the set if it isn't already there.
4. **Sort** the path list (stable, alphabetical) — same convention as
   today.
5. **Parse** each path via `dump_and_parse`. Failures become entries in
   `errors`. Successes go into `justfiles`.
6. **Pin index** — if `opts.justfile` is set, find its absolutized path
   in `justfiles` and set `active_index` to that position. Otherwise 0.

### Caller

`src/lib.rs::async_main`:

```rust
let disc = discovery::discover(DiscoverOptions {
    path: cli.path.as_deref(),
    justfile: cli.justfile.as_deref(),
})?;
let mut app = app::App::new(disc.justfiles, disc.errors, ...);
app.active_justfile = disc.active_index;
```

The `JustNotFound` early-exit branch stays.

### File touched list

- `src/cli.rs` — `path` becomes `Option<PathBuf>`.
- `src/discovery/mod.rs` — collapse three entry points into one. Add
  `walk_roots` helper. Add `active_index` to `DiscoveryResult`. Add the
  pin-merge step.
- `src/lib.rs` — call new API, post-assign `active_justfile`.
- `README.md` — flip the `--justfile` line in the Usage section so it
  reads "load FILE and pin it; still walks for siblings".

No new files. No changes to `walk.rs`. No `App` constructor change.

## Testing

Discovery has unit + integration coverage today. Add:

- **Unit test** for `walk_roots` covering all four invocation rows in the
  behavior matrix.
- **Integration test** in `tests/discovery_integration.rs` (or the
  existing `discovery_walk_tests.rs`) exercising:
  - `--justfile F` only: result contains F's siblings.
  - `[PATH] P` + `--justfile F` (different parents): result is the union.
  - `--justfile F` where F is gitignored: F still appears in the result.
  - `active_index` points at F when `--justfile` is given.

The pre-existing `discover_explicit` test (if present) is rewritten to
target the new `discover(opts)` API.

## Risks

- Breaking change to the public API of `discovery::discover`. Acceptable:
  the only external caller is `lib.rs::async_main`. No semver guarantee on
  the internal module.
- Walk could now traverse two large trees if both `[PATH]` and
  `--justfile` are passed and they sit in different parts of the tree.
  Acceptable: discovery is one-shot at startup, the walker honors
  `.gitignore`, and the user explicitly asked for both. Document in the
  README that the walks union.
- If the user's `--justfile FILE` points outside their cwd's tree and the
  parent has no other justfiles, the result is just FILE — same as today.

## Out of scope follow-ups

- Multi-pin support (`--justfile F1 --justfile F2`).
- Discovery progress in the UI.
- Per-justfile environment overrides.
- Splitting README into `docs/usage.md`.
