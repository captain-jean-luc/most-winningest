{
  lib,

  rustPlatform,
  pkg-config,
  openssl,
  libpq,
}: rustPlatform.buildRustPackage {
  pname = "most-winningest";
  version = "69.420";
  auditable = false; #cargo-auditable in nixpkgs doesn't support 2024 edition yet

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [
    openssl
    libpq
  ];

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "Program to determin who is the most winningest on the LOTPW thread";
    homepage = "https://github.com/captain-jean-luc/most-winningest";
    license = lib.licenses.gpl3;
    mainProgram = "most-winningest";
    platforms = lib.platforms.all;
  };
}

