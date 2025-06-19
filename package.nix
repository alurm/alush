{
  rustPlatform,
  lib,
  ...
}:
rustPlatform.buildRustPackage {
  pname = "alush";
  version = "0.1.0";
  src = ./.;
  cargoHash = "sha256-yyNDCyVE+s9mFWmQPvCl4U8g82wk4KwU3kPk3rBqKrA=";
  meta = {
    homepage = "https://github.com/alurm/alush";
    maintainers = [];
    license = lib.licenses.mit;
    description = "A GC and a shell with closures and maps in Rust";
  };
}
