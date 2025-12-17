{ pkgs, ... }:

{
  languages = {
    rust = {
      enable = true;
      channel = "stable";
    };
  };

  packages = with pkgs; [
    gtk4
    libadwaita
  ];
}
