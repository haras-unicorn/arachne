// TODO: pure

use itertools::Itertools;

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

lazy_static::lazy_static! {
  static ref BASE: String = format!(
    r#"
      pkgs.dockerTools.buildImage {{
        name = "{CARGO_PKG_NAME}/base";
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

pub struct Image {
  nix_path: std::path::PathBuf,
  docker_path: std::path::PathBuf,
  artifact: std::path::PathBuf,
  user: String,
  repo: String,
  name: String,
  tag: String,
}

impl Image {
  pub(crate) async fn new<TPkgs, TCmd: std::fmt::Display>(
    repo: &str,
    pkgs: TPkgs,
    cmd: TCmd,
  ) -> anyhow::Result<Self>
  where
    TPkgs: IntoIterator,
    TPkgs::Item: std::fmt::Display,
  {
    let nix_path: std::path::PathBuf =
      std::env::var(CARGO_PKG_NAME.to_uppercase() + "_TEST_NIX_PATH")?.into();
    let docker_path: std::path::PathBuf =
      std::env::var(CARGO_PKG_NAME.to_uppercase() + "_TEST_DOCKER_PATH")?
        .into();

    let pkgs = pkgs.into_iter().join(" ");
    let tag = "latest";
    let base: String = BASE.to_string();
    let spec = format!(
      r#"
        let
          pkgs = import <nixpkgs> {{ }};

          base = {base};

          cmd = pkgs.writeShellApplication {{
            name = "cmd";
            runtimeInputs = with pkgs; [ {} ];
            text = ''
              {}
            '';
          }};
        in
        pkgs.dockerTools.buildImage {{
          name = "{CARGO_PKG_NAME}/{repo}";
          tag = "{tag}";
          created = "now";
          fromImage = base;
          copyToRoot = pkgs.buildEnv {{
            name = "image-root";
            paths = with pkgs; [ cmd ];
            pathsToLink = [ "/bin" ];
          }};
          config = {{
            Cmd = [ "cmd" ];
          }};
        }}
      "#,
      pkgs, cmd,
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
    let load = tokio::process::Command::new(docker_path.as_os_str())
      .arg("load")
      .arg("--input")
      .arg(artifact.as_os_str())
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
    tracing::info!("Loaded image {repo}");

    Ok(Self {
      nix_path,
      docker_path,
      artifact,
      user: CARGO_PKG_NAME.to_owned(),
      repo: repo.to_owned(),
      name: format!("{}/{}", CARGO_PKG_NAME, repo),
      tag: tag.to_owned(),
    })
  }

  pub(crate) fn artifact(&self) -> &std::path::Path {
    return self.artifact.as_path();
  }

  pub(crate) fn repo(&self) -> &str {
    &self.repo
  }

  pub(crate) fn user(&self) -> &str {
    &self.user
  }

  pub(crate) fn name(&self) -> &str {
    &self.name
  }

  pub(crate) fn tag(&self) -> &str {
    &self.tag
  }
}

impl Drop for Image {
  fn drop(&mut self) {
    let result = match std::process::Command::new(self.docker_path.as_os_str())
      .arg("image")
      .arg("rm")
      .arg(format!("{}/{}", self.user, self.repo))
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
