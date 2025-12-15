{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachSystem
      [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ]
      (
        system:
        let
          pkgs = import nixpkgs {
            overlays = [ (import rust-overlay) ];
            localSystem = system;
          };

          craneLib = crane.mkLib pkgs;

          commonArgs = {
            src = craneLib.cleanCargoSource ./.;

            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.gtk4
              pkgs.libadwaita
            ];

            # buildInputs =
            #   pkgs.lib.optionals pkgs.stdenv.isLinux [
            #   ]
            #   ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            #   ];
          };

          # Native build
          click = craneLib.buildPackage commonArgs;

          # Cross-compile to other Linux architectures
          mkLinuxCross =
            targetSystem: targetTriple:
            let
              crossPkgs = import nixpkgs {
                overlays = [ (import rust-overlay) ];
                localSystem = system;
                crossSystem = {
                  config = targetSystem;
                };
              };

              crossCraneLib = (crane.mkLib crossPkgs).overrideToolchain (
                p:
                p.rust-bin.stable.latest.default.override {
                  targets = [ targetTriple ];
                }
              );
            in
            crossCraneLib.buildPackage (
              commonArgs
              // {
                strictDeps = true;
                doCheck = false;

                buildInputs = [
                  crossPkgs.gtk4
                  crossPkgs.libadwaita
                ];
              }
            );

        in
        {
          packages = {
            default = click;
            native = click;
          }
          // pkgs.lib.optionalAttrs (pkgs.stdenv.isLinux && system == "x86_64-linux") {
            # Only offer aarch64-linux when building from x86_64-linux
            linux-aarch64 = mkLinuxCross "aarch64-unknown-linux-gnu" "aarch64-unknown-linux-gnu";
          }
          // pkgs.lib.optionalAttrs (pkgs.stdenv.isLinux && system == "aarch64-linux") {
            # Only offer x86_64-linux when building from aarch64-linux
            linux-x86_64 = mkLinuxCross "x86_64-unknown-linux-gnu" "x86_64-unknown-linux-gnu";
          };

          devShells.default = craneLib.devShell {
            inputsFrom = [ click ];
            packages = [
              pkgs.rust-analyzer
            ];
          };
        }
      );
}
