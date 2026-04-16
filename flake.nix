{
  description = "mdbook-drawio dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
          };
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rustfmt" "clippy" ];
        };
        rustPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "mdbook-drawio";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          # Remember to replace this placeholder hash!
          cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            mdbook
            drawio-headless
            rustToolchain
          ];
          RUST_BACKTRACE = 1;
        };

        defaultPackage = rustPackage;
      }
    );
}
