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
    name = "parinfer-rust-vim-tests-2019.10.24";
    src = ./.;
    buildPhase = ''
      export VIM_TO_TEST=${pkgs.vim}/bin/vim
      LC_ALL=en_US.UTF-8 \
        LOCALE_ARCHIVE=${pkgs.glibcLocales}/lib/locale/locale-archive \
        $VIM_TO_TEST --clean -u tests/vim/run.vim
    '';
    installPhase = ''
      touch $out
    '';
  };
}
