{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust =
          let
            ci-extensions = [
              "rustfmt"
              "clippy"
            ];
          in
          {
            ci = pkgs.rust-bin.stable.latest.minimal.override {
              extensions = ci-extensions;
            };
            dev = pkgs.rust-bin.stable.latest.minimal.override {
              extensions = ci-extensions ++ [
                "rust-analyzer"
                "rust-src"
              ];
            };
          };
      in
      {
        devShells = {
          default = pkgs.mkShell {
            packages = [ rust.dev ];
          };
          ci = pkgs.mkShell {
            packages = [ rust.ci ];
          };
        };
      }
    );
}
