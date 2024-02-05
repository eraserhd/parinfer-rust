{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        localeEnv = if pkgs.stdenv.isDarwin then "" else "LOCALE_ARCHIVE=${pkgs.glibcLocales}/lib/locale/locale-archive";
        naersk' = pkgs.callPackage naersk { };

        parinfer-rust = naersk'.buildPackage {
          src = ./.;
          doCheck = true;
          nativeBuildInputs = [ pkgs.cargo-nextest ];
          buildFlags = [ "--release" "--no-default-features" ];
          cargoTestCommands = x: [ ''cargo nextest run'' ];
        };

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

        vim-tests = runVimTests "vim" "${pkgs.vim}/bin/vim";

        neovim-tests = runVimTests "neovim" "${pkgs.neovim}/bin/nvim";
      in
      rec {
        # For `nix build` & `nix run`:
        packages = {
          default = parinfer-rust;
          app = parinfer-rust;
        };

        checks = {
          neovim = neovim-tests;
          vim = vim-tests;
          kakoune = pkgs.stdenv.mkDerivation {
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

        devShells = with pkgs; {
          default = mkShell {
            nativeBuildInputs = [
              rustc
              cargo
              vim
              neovim
            ] ++ lib.optional stdenv.isDarwin libiconv;
          };
        };
      });
}
