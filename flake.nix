{
  description = "edifact-bo4e-automapper — Rust EDIFACT ↔ BO4E conversion";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Rust
            rust-toolchain

            # Task runner
            pkgs.just

            # Testing tools
            pkgs.cargo-insta
            pkgs.cargo-nextest

            # Dev tools
            pkgs.tokei
            pkgs.cargo-watch

            # For leptos WASM builds
            pkgs.trunk
            pkgs.wasm-bindgen-cli
          ];

          shellHook = ''
            echo "edifact-bo4e-automapper dev shell"
            echo "Rust $(rustc --version)"
            echo "Run 'just' to see available commands"
          '';
        };
      }
    );
}
