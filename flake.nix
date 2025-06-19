{
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    self,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = (import nixpkgs) {
        inherit system;
        overlays = [(import rust-overlay) self.overlays.default];
      };
    in {
      packages = {
        default = pkgs.alush;
        static = pkgs.pkgsStatic.alush;

        # For releases.
        cross-static-aarch64-linux = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic.alush;
        cross-static-x86_64-linux = pkgs.pkgsCross.musl64.pkgsStatic.alush;
      };

      devShells.default = pkgs.mkShell {
        packages = [
          (pkgs.rust-bin.nightly.latest.default.override {
            extensions = ["rust-src" "rust-docs" "miri" "rust-analyzer"];
          })
        ];
      };
    })
    // {
      overlays.default = final: prev: {
        alush = final.callPackage ./package.nix {};
      };
    };
}
