{ pkgs ? import <nixpkgs> {} }:
with pkgs; mkShell rec {
  packages = with xorg; [
    llvmPackages.libcxxClang opencv libGL
    wayland libX11 libXcursor
    libXi libXrandr libxkbcommon
  ];
  LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${lib.makeLibraryPath packages}";
  LIBCLANG_PATH = "${libclang.lib}/lib";
}
