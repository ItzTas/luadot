{
  description = "A dotfiles manager configured in Lua";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forEachSystem = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      overlays.default = final: _prev: {
        luadot = final.callPackage ./nix/package.nix { };
      };

      packages = forEachSystem (
        system: rec {
          luadot = (pkgsFor system).callPackage ./nix/package.nix { };
          default = luadot;
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.luadot ];

            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.rustfmt
              pkgs.rust-analyzer
              pkgs.cocogitto
              pkgs.shellcheck
            ];
          };
        }
      );

      formatter = forEachSystem (system: (pkgsFor system).nixfmt-rfc-style);
    };
}
