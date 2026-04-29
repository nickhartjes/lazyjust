# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
