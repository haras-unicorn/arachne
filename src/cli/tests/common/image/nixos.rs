// TODO: pure
// TODO: stateVersion
// NOTE: https://wiki.nixos.org/wiki/NixOS_Containers

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

const NIXOS_SYSTEM: &str =
  "(builtins.getFlake (builtins.toString <nixpkgs>)).lib.nixosSystem";

pub struct NixosImage {
  nix_path: std::path::PathBuf,
  artifact: std::path::PathBuf,
  name: String,
}

impl NixosImage {
  pub(crate) async fn new<TPkgs>(
    name: &str,
    modules: TPkgs,
  ) -> anyhow::Result<Self>
  where
    TPkgs: IntoIterator,
    TPkgs::Item: std::fmt::Display,
  {
    let nix_path: std::path::PathBuf =
      std::env::var(CARGO_PKG_NAME.to_uppercase() + "_TEST_NIX_PATH")?.into();

    let modules = itertools::Itertools::join(
      &mut modules.into_iter().map(|module| format!("({module})")),
      " ",
    );
    let spec = format!(
      r#"
        let
          nixosSystem = {NIXOS_SYSTEM} {{
            system = builtins.currentSystem;
            modules = [
              {modules}
              ({{ lib, ... }}: {{
                boot.isContainer = lib.mkForce true;
                system.stateVersion = lib.mkForce "24.11";
              }})
            ];
          }};
        in
        nixosSystem.config.system.build.toplevel
      "#
    );

    let build_output = tokio::process::Command::new(nix_path.as_os_str())
      .arg("build")
      .arg("--print-out-paths")
      .arg("--no-link")
      .arg("--impure")
      .arg("--expr")
      .arg(spec)
      .output()
      .await?;
    if !build_output.status.success() {
      let stderr = String::from_utf8(build_output.stderr)?;
      return Err(anyhow::anyhow!("Image creation failed because {stderr}"));
    }
    let artifact: std::path::PathBuf =
      String::from_utf8(build_output.stdout)?.trim().into();
    tracing::info!("Created image at {artifact:?}");

    Ok(Self {
      nix_path,
      artifact,
      name: name.to_owned(),
    })
  }

  pub(crate) fn artifact(&self) -> &std::path::Path {
    return self.artifact.as_path();
  }

  pub(crate) fn name(&self) -> &str {
    &self.name
  }
}
