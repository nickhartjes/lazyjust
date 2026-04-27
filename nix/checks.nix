{ craneLib, common }:

{
  # Several tests across config::paths, config, and theme::registry mutate
  # process env vars and use module-local mutexes that don't cross-coordinate.
  # That's a pre-existing flake; serializing tests under Nix avoids it.
  cargoTest = craneLib.cargoTest (common.commonArgs // {
    inherit (common) cargoArtifacts;
    cargoTestExtraArgs = "-- --test-threads=1";
  });

  cargoClippy = craneLib.cargoClippy (common.commonArgs // {
    inherit (common) cargoArtifacts;
    cargoClippyExtraArgs = "--all-targets -- -D warnings";
  });

  cargoFmt = craneLib.cargoFmt {
    inherit (common) src;
  };
}
