{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell rec {
    nativeBuildInputs = with pkgs; [
      pkg-config
    ];

    buildInputs = with pkgs; [
      alsa-lib
      fontconfig
      libxkbcommon
      xorg.libXcursor
      xorg.libX11
    ];

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
}
