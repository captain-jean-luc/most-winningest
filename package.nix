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
  cargoHash = "sha256-oSb9PDLVZbbkQOtc/9qE+bo24m9Agd2c/4XDRXpjhGw=";
}

