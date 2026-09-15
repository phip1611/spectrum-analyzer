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

          # Dependencies needed for audio-visualizer (test and examples)
          libs = with pkgs; [
            alsa-lib
            libGL
            libx11
            libxcursor
            libxi
            libxkbcommon
            libxrandr
            wayland
          ];
        in
        pkgs.mkShell {
          packages =
            with pkgs;
            [
              pkg-config
            ]
            ++ libs;
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}";
        };
    };
}
