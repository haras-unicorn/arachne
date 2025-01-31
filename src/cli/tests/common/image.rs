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
  name: String,
}

impl Image {
  pub(crate) async fn new(name: &str) -> anyhow::Result<Self> {
    let nix_path: std::path::PathBuf =
      std::env::var(NAME.to_uppercase() + "_NIX_PATH")?.try_into()?;
    let version = "latest";
    let base: String = BASE.to_string();
    let spec = format!(
      r#"
        let
          pkgs = import <nixpkgs> {{ }};
          base = {base};
        in
        pkgs.dockerTools.buildImage {{
          name = "{NAME}/{name}";
          tag = "{version}";
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
    let name = format!("{NAME}/{name}:{version}");

    let build_output = tokio::process::Command::new("nix")
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
      return Err(anyhow::format_err!(
        "Image creation failed because {stderr}"
      ));
    }
    let artifact: std::path::PathBuf =
      String::from_utf8(build_output.stdout)?.trim().into();
    tracing::info!("Created image at {artifact:?}");

    // TODO: this thing...
    // let file = tokio::fs::File::open(artifact).await?;
    // let mut byte_stream =
    //   codec::FramedRead::new(file, codec::BytesCodec::new())
    //     .map(|r| r.unwrap().freeze());
    // bollard::Docker::connect_with_defaults()?.import_image(
    //   bollard::image::ImportImageOptions {
    //     ..Default::default()
    //   },
    //   byte_stream,
    //   None,
    // );
    let load = tokio::process::Command::new("docker")
      .arg("load")
      .arg("--input")
      .arg(artifact.clone())
      .output()
      .await?;
    if !load.status.success() {
      match String::from_utf8(load.stderr) {
        Ok(stderr) => {
          return Err(anyhow::format_err!(
            "Failed loading image because {stderr}"
          ))
        }
        Err(err) => {
          return Err(anyhow::format_err!(
            "Failed loading image and failed parsing stderr because {err}"
          ));
        }
      }
    }
    tracing::info!("Loaded image {name}");

    Ok(Self {
      nix_path,
      artifact,
      name,
    })
  }

  pub(crate) fn artifact(&self) -> &std::path::Path {
    return self.artifact.as_path();
  }

  pub(crate) fn name(&self) -> &str {
    return &self.name;
  }
}

impl Drop for Image {
  fn drop(&mut self) {
    let result = match std::process::Command::new("docker")
      .arg("image")
      .arg("rm")
      .arg(self.name.to_owned())
      .output()
    {
      Err(err) => {
        tracing::error!("Failed running image remove because {err}");
        return;
      }
      Ok(result) => result,
    };

    if !result.status.success() {
      match String::from_utf8(result.stderr) {
        Ok(stderr) => {
          tracing::error!("Failed removing image because {stderr}");
        }
        Err(err) => {
          tracing::error!(
            "Failed removing image and failed parsing stderr because {err}"
          );
        }
      }
    }
  }
}
