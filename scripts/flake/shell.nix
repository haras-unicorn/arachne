{ nixpkgs, ... }:

{
  mkDevShell = system:
    let
      pkgs = import nixpkgs { inherit system; };
    in
    pkgs.mkShell {
      RUST_BACKTRACE = "full";

      buildInputs = with pkgs; [
        pkg-config
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
