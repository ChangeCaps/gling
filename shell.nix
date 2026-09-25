{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    pkg-config
    gtk4
    gtk4-layer-shell
    librsvg
    alsa-lib
    dbus
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
