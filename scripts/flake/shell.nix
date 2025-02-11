{ nixpkgs, ... }:

{
  mkDevShell = system:
    let
      pkgs = import nixpkgs { inherit system; };
    in
    pkgs.mkShell {
      shellHook = ''
        name="$(basename -s .git "$(git config --get remote.origin.url)")"
        export ''${name^^}_TEST_TMP="$(git rev-parse --show-toplevel)/tmp"
        export ''${name^^}_TEST_NIX_PATH="$(realpath $(which nix))"
        export ''${name^^}_TEST_DOCKER_PATH="$(realpath $(which docker))"
        export ''${name^^}_TEST_NIXOS_CONTAINER_PATH="$(realpath $(which nixos-container))"
        export ''${name^^}_TEST_SYSTEMD_NSPAWN_PATH="$(realpath $(which systemd-nspawn))"
        export ''${name^^}_TEST_SLIRP4NETNS_PATH="$(realpath $(which slirp4netns))"
        export ''${name^^}_TEST_NSENTER_PATH="$(realpath $(which nsenter))"
      '';

      buildInputs = [
        pkgs.pkg-config
        pkgs.openssl
        pkgs.nixVersions.stable
        pkgs.nixos-container
        pkgs.git
        pkgs.docker-client
        pkgs.systemd
        pkgs.slirp4netns
        pkgs.util-linux
      ];

      packages = with pkgs; [
        # versioning
        git
        commitlint-rs

        # scripts
        just
        nushell

        # spelling
        nodePackages.cspell

        # markdown
        mdbook
        marksman
        markdownlint-cli
        nodePackages.markdown-link-check

        # misc
        nodePackages.prettier
        nodePackages.yaml-language-server
        nodePackages.vscode-langservers-extracted
        taplo

        # nix
        nil
        nixpkgs-fmt

        # rust
        llvmPackages.clangNoLibcxx
        lldb
        rustc
        cargo
        clippy
        rustfmt
        rust-analyzer
        cargo-edit
        cargo-semver-checks
      ] ++ pkgs.lib.optionals
        (
          pkgs.stdenv.hostPlatform.isLinux
        ) [
        # NOTE: broken on darwin
        release-plz
      ];
    };

  mkDocsShell = system:
    let
      pkgs = import nixpkgs { inherit system; };
    in
    pkgs.mkShell {
      packages = with pkgs; [
        # scripts
        just
        nushell

        # markdown
        mdbook

        # rust
        cargo
      ];
    };
}
