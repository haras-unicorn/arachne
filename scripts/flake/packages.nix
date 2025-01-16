{ nixpkgs, crane, ... }:

let
  mkCraneLib = pkgs: crane.mkLib pkgs;

  mkSrc = pkgs: ((mkCraneLib pkgs).cleanCargoSource ../../.);

  mkCommon = pkgs: {
    src = mkSrc pkgs;
    strictDeps = true;

    nativeBuildInputs = [
      pkgs.pkg-config
    ];
    buildInputs = [
      pkgs.openssl
    ];
  };

  mkCargoArtifacts = pkgs: (mkCraneLib pkgs).buildDepsOnly (mkCommon pkgs);

  mkIndividual = pkgs: (mkCommon pkgs) // {
    cargoArtifacts = mkCargoArtifacts pkgs;
    inherit ((mkCraneLib pkgs).crateNameFromCargoToml { src = mkSrc pkgs; }) version;
  };

  mkCrateSrc = pkgs: crate: nixpkgs.lib.fileset.toSource {
    root = ../../.;
    fileset = nixpkgs.lib.fileset.unions [
      ../../Cargo.toml
      ../../Cargo.lock
      ((mkCraneLib pkgs).fileset.commonCargoSources crate)
    ];
  };

  mkPackage = system: crate: name:
    let
      pkgs = import nixpkgs { inherit system; };
    in
    (mkCraneLib pkgs).buildPackage ((mkIndividual pkgs) // {
      pname = name;
      cargoExtraArgs = "-p ${name}";
      cargoArtifacts = (mkCraneLib pkgs).buildDepsOnly (mkCommon pkgs);
      src = mkCrateSrc pkgs crate;
    });
in
{
  inherit mkPackage;
}
