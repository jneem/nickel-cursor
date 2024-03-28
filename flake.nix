{
  description = "Basic rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
    nickel.url = "github:tweag/nickel";
  };

  outputs = { nixpkgs, fenix, flake-utils, nickel, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-toolchain = pkgs.fenix.latest.withComponents [
          "cargo"
          "clippy"
          "rustc"
          "rustfmt"
          "rust-analyzer"
          "rust-src"
        ];

        nickel-cursor = pkgs.rustPlatform.buildRustPackage {
          pname = "nickel-cursor";
          version = "0.1.0";

          src = pkgs.lib.sources.sourceByRegex ./. ["Cargo\.*" "src" "src/.*"];
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

        cursor-ncl = pkgs.stdenv.mkDerivation {
          name = "cursor-ncl";
          version = "0.1.0";
          src = ./cursor.ncl;

          unpackPhase = "cp $src cursor.ncl";
          installPhase = ''
            mkdir -p $out
            cp cursor.ncl $out/cursor.ncl
          '';
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            cargo-outdated
            nickel.packages.${system}.nickel-lang-cli
            nickel.packages.${system}.lsp-nls
            nls
            rust-toolchain
          ];
        };

        packages.nickel-cursor = nickel-cursor;
        packages.cursor-ncl = cursor-ncl;
        packages.default = pkgs.symlinkJoin { name = "nickel-cursor"; paths = [nickel-cursor cursor-ncl]; };
      }
    );
}
