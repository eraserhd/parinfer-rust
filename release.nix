{ nixpkgs ? (import ./nixpkgs.nix), ... }:
let
  pkgs = import nixpkgs { config = {}; };
  parinfer-rust = pkgs.callPackage ./derivation.nix {};
in {
  test = parinfer-rust.overrideAttrs (self: {
    postBuild = ''
      RUST_BACKTRACE=1 cargo test
    '';
  });
}
