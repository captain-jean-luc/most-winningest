{ system ? builtins.currentSystem }:
(builtins.getFlake ("git+file://" + toString ./.)).devShells.${system}.default
