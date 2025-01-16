{ nixpkgs, ... }:

{
  mkChecks = system:
    let
      pkgs = import nixpkgs { inherit system; };
    in
    {
      commitlint = pkgs.writeShellApplication {
        name = "commitlint";
        runtimeInputs = [ pkgs.commitlint-rs ];
        text = ''
          commitlint --from main
        '';
      };
      just = pkgs.writeShellApplication {
        name = "just";
        runtimeInputs = [ pkgs.just ];
        text = ''
          cd "$(git rev-parse --show-toplevel)"
          just --unstable --fmt --check
        '';
      };
      prettier = pkgs.writeShellApplication {
        name = "prettier";
        runtimeInputs = [ pkgs.nodePackages.prettier ];
        text = ''
          prettier --check "$(git rev-parse --show-toplevel)"
        '';
      };
      nixpkgs-fmt = pkgs.writeShellApplication {
        name = "nixpkgs-fmt";
        runtimeInputs = [ pkgs.nixpkgs-fmt ];
        text = ''
          nixpkgs-fmt "$(git rev-parse --show-toplevel)"
        '';
      };
      cspell = pkgs.writeShellApplication {
        name = "cspell";
        runtimeInputs = [ pkgs.nodePackages.cspell ];
        text = ''
          cspell lint "$(git rev-parse --show-toplevel)" --no-progress
        '';
      };
      clippy = pkgs.writeShellApplication {
        name = "clippy";
        runtimeInputs = [ pkgs.cargo pkgs.clippy ];
        text = ''
          cd "$(git rev-parse --show-toplevel)"
          cargo clippy -- -D warnings
        '';
      };
    };
}
