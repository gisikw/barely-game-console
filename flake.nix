{
  description = "barely-game-console — RFID-powered retro game console launcher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        guiBuildInputs = with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
          libGL
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
        ];

        runtimeLibPath = pkgs.lib.makeLibraryPath (with pkgs; [
          wayland
          libxkbcommon
          libGL
          vulkan-loader
        ]);

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          buildInputs = guiBuildInputs;
          nativeBuildInputs = with pkgs; [ pkg-config makeWrapper ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        package = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          postInstall = ''
            mkdir -p $out/share/barely-game-console
            cp ${./retroarch.cfg} $out/share/barely-game-console/retroarch.cfg
            wrapProgram $out/bin/barely-game-console \
              --prefix LD_LIBRARY_PATH : ${runtimeLibPath} \
              --set BGC_RETROARCH_CONFIG $out/share/barely-game-console/retroarch.cfg
            cp ${./overlay.nix} $out/overlay.nix
          '';
        });
      in
      {
        packages.default = package;

        devShells.default = craneLib.devShell {
          inputsFrom = [ package ];
          packages = with pkgs; [ rust-analyzer cargo-watch just ];
          shellHook = ''
            echo "barely-game-console — just --list for recipes"
          '';
        };
      }
    );
}
