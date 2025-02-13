{
  rustPlatform,
  pkg-config,
  openssl,
  postgresql,
}: rustPlatform.buildRustPackage {
  pname = "most-winningest";
  version = "69.420";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl postgresql ];

  src = ./.;

  useFetchCargoVendor = true;
  cargoHash = "";
}

