const NAME: &str = env!("CARGO_PKG_NAME");

lazy_static::lazy_static! {
  static ref BASE: String = format!(
    r#"
      pkgs.dockerTools.buildImage {{
        name = "{NAME}/base";
        tag = "latest";
        created = "now";
        copyToRoot = with pkgs.dockerTools; [
          usrBinEnv
          binSh
        ];
      }}
    "#
  );
}

pub(crate) struct Image {
  nix_path: std::path::PathBuf,
  artifact: std::path::PathBuf,
}

impl Image {
  pub(crate) async fn new(name: &str) -> anyhow::Result<Self> {
    let nix_path: std::path::PathBuf =
      std::env::var(NAME.to_uppercase() + "_NIX_PATH")?.try_into()?;
    let base: String = BASE.to_string();
    let spec = format!(
      r#"
        let
          pkgs = import <nixpkgs> {{ }};
          base = {base};
        in
        pkgs.dockerTools.buildImage {{
          name = "{NAME}/{name}";
          tag = "latest";
          created = "now";
          fromImage = base;
          copyToRoot = pkgs.buildEnv {{
            name = "image-root";
            paths = [ pkgs.hello ];
            pathsToLink = [ "/bin" ];
          }};
          config = {{
            Cmd = [ "hello" ];
          }};
        }}
      "#
    );
    let result = tokio::process::Command::new("nix")
      .arg("build")
      .arg("--print-out-paths")
      .arg("--no-link")
      .arg("--impure")
      .arg("--expr")
      .arg(spec)
      .output()
      .await?;
    if !result.status.success() {
      let stderr = String::from_utf8(result.stderr)?;
      return Err(anyhow::format_err!(
        "Image creation failed because {stderr}"
      ));
    }
    let artifact: std::path::PathBuf =
      String::from_utf8(result.stdout)?.trim().into();
    tracing::info!("Created image at {artifact:?}");
    Ok(Self { nix_path, artifact })
  }

  pub(crate) fn artifact(&self) -> &std::path::Path {
    return self.artifact.as_path();
  }
}
