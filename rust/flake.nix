{
  inputs.nixpkgs.url = "github:nixos/nixpkgs";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  outputs = {self, nixpkgs, rust-overlay}: let
    system = "aarch64-linux";
    pkgs = (import nixpkgs) {
      inherit system;
      overlays = [(import rust-overlay)];
    };
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        (pkgs.rust-bin.nightly.latest.default.override {
          extensions = ["rust-src" "rust-docs" "miri" "rust-analyzer"];
        })
      ];
    };
  };
}
