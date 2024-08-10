{
  inputs.nixpkgs.url = "nixpkgs";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };
    in {
      packages.default = pkgs.callPackage ./package.nix {};
      devShells.default = pkgs.mkShell { 
        packages = [ pkgs.diesel-cli pkgs.clippy ];
        inputsFrom = [ self.packages."x86_64-linux".default ];
      };
    });
}
