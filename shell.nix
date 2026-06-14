{ ... }:
let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  buildInputs = with pkgs; [ clang pkg-config opencv4 ];
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
}
