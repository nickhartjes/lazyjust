{ lib
, stdenv
, craneLib
, pkg-config
, makeWrapper
, libxcb
, libxkbcommon
, xorg
, wayland
}:

let
  src = craneLib.cleanCargoSource ../.;

  commonArgs = {
    inherit src;
    strictDeps = true;

    nativeBuildInputs = [ pkg-config makeWrapper ];

    buildInputs = lib.optionals stdenv.isLinux [
      libxcb
      libxkbcommon
      wayland
      xorg.libX11
      xorg.libXcursor
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
{
  inherit src commonArgs cargoArtifacts;
}
