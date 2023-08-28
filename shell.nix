let 
rustOverlay = builtins.fetchTarball {
          url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
          sha256 = "04csw82q0y46y3bcpk645cfkid95q6ghnacw8b9x3lmwppwab686";
        };
in(import <nixpkgs> {
  crossSystem = {
    config = "x86_64-w64-mingw32";
  };
  overlays = [ (import rustOverlay) ];
}).callPackage (
{ mkShell, stdenv, rust-bin, windows, wine64 }:
mkShell {
  nativeBuildInputs = [
    rust-bin.stable.latest.minimal
  ];

  depsBuildBuild = [ wine64 ];
  buildInputs = [ windows.pthreads ];

  CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
  CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER = "wine64";
}) {}
