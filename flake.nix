{
  description = "mdbook-drawio dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # Tool used to manage the changelog, see 'changelog/README.md'. Pinned,
    # because the flake of later revisions misses the `colorama` dependency.
    qlog = {
      url = "github:QBayLogic/qlog/3bea2429e555e969984fbf897216cee74df2f73d";
      inputs.flake-utils.follows = "flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, qlog }:
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
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
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
            qlog.packages.${system}.default
          ];
          RUST_BACKTRACE = 1;
        };

        defaultPackage = rustPackage;
      }
    );
}
