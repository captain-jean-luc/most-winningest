{
  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { nixpkgs, flake-utils, self }: 
    {
      overlays.default = final: prev: {
        most-winningest = final.callPackage ./package.nix { };
      };
    }
    //
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ self.overlays.default ];
      };
      diesel-cli = pkgs.diesel-cli;
    in rec {
      packages.default = pkgs.most-winningest;
      devShells.default = pkgs.mkShell { 
        packages = [ diesel-cli pkgs.rustc pkgs.cargo ];
        inputsFrom = [ packages.default ];
      };
    });
}
