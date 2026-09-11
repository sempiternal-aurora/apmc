{
  description = "rust compiler";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "rust";
          packages = [
            pkgs.git
          ];
          buildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rustc"
                "cargo"
                "clippy"
                "rustfmt"
                "rust-analyzer"
                "rust-src"
              ];
            })
          ];
        };
      }
    );
}
