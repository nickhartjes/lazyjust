# Nix Flake Packaging — Design

## Goal

Ship `lazyjust` as a Nix flake so NixOS / nix-darwin / home-manager users can
install, run, and develop the project with native Nix tooling. Keep the
derivation close to nixpkgs conventions so a future nixpkgs upstream PR can
reuse the same code with minimal change.

## Non-goals

- Submitting to `NixOS/nixpkgs` (separate follow-up project).
- Wiring `nix flake check` into existing GitHub Actions `ci.yml` (separate
  follow-up; flake must work locally first).
- Static / musl builds.
- Windows support (Nix doesn't target it).

## Decisions

| Question | Decision |
|---|---|
| Distribution scope | Flake first; nixpkgs PR is a later follow-up |
| Outputs | `packages`, `apps`, `devShells`, `checks`, `overlays` |
| Builder | `crane` (granular caching) |
| Platforms | `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, `aarch64-darwin` via `flake-utils.lib.eachDefaultSystem` |
| Runtime dep | Wrap binary with `just` on `PATH` via `makeWrapper` |
| Checks | `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` |
| Toolchain | `rust-overlay` pin to `1.78.0` (matches `Cargo.toml` `rust-version`) |
| nixpkgs channel | `nixos-unstable` |

## File layout

```
flake.nix              # inputs + outputs glue (~50 lines)
flake.lock             # committed
nix/
  common.nix           # shared src + commonArgs + cargoArtifacts
  package.nix          # crane buildPackage
  devshell.nix         # rust toolchain + tooling
  checks.nix           # test/clippy/fmt
.gitignore             # add `result`, `result-*`
README.md              # add Nix install/run section
```

Splitting into `nix/` keeps each file focused and easy to read. `flake.nix`
itself only handles inputs and the per-system fan-out.

## Inputs

```nix
inputs = {
  nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
  flake-utils.url  = "github:numtide/flake-utils";
  crane.url        = "github:ipetkov/crane";
  rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };
};
```

## Shared build args (`nix/common.nix`)

`commonArgs` and `cargoArtifacts` are reused by both the package and the
checks, so they live in their own file:

```nix
{ lib, stdenv, craneLib, pkg-config, makeWrapper
, libxcb, libxkbcommon, xorg, wayland, darwin }:

let
  src = craneLib.cleanCargoSource ../.;
  commonArgs = {
    inherit src;
    strictDeps = true;
    nativeBuildInputs = [ pkg-config makeWrapper ];
    buildInputs =
      lib.optionals stdenv.isLinux [
        libxcb libxkbcommon wayland xorg.libX11 xorg.libXcursor
      ]
      ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
        AppKit Cocoa
      ]);
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
{ inherit src commonArgs cargoArtifacts; }
```

## Package derivation (`nix/package.nix`)

```nix
{ lib, craneLib, common, just }:

craneLib.buildPackage (common.commonArgs // {
  inherit (common) cargoArtifacts;
  pname = "lazyjust";
  postInstall = ''
    wrapProgram $out/bin/lazyjust \
      --prefix PATH : ${lib.makeBinPath [ just ]}
  '';
  meta = with lib; {
    description = "Terminal UI for the just command runner";
    homepage = "https://github.com/nickhartjes/lazyjust";
    license = with licenses; [ mit asl20 ];
    mainProgram = "lazyjust";
    platforms = platforms.unix;
  };
})
```

Key points:

- `cleanCargoSource` excludes `target/`, dotfiles → cache-stable input hash.
- `buildDepsOnly` builds dependency closure once; reused by package and checks.
- Linux `buildInputs` cover `arboard` (X11 + Wayland clipboard backends);
  `crossterm` / `ratatui` need nothing extra.
- Darwin frameworks (`AppKit`, `Cocoa`) cover `arboard`'s macOS backend.
- `wrapProgram` puts `just` on the binary's `PATH` (decision Q4a).
- `meta.license` lists both `mit` and `asl20` because `Cargo.toml` is
  `MIT OR Apache-2.0`.

## DevShell (`nix/devshell.nix`)

```nix
{ lib, stdenv, mkShell, just, cargo-nextest, cargo-watch
, pkg-config, libxcb, libxkbcommon, xorg, wayland
, rustToolchain }:
mkShell {
  packages = [
    rustToolchain
    just
    cargo-nextest
    cargo-watch
    pkg-config
  ] ++ lib.optionals stdenv.isLinux [
    libxcb libxkbcommon wayland xorg.libX11
  ];
  RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
}
```

`rustToolchain` is the `rust-overlay` pin with `rust-src`, `clippy`, `rustfmt`
extensions; `RUST_SRC_PATH` lets rust-analyzer find the standard library
sources.

## Checks (`nix/checks.nix`)

```nix
{ craneLib, common }:
{
  cargoTest = craneLib.cargoTest (common.commonArgs // {
    inherit (common) cargoArtifacts;
  });
  cargoClippy = craneLib.cargoClippy (common.commonArgs // {
    inherit (common) cargoArtifacts;
    cargoClippyExtraArgs = "--all-targets -- -D warnings";
  });
  cargoFmt = craneLib.cargoFmt { inherit (common) src; };
}
```

All three reuse `cargoArtifacts` from `common`, so `nix flake check` incurs no
extra dependency rebuilds.

## flake.nix outputs glue

```nix
outputs = { self, nixpkgs, flake-utils, crane, rust-overlay }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      rustToolchain = pkgs.rust-bin.stable."1.78.0".default.override {
        extensions = [ "rust-src" "clippy" "rustfmt" ];
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      common   = pkgs.callPackage ./nix/common.nix  { inherit craneLib; };
      package  = pkgs.callPackage ./nix/package.nix { inherit craneLib common; };
      checks   = import ./nix/checks.nix             { inherit craneLib common; };
      devShell = pkgs.callPackage ./nix/devshell.nix { inherit rustToolchain; };
    in {
      packages.default  = package;
      packages.lazyjust = package;
      apps.default      = flake-utils.lib.mkApp { drv = package; };
      devShells.default = devShell;
      checks            = checks // { inherit package; };
    })
  // {
    overlays.default = final: prev:
      let
        craneLib = (crane.mkLib final).overrideToolchain
          (final.rust-bin.stable."1.78.0".default);
        common = final.callPackage ./nix/common.nix { inherit craneLib; };
      in {
        lazyjust = final.callPackage ./nix/package.nix {
          inherit craneLib common;
        };
      };
  };
```

`overlays.default` lives outside `eachDefaultSystem` because overlays are
system-agnostic; consumers apply them with their own `pkgs`.

## README addition

Slot under existing install section (next to Homebrew):

```markdown
### NixOS / Nix

Run without installing:

    nix run github:nickhartjes/lazyjust

Install to profile:

    nix profile install github:nickhartjes/lazyjust

NixOS / home-manager via overlay:

    # flake.nix
    inputs.lazyjust.url = "github:nickhartjes/lazyjust";

    # configuration.nix / home.nix
    nixpkgs.overlays = [ inputs.lazyjust.overlays.default ];
    environment.systemPackages = [ pkgs.lazyjust ];

Dev shell:

    nix develop
```

## Verification

Local checks before commit (run on `aarch64-darwin`):

```bash
nix flake check               # test + clippy + fmt + package build
nix build .#lazyjust          # produces ./result/bin/lazyjust
./result/bin/lazyjust --version
nix run .                     # smoke test
nix develop -c cargo build    # devshell sanity check
```

Other three platforms (`x86_64-darwin`, both Linux) verified by community / a
later GH Actions Nix job — out of scope for this design.

## Risks

- **Closure size on Linux**: `arboard` pulls X11 + Wayland libs (a few MB).
  Acceptable; users opting into Nix accept transitive closure costs.
- **Rust toolchain pin**: `rust-bin.stable."1.78.0"` must exist in
  `rust-overlay`. If not, fall back to `rust-bin.stable.latest.default`. Verify
  during implementation.
- **Wrapper assumptions**: wrapping puts `just` on `PATH`. The binary must not
  spawn other tools that would silently disappear under wrapping. Code review
  during implementation: does `lazyjust` invoke anything besides `just`? If
  yes, add to wrapper or handle separately.
- **`arboard` macOS frameworks**: `darwin.apple_sdk.frameworks.AppKit`/`Cocoa`
  may need `Foundation` too. Add if build fails.

## Out of scope (future work)

- Nixpkgs upstream PR (port `nix/package.nix` to `rustPlatform.buildRustPackage`
  in nixpkgs format).
- `nix flake check` step in `ci.yml`.
- Cachix binary cache.
- Home-manager module exposing config options.
