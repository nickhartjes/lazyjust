{ lib
, craneLib
, common
, just
}:

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
