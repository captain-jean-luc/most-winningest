{
  rustPlatform,
  pkg-config,
  openssl,
  postgresql,
}: rustPlatform.buildRustPackage {
  pname = "most-winningest";
  version = "69.420";
  auditable = false; #cargo-auditable in nixpkgs doesn't support 2024 edition yet

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl postgresql ];

  src = ./.;

  useFetchCargoVendor = true;
  cargoLock.lockFile = ./Cargo.lock;
}

