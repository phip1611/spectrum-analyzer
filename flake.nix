{
  description = "spectrum-analyzer Rust crate";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs }@inputs:
    {
      devShells.x86_64-linux.default =
        let
          pkgs = inputs.nixpkgs.legacyPackages.x86_64-linux;

          # Mainly runtime deps of the examples, not the base lib.
          runtimeDeps = with pkgs; [
            alsa-lib
            fontconfig
            libxkbcommon
            xorg.libXcursor
            xorg.libX11
          ];
        in
        pkgs.mkShell {
          packages =
            with pkgs;
            [
              pkg-config
            ]
            ++ runtimeDeps;
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath runtimeDeps}";
        };
    };
}
