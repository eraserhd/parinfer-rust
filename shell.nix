let
  nixpkgs = import ./nixpkgs.nix;
  pkgs = import nixpkgs {
    config = {};
    overlays = [ (import ./overlay.nix) ];
  };
in pkgs.parinfer-rust.overrideAttrs (oldAttrs: {
  nativeBuildInputs = oldAttrs.nativeBuildInputs ++ (with pkgs; [
    vim
    neovim
  ]);
})
