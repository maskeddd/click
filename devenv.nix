{ pkgs, lib, ... }:

let
  cargo-packager = pkgs.rustPlatform.buildRustPackage rec {
    pname = "cargo-packager";
    version = "0.11.8";

    src = pkgs.fetchCrate {
      inherit pname version;
      sha256 = "sha256-DjqrsomwtM5JzGrBIjfREZ15pUijza+/p+3CwXe+dSY=";
    };

    cargoLock = {
      lockFile = "${src}/Cargo.lock";
    };
  };
in
{
  languages = {
    rust = {
      enable = true;
      channel = "stable";
    };
  };

  packages = [
    cargo-packager
  ]
  ++ lib.optionals pkgs.stdenv.isLinux (
    with pkgs;
    [
      libxkbcommon
      libGL
      wayland
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      xorg.libX11
    ]
  );

  # Only define LD_LIBRARY_PATH on Linux
  env.LD_LIBRARY_PATH = lib.mkIf pkgs.stdenv.isLinux (
    lib.makeLibraryPath [
      pkgs.libxkbcommon
      pkgs.libGL
      pkgs.wayland
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.xorg.libX11
    ]
  );

}
