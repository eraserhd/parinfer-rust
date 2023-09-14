{ nixpkgs ? (import ./nixpkgs.nix), ... }:
let
  pkgs = import nixpkgs {
    config = {};
    overlays = [
      (import ./overlay.nix)
    ];
  };
  parinfer-rust = pkgs.parinfer-rust;
  localeEnv = if pkgs.stdenv.isDarwin then "" else "LOCALE_ARCHIVE=${pkgs.glibcLocales}/lib/locale/locale-archive";

  runVimTests = name: path: pkgs.stdenv.mkDerivation {
    name = "parinfer-rust-${name}-tests";
    src = ./tests/vim;
    buildPhase = ''
      printf 'Testing %s\n' '${path}'
      LC_ALL=en_US.UTF-8 \
        ${localeEnv} \
        VIM_TO_TEST=${path} \
        PLUGIN_TO_TEST=${parinfer-rust}/share/vim-plugins/parinfer-rust \
        ${pkgs.vim}/bin/vim --clean -u run.vim
    '';
    installPhase = ''
      touch $out
    '';
  };

in {
  vim-tests = runVimTests "vim" "${pkgs.vim}/bin/vim";

  neovim-tests = runVimTests "neovim" "${pkgs.neovim}/bin/nvim";

  kakoune-tests = pkgs.stdenv.mkDerivation {
    name = "parinfer-rust-kakoune-tests";
    src = ./tests/kakoune;
    buildInputs = [
      pkgs.kakoune-unwrapped
      parinfer-rust
    ];
    buildPhase = ''
      patchShebangs ./run.sh
      PLUGIN_TO_TEST=${parinfer-rust}/share/kak/autoload/plugins ./run.sh
    '';
    installPhase = ''
      touch $out
    '';
  };
}
