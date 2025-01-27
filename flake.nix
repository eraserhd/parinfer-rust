{
  description = "Infer parentheses for Clojure, Lisp, and Scheme";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        parinfer-rust = pkgs.callPackage ./derivation.nix {};

        localeEnv = if pkgs.stdenv.isDarwin
                    then ""
                    else "LOCALE_ARCHIVE=${pkgs.glibcLocales}/lib/locale/locale-archive";
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
        packages = {
          default = parinfer-rust;
          inherit parinfer-rust;
        };
        checks = {
          vim-tests = runVimTests "vim" "${pkgs.vim}/bin/vim";

          #FIXME: Currently broken
          #neovim-tests = runVimTests "neovim" "${pkgs.neovim}/bin/nvim";

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
        };
        devShells.default = parinfer-rust.overrideAttrs (oldAttrs: {
          nativeBuildInputs = oldAttrs.nativeBuildInputs ++ (with pkgs; [
            vim
            neovim
          ]);
        });
    })) // {
      overlays.default = final: prev: {
        parinfer-rust = prev.callPackage ./derivation.nix {};
      };
    };
}
