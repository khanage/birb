{
  description = "Birb build";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
     (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nativeBuildInputs = with pkgs; [
          rust-bin.stable.latest.default
          pkg-config
        ];
        buildInputs = with pkgs; [ 
          udev
          wayland
          alsa-lib
          libxkbcommon
          vulkan-loader
        ];
      in
      with pkgs;
      {
          devShells.default = with pkgs; mkShell {
            inherit nativeBuildInputs buildInputs;

            shellHook = ''
              export LD_LIBRARY_PATH=${
                pkgs.lib.makeLibraryPath [ 
                  udev
                  alsa-lib
                  vulkan-loader
                  wayland
                  libxkbcommon
                ]}
            
              echo 🎯 Setting LD_LIBRARY_PATH=$LD_LIBRARY_PATH
            '';
          };
        }
     );
}
