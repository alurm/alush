{
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = (import nixpkgs) {
        inherit system;
        overlays = [(import rust-overlay)];
      };
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
        pname = "alush";
        version = "0.1.0";
        src = ./.;
        cargoHash = "sha256-yyNDCyVE+s9mFWmQPvCl4U8g82wk4KwU3kPk3rBqKrA=";
        meta = {
          homepage = "https://github.com/alurm/alush";
          maintainers = [];
          license = pkgs.lib.licenses.mit;
          description = "A GC and a shell with closures and maps in Rust";
        };
      });
      devShells.default = pkgs.mkShell {
        packages = [
          (pkgs.rust-bin.nightly.latest.default.override {
            extensions = ["rust-src" "rust-docs" "miri" "rust-analyzer"];
          })
        ];
      };
    });
}
