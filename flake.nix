{
  description = "MPD Protocol spec w/ Navidrome Support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      systems = [
        "x86_64-linux"
      ];
    in
    {
      nixosModules.default = import ./nix/module.nix { inherit self; };
    }
    // flake-utils.lib.eachSystem systems (system:
      let
        pkgs = import nixpkgs { inherit system; };
        wildflower = pkgs.rustPlatform.buildRustPackage {
          pname = "wildflower";
          version = "0.67";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.alsa-lib ];
          meta.mainProgram = "mpdnavi";
        };
      in
      {
        packages.default = wildflower;
        packages.obsidianfm = wildflower;

        apps.default = flake-utils.lib.mkApp {
          drv = wildflower;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            clippy
            pkg-config
            rustc
            rustfmt
          ];
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
