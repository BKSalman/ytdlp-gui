{
  description = "A GUI for yt-dlp written in Rust";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        rustOverlay = builtins.fetchTarball {
          url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
          sha256 = "0vzbfcn291hp2ksw4anrddqhqsz75ydwzc2z2gswv695m58yl862";
        };
        pkgs = import nixpkgs { inherit system; overlays = [ (import rustOverlay) ]; };
      in with pkgs; {
        devShell = mkShell rec {
          buildInputs = [
            rust-bin.stable.latest.default
            rust-analyzer
            cmake
            fontconfig
            pkg-config

            # vulkan-headers
            vulkan-loader

            # libxkbcommon

            # WINIT_UNIX_BACKEND=wayland
            # wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
          ];
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      });
}
