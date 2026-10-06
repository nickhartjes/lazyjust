# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.6] - 2026-10-06

Fixes a bug that left the session pane blank for anyone whose shell rc
runs a tool that probes the terminal — `fastfetch`, powerlevel10k's
instant prompt, or anything else drawing inline images.

### Fixed
- Recipes run again when the user's shell rc probes the terminal. The
  session PTY spawns `$SHELL -i`, so rc files run before the primed recipe
  line. Tools that draw inline images emit a DSR cursor-position query
  (`ESC [ 6 n`) and then block reading stdin for the report. Nothing ever
  replied — lazyjust had no DSR handling and `vt100` has none either — so
  the probe never returned and swallowed the primed recipe line while
  hunting for its answer. The recipe never ran, and the pane kept showing
  the screen the rc file had already cleared. lazyjust now answers the
  query with the cursor position at the point the query appeared ([#76]).

  Latent since `01fd4c9` (0.1.0) replaced the recipe-embedded wrapper with
  priming an interactive shell; it only started biting when `fastfetch`
  2.69.0 began probing.

### Changed
- Refreshed 54 lockfile packages, plus two manifest bumps `cargo update`
  cannot make on its own: `dirs` 6 → 7 and `rstest` 0.26 → 0.27 ([#76]).
  The `dirs` major is safe here — its only breaking change is
  `preference_dir` on Windows, which lazyjust does not call, and the three
  functions it does use resolve identically on macOS and Linux. No config
  or session-log migration. Supersedes [#56], [#67], [#68], [#69], [#70],
  [#71], [#72], [#73], [#74] and [#75].

### Internal
- Dev toolchain moved to Rust 1.99.0; `jdx/mise-action` to v5.1.1 and the
  SonarQube scan action to a newer digest ([#76]).
- Added a PTY regression test that spawns its own terminal-probing shell.
  The existing session integration test forces `SHELL=/bin/sh`, which runs
  no rc file, which is why CI never caught this ([#76]).

## [0.2.5] - 2026-08-23

Maintenance release: dependency refresh only. lazyjust's own production code
is byte-identical to 0.2.4 — no behaviour changes, no new features, no fixes
to lazyjust itself.

### Changed
- Refreshed every dependency to its latest compatible release ([#61]). The
  one with plausible reach into lazyjust is `ratatui` 0.30.0 → 0.30.2, whose
  buffer-diff fix for "uncovered" cells ([ratatui#2587]) touches the overlay
  path every modal here goes through; no lazyjust-side symptom was
  identified, so treat it as hygiene rather than a fix. Also `tokio`
  1.52.3 → 1.53.1, `ignore` 0.4.25 → 0.4.33, `clap` 4.6.1 → 4.6.6, plus
  `anyhow`, `futures`, `serde`, `serde_json`, `thiserror`, `toml`,
  `toml_edit` and ~100 transitive crates.

### Internal
- Serialized the `path_display` tests that mutate `HOME` behind a module
  mutex. They raced under default `cargo test` parallelism — reproducible at
  7 failures in 40 runs — and failed intermittently on Windows CI ([#61]).
- SonarQube now runs on a weekly schedule, and its bot guard is scoped to
  pull requests, so a broken scan can no longer go unnoticed while `main` is
  quiet ([#65]).
- Dev toolchain moved to Rust 1.98.0 and `just` 1.58.0 ([#61]).

## [0.2.4] - 2026-05-17

### Fixed
- Recipe list now scrolls to follow the cursor; previously the selected
  recipe could leave the visible viewport when navigating past the
  bottom of the pane (`d7b6dd4`).

## [0.2.3] - 2026-05-06

### Fixed
- Selecting a different justfile from the dropdown now refreshes the
  recipe list; previously the view kept showing the prior file's
  recipes in `ListMode::Active` (`55b0e8d`).

## [0.2.2] - 2026-04-30

### Added
- `Ctrl+c` quits lazyjust (with the same confirm-on-running-sessions
  prompt as `q`); when a session pane is focused, `Ctrl+c` is still
  forwarded to the child as SIGINT (`e0cf28a`).

## [0.2.1] - 2026-04-30

### Fixed
- Cluster recipes by group so headers don't repeat in the recipe list (`fc579e1`).

## [0.2.0] - 2026-04-29

### Added
- `list_mode` setting — merge recipes across justfiles ([#46]).
- Discovery always walks, with optional `--justfile` pin ([#44]).
- Onboarding first-run hint and Usage clarity ([#43]).

### Changed
- `just release` recipe + `RELEASE.md` + `AGENTS.md` ([#45]).
- Bump `sonarsource/sonarqube-scan-action` to v8 ([#42]).

### Fixed
- Test race condition ([#43]).

## [0.1.3] - 2026-04-28

### Added
- Shorten justfile path display in UI ([#40]).

### Changed
- Refactor: split high-complexity functions per SonarCloud findings ([#33]).
- SonarCloud cleanup: explicit `else` branches, method references, wildcard imports, tightened `release.yml` permissions ([#32], [#37]).
- Bump `sonarsource/sonarqube-scan-action` to v7.2 ([#28]).
- Bump `swatinem/rust-cache` digest ([#36]).

### Fixed
- CI: pin actions to commit SHAs, harden `curl` in release workflow, restrict `--proto` to https for SonarCloud S6506 ([#35], [#38]).
- CI(nix): cache `/nix/store` across runs, drop duplicate package build, swap installer to `cachix/install-nix-action` ([#34], [#39]).

## [0.1.2] - 2026-04-25

### Added
- SonarQube/SonarCloud scan workflow ([#26]).

### Fixed
- Honor `--justfile` and emit absolute discovery paths ([#29], [#30]).
- CI: install `rustfmt` and `clippy` explicitly after mise ([#31]).

### Changed
- Bump `jdx/mise-action` to v4 ([#25]).

## [0.1.1] - 2026-04-24

### Added
- Package `lazyjust` as a Nix flake ([#19]).
- Renovate config; switch from Dependabot to Renovate, pin mise versions ([#20], [#21]).

### Changed
- Standardize CI on mise; cancel superseded runs ([#24]).
- Bump `ratatui` 0.26.3 → 0.30.0 ([#5]).
- Bump `toml` 0.8.23 → 1.1.2 ([#11]).
- Bump `crossterm` 0.27.0 → 0.29.0 ([#4]).

## [0.1.0] - 2026-04-24

### Added
- Initial open-source release: M3 UI redesign and launch as `lazyjust` ([#1]).
- Theme picker modal with live preview hints; `t/j/k/Enter/Esc` bindings; `Mode::ThemePicker`.
- Config: honor `XDG_CONFIG_HOME` on all platforms; `toml_edit` writer preserves comments on `set_theme`.
- CI: tag-triggered release workflow with multi-platform builds; Homebrew formula bump ([#2]).
- CI: `color-gate` blocks hardcoded `Color::X` in `src/ui/` chrome.
- README: badges, why, install paths, platform status, full keybindings, acknowledgements ([#13]).
- Dependabot grouping for github-actions; renamed cargo group ([#14]).

### Fixed
- UI: drop forced bg on theme picker and dropdown highlight.
- Reducer: move `theme_picker_tests` to EOF for clippy + apply fmt.
- CI(release): create GitHub release before upload-assets jobs ([#17]); sha256 sidecar named `lazyjust-VER-TARGET.sha256` ([#18]); install `just` before tests ([#16]); render Homebrew formula for all 4 targets.

### Changed
- Bump `dirs` 6, `thiserror` 2, `portable-pty` 0.9, `vt100` 0.16, `toml_edit` 0.25, `rstest` 0.26 ([#15]).

[#56]: https://github.com/nickhartjes/lazyjust/pull/56
[#67]: https://github.com/nickhartjes/lazyjust/pull/67
[#68]: https://github.com/nickhartjes/lazyjust/pull/68
[#69]: https://github.com/nickhartjes/lazyjust/pull/69
[#70]: https://github.com/nickhartjes/lazyjust/pull/70
[#71]: https://github.com/nickhartjes/lazyjust/pull/71
[#72]: https://github.com/nickhartjes/lazyjust/pull/72
[#73]: https://github.com/nickhartjes/lazyjust/pull/73
[#74]: https://github.com/nickhartjes/lazyjust/pull/74
[#75]: https://github.com/nickhartjes/lazyjust/pull/75
[#76]: https://github.com/nickhartjes/lazyjust/pull/76
[#61]: https://github.com/nickhartjes/lazyjust/pull/61
[#65]: https://github.com/nickhartjes/lazyjust/pull/65
[ratatui#2587]: https://github.com/ratatui/ratatui/pull/2587
[0.2.6]: https://github.com/nickhartjes/lazyjust/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/nickhartjes/lazyjust/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/nickhartjes/lazyjust/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/nickhartjes/lazyjust/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/nickhartjes/lazyjust/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/nickhartjes/lazyjust/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/nickhartjes/lazyjust/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/nickhartjes/lazyjust/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/nickhartjes/lazyjust/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/nickhartjes/lazyjust/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nickhartjes/lazyjust/releases/tag/v0.1.0

[#1]: https://github.com/nickhartjes/lazyjust/pull/1
[#2]: https://github.com/nickhartjes/lazyjust/pull/2
[#4]: https://github.com/nickhartjes/lazyjust/pull/4
[#5]: https://github.com/nickhartjes/lazyjust/pull/5
[#11]: https://github.com/nickhartjes/lazyjust/pull/11
[#13]: https://github.com/nickhartjes/lazyjust/pull/13
[#14]: https://github.com/nickhartjes/lazyjust/pull/14
[#15]: https://github.com/nickhartjes/lazyjust/pull/15
[#16]: https://github.com/nickhartjes/lazyjust/pull/16
[#17]: https://github.com/nickhartjes/lazyjust/pull/17
[#18]: https://github.com/nickhartjes/lazyjust/pull/18
[#19]: https://github.com/nickhartjes/lazyjust/pull/19
[#20]: https://github.com/nickhartjes/lazyjust/pull/20
[#21]: https://github.com/nickhartjes/lazyjust/pull/21
[#24]: https://github.com/nickhartjes/lazyjust/pull/24
[#25]: https://github.com/nickhartjes/lazyjust/pull/25
[#26]: https://github.com/nickhartjes/lazyjust/pull/26
[#28]: https://github.com/nickhartjes/lazyjust/pull/28
[#29]: https://github.com/nickhartjes/lazyjust/pull/29
[#30]: https://github.com/nickhartjes/lazyjust/pull/30
[#31]: https://github.com/nickhartjes/lazyjust/pull/31
[#32]: https://github.com/nickhartjes/lazyjust/pull/32
[#33]: https://github.com/nickhartjes/lazyjust/pull/33
[#34]: https://github.com/nickhartjes/lazyjust/pull/34
[#35]: https://github.com/nickhartjes/lazyjust/pull/35
[#36]: https://github.com/nickhartjes/lazyjust/pull/36
[#37]: https://github.com/nickhartjes/lazyjust/pull/37
[#38]: https://github.com/nickhartjes/lazyjust/pull/38
[#39]: https://github.com/nickhartjes/lazyjust/pull/39
[#40]: https://github.com/nickhartjes/lazyjust/pull/40
[#42]: https://github.com/nickhartjes/lazyjust/pull/42
[#43]: https://github.com/nickhartjes/lazyjust/pull/43
[#44]: https://github.com/nickhartjes/lazyjust/pull/44
[#45]: https://github.com/nickhartjes/lazyjust/pull/45
[#46]: https://github.com/nickhartjes/lazyjust/pull/46
