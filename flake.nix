{
  description = "Infer parentheses for Clojure, Lisp, and Scheme";
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        parinfer-rust = pkgs.callPackage ./derivation.nix {};
      in {
        packages = {
          default = parinfer-rust;
          inherit parinfer-rust;
        };
        checks = {
          test = pkgs.runCommandNoCC "parinfer-rust-test" {} ''
            mkdir -p $out
            : ${parinfer-rust}
          '';
        };
    })) // {
      overlays.default = final: prev: {
        parinfer-rust = prev.callPackage ./derivation.nix {};
      };
    };
}
