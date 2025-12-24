{ pkgs, lib, ... }:

{
  languages = {
    rust = {
      enable = true;
      channel = "stable";
    };
  };

  packages = lib.optionals pkgs.stdenv.isLinux (
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
