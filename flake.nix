{
  description = "A GUI for yt-dlp written in Rust";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        craneLib = crane.lib.${system};
        rustOverlay = builtins.fetchTarball {
          url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
          sha256 = "1jcfh1n57sq3g1mxdf6grqc0rcpams14gbampshfvx0g459b2sj9";
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

        nativeBuildInputs = with pkgs; [
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
      in with pkgs; {
        packages = {
            ytdlp-gui = craneLib.buildPackage {
            src = craneLib.cleanCargoSource (craneLib.path ./.);

            inherit buildInputs nativeBuildInputs;

            # idk how to make this work
            # postInstall = ''
            #   for _size in "16x16" "32x32" "48x48" "64x64" "128x128" "256x256"; do
            #       install -Dm644 "$src/data/icons/$_size/apps/ytdlp-gui.png" "$out/share/icons/hicolor/$_size/apps/ytdlp-gui.png"
            #   done
            #   install -Dm644 "$src/data/applications/ytdlp-gui.desktop" -t "$out/share/applications/"
            # '';

            postInstall = ''
              wrapProgram $out/bin/ytdlp-gui \
                --prefix PATH : ${lib.makeBinPath [ pkgs.gnome.zenity pkgs.libsForQt5.kdialog]}\
                --suffix LD_LIBRARY_PATH : ${libPath}
            '';
          };

          default = ytdlp-gui;
        };

        devShell = mkShell {
          inherit buildInputs nativeBuildInputs;

          packages = with pkgs; [
            rust-bin.stable.latest.default
            rust-analyzer
          ];
          LD_LIBRARY_PATH = "${libPath}";
        };
      }) // {
        overlay = final: prev: {
          inherit (self.packages.${final.system}) ytdlp-gui;
        };
      };
}
