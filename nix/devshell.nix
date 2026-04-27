{ lib
, stdenv
, mkShell
, just
, cargo-nextest
, cargo-watch
, pkg-config
, libxcb
, libxkbcommon
, libx11
, wayland
, rustToolchain
}:

mkShell {
  packages = [
    rustToolchain
    just
    cargo-nextest
    cargo-watch
    pkg-config
  ] ++ lib.optionals stdenv.isLinux [
    libxcb
    libxkbcommon
    wayland
    libx11
  ];

  RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
}
