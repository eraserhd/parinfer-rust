{ nixpkgs ? (import ./nixpkgs.nix), ... }:
let
  pkgs = import nixpkgs { config = {}; };
  parinfer-rust = pkgs.callPackage ./derivation.nix {};
in {
  unit-tests = parinfer-rust.overrideAttrs (self: {
    postBuild = ''
      RUST_BACKTRACE=1 cargo test
    '';
  });

  vim-tests = pkgs.stdenv.mkDerivation {
    name = "parinfer-rust-vim-tests";
    src = ./tests/vim;
    buildPhase = ''
      LC_ALL=en_US.UTF-8 \
        LOCALE_ARCHIVE=${pkgs.glibcLocales}/lib/locale/locale-archive \
        VIM_TO_TEST=${pkgs.vim}/bin/vim \
        PLUGIN_TO_TEST=${parinfer-rust}/share/vim-plugins/parinfer-rust \
        ${pkgs.vim}/bin/vim --clean -u run.vim
    '';
    installPhase = ''
      touch $out
    '';
  };
}
