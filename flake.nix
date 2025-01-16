{
  description = "arachne - Humble Nix framework and Rust CLI for multi-node systems.";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";

    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, ... } @rawInputs:
    let
      inputs = rawInputs;

      libPart = {
        lib = nixpkgs.lib.mapAttrs'
          (name: value: { inherit name; value = value inputs; })
          (((import "${self}/src/lib/import.nix") inputs).importDir "${self}/src/lib");
      };

      scripts = nixpkgs.lib.mapAttrs'
        (name: value: { inherit name; value = value inputs; })
        (self.lib.import.importDir "${self}/scripts/flake");

      systemPart = flake-utils.lib.eachDefaultSystem (system: rec {
        devShells.dev = scripts.shell.mkDevShell system;
        devShells.docs = scripts.shell.mkDocsShell system;
        devShells.default = devShells.dev;
        formatter = scripts.formatter.mkFormatter system;
        checks = scripts.checks.mkChecks system;
        packages.arachne = scripts.packages.mkPackage system ./src/cli "arachne";
        packages.default = packages.arachne;
        apps.arachne = {
          type = "app";
          program = "${packages.arachne}/bin/arachne";
        };
        apps.default = apps.arachne;
      });
    in
    libPart // systemPart;
}
