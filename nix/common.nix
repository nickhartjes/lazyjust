{ lib
, stdenv
, craneLib
, just
, pkg-config
, makeWrapper
, libxcb
, libxkbcommon
, libx11
, libxcursor
, wayland
}:

let
  # cleanCargoSource only keeps Rust-recognized files; the integration
  # tests under tests/fixtures/ need their justfiles + .gitignore at
  # build time, so we widen the filter to include anything under tests/.
  src = lib.cleanSourceWith {
    src = ../.;
    filter = path: type:
      (craneLib.filterCargoSources path type)
      || (builtins.match ".*/tests/.*" path != null);
  };

  commonArgs = {
    inherit src;
    strictDeps = true;

    nativeBuildInputs = [ pkg-config makeWrapper just ];

    buildInputs = lib.optionals stdenv.isLinux [
      libxcb
      libxkbcommon
      wayland
      libx11
      libxcursor
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
{
  inherit src commonArgs cargoArtifacts;
}
