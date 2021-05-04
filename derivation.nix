{ lib, stdenv, libiconv, rustPlatform, fetchFromGitHub, llvmPackages }:

rustPlatform.buildRustPackage rec {
  name = "parinfer-rust-${version}";
  version = "0.4.3";

  src = ./.;
  cargoSha256 = "69uH+D1I48ozo5aVjwUgVRtS2AHv65v4SgmcbLyN5PI=";

  buildInputs = [
    llvmPackages.libclang
    llvmPackages.clang
    libiconv
  ];
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  preConfigure = ''
    # cc-rs crate tries to use XCode on Mac OS X
    cat >build.rs <<EOF
    use std::{env,fs,process};
    fn main() {
      let out_dir = env::var("OUT_DIR").expect("$OUT_DIR is not set.");
      let c_dir = format!("{}/c", out_dir);
      fs::create_dir_all(&c_dir).expect("unable to create C directory");
      let object_file = format!("{}/parinfer.o", c_dir);
      let library_file = format!("{}/libparinfer.a", c_dir);
      process::Command::new("cc").args(&["-O2", "-o", &object_file, "-c", "parinfer.c"]).output().expect("failed to compile");
      process::Command::new("ar").args(&["rcs", &library_file, &object_file]).output().expect("failed to make library");
      println!("cargo:rustc-link-search={}", c_dir);
    }
    EOF
  '';

  postInstall = ''
    mkdir -p $out/share/kak/autoload/plugins
    cp rc/parinfer.kak $out/share/kak/autoload/plugins/

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
