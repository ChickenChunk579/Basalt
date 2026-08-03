{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            pkgconf
            pkg-config
            raylib
            clang
            mold
          ];

          shellHook = ''
			export PATH=$PATH:$PWD/target/debug
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "basalt";
          version = "0.1.0";

          src = pkgs.lib.cleanSource ./.;

          cargoLock.lockFile = ./Cargo.lock;
        };
      });
}
