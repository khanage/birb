{
  description = "Birb build";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nativeBuildInputs = with pkgs; [
          rust-bin.stable.latest.default
          pkg-config
          lld
          clang
        ];
        buildInputs = with pkgs; [
          udev
          wayland
          alsa-lib
          libxkbcommon
          vulkan-loader
        ];
      in
        with pkgs; {
          devShells.default = with pkgs;
            mkShell {
              inherit nativeBuildInputs buildInputs;
              LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;
            };
        }
    );
}
