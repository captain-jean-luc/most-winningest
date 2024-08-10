{
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  openssl,
  postgresql,
}: rustPlatform.buildRustPackage rec {
  pname = "most-winningest";
  version = "69.420";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl postgresql ];

  src = ./.;

  cargoHash = "sha256-lYIUYRsgANjba+VlBgTOWMN7AyXBRkGlmdK51qxJ7as=";
}

