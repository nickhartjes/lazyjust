{ lib
, stdenv
, craneLib
, pkg-config
, makeWrapper
, libxcb
, libxkbcommon
, xorg
, wayland
, darwin
}:

let
  src = craneLib.cleanCargoSource ../.;

  commonArgs = {
    inherit src;
    strictDeps = true;

    nativeBuildInputs = [ pkg-config makeWrapper ];

    buildInputs =
      lib.optionals stdenv.isLinux [
        libxcb
        libxkbcommon
        wayland
        xorg.libX11
        xorg.libXcursor
      ]
      ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
        AppKit
        Cocoa
      ]);
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
{
  inherit src commonArgs cargoArtifacts;
}
