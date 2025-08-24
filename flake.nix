{
  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, flake-utils, fenix, ... }: 
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };
      toolchain = fenix.packages.${system}.stable.completeToolchain;
      rustPlatform = pkgs.makeRustPlatform {
        cargo = toolchain;
        rustc = toolchain;
      };
      diesel-cli = pkgs.diesel-cli.override { inherit rustPlatform; };
    in rec {
      packages.default = pkgs.callPackage ./package.nix { inherit rustPlatform; };
      devShells.default = pkgs.mkShell { 
        packages = [ diesel-cli toolchain ];
        inputsFrom = [ packages.default ];
      };
    });
}
