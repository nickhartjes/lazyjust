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
  src = craneLib.cleanCargoSource ../.;

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
