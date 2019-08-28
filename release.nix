{ nixpkgs ? (import ./nixpkgs.nix), ... }:
let
  pkgs = import nixpkgs { config = {}; };
  parinfer-rust = pkgs.callPackage ./derivation.nix {};
in {
  test = pkgs.runCommandNoCC "parinfer-rust-test" {} ''
    true
  '';
}