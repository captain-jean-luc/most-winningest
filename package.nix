{
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
}

