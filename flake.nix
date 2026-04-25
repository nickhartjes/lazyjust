{
  description = "Terminal UI for the just command runner";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url       = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

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

        common   = pkgs.callPackage ./nix/common.nix   { inherit craneLib; };
        package  = pkgs.callPackage ./nix/package.nix  { inherit craneLib common; };
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
            final.rust-bin.stable."1.78.0".default;
          common = final.callPackage ./nix/common.nix { inherit craneLib; };
        in {
          lazyjust = final.callPackage ./nix/package.nix {
            inherit craneLib common;
          };
        };
    };
}
