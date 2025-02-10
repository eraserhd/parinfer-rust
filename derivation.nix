{ lib, stdenv, libiconv, rustPlatform, fetchFromGitHub, llvmPackages }:

rustPlatform.buildRustPackage rec {
  name = "parinfer-rust-${version}";
  version = "0.5.0";

  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  useFetchCargoVendor = true;

  nativeBuildInputs = [
    llvmPackages.libclang
    llvmPackages.clang
    libiconv
    rustPlatform.bindgenHook
  ];
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  postInstall = ''
    mkdir -p $out/share/kak/autoload/plugins
    sed "s,^str parinfer_path .*,str parinfer_path '${placeholder "out"}/bin/parinfer-rust'," \
      rc/parinfer.kak >$out/share/kak/autoload/plugins/parinfer.kak

    rtpPath=$out/share/vim-plugins/parinfer-rust
    mkdir -p $rtpPath/plugin
    sed "s,let s:libdir = .*,let s:libdir = '${placeholder "out"}/lib'," \
      plugin/parinfer.vim >$rtpPath/plugin/parinfer.vim
  '';

  meta = with lib; {
    description = "Infer parentheses for Clojure, Lisp, and Scheme.";
    homepage = "https://github.com/eraserhd/parinfer-rust";
    license = licenses.isc;
    maintainers = with maintainers; [ eraserhd ];
    platforms = platforms.all;
  };
}
