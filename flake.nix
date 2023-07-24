{
  description = "A GUI for yt-dlp written in Rust";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        rustOverlay = builtins.fetchTarball {
          url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
          sha256 = "1cr3ph1ww4scgw3hdhnag2qpqx36xplvlsjwa3z6rmrf424zqx9z";
        };
        pkgs = import nixpkgs { inherit system; overlays = [ (import rustOverlay) ]; };
        libPath =  with pkgs; lib.makeLibraryPath [
          libGL
          bzip2
          fontconfig
          freetype
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
        ];
      in with pkgs; {
        devShell = mkShell rec {
          packages = [
            rust-bin.stable.latest.default
            rust-analyzer
          ];
          nativeBuildInputs = with pkgs; [
            glibc
            pkg-config
            cmake
            mesa
            makeWrapper
          ];

          buildInputs = with pkgs; [
            fontconfig
            freetype

            vulkan-headers
            vulkan-loader
            libGL

            libxkbcommon
            # WINIT_UNIX_BACKEND=wayland
            wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
          ];
          LD_LIBRARY_PATH = "${libPath}";
        };
      });
}
