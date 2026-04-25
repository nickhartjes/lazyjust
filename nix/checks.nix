{ craneLib, common }:

{
  cargoTest = craneLib.cargoTest (common.commonArgs // {
    inherit (common) cargoArtifacts;
  });

  cargoClippy = craneLib.cargoClippy (common.commonArgs // {
    inherit (common) cargoArtifacts;
    cargoClippyExtraArgs = "--all-targets -- -D warnings";
  });

  cargoFmt = craneLib.cargoFmt {
    inherit (common) src;
  };
}
