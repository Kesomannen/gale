{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    devenv = {
      url = "github:cachix/devenv/v2.4.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  nixConfig = {
    extra-substituters = ["https://devenv.cachix.org"];
    extra-trusted-public-keys = ["devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="];
  };
  outputs = {
    self,
    nixpkgs,
    devenv,
    ...
  } @ inputs: let
    forSystems = systems: function:
      nixpkgs.lib.genAttrs systems (
        system: function nixpkgs.legacyPackages.${system}
      );
    forAllSystems = forSystems nixpkgs.lib.systems.flakeExposed;
    forPackageSystems = forSystems (nixpkgs.lib.intersectLists
      nixpkgs.lib.systems.flakeExposed
      ["aarch64-linux" "x86_64-linux" "aarch64-darwin" "x86_64-darwin"]);
  in {
    formatter = forAllSystems (pkgs: pkgs.alejandra);
    packages = forPackageSystems (pkgs: let
      gale = pkgs.callPackage ./packages/gale/package.nix {src = self;};
    in {
      inherit gale;
      default = gale;
    });
    devShells = forAllSystems (pkgs: {
      default = devenv.lib.mkShell {
        inherit inputs pkgs;
        modules = [./devenv.nix];
      };
    });
  };
}
