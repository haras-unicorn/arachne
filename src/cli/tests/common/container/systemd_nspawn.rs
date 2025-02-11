const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub struct SystemdNspawnContainer {
  pub name: String,
  pub artifact: std::path::PathBuf,
  pub root: std::path::PathBuf,
  pub profiles: std::path::PathBuf,
  pub gcroots: std::path::PathBuf,
  child: Option<tokio::process::Child>,
}

impl SystemdNspawnContainer {
  pub async fn new(
    name: &str,
    artifact: std::path::PathBuf,
  ) -> anyhow::Result<Self> {
    let tmp: std::path::PathBuf =
      std::env::var(CARGO_PKG_NAME.to_uppercase() + "_TEST_TMP")?.into();
    let containers = tmp.join("containers");

    let container = containers.join(name);
    let root = container.join("root");
    let profiles = container.join("profiles");
    let gcroots = container.join("gcroots");

    anyhow::Context::context(
      tokio::fs::create_dir_all(&root).await,
      "Failed to create container root directory",
    )?;
    anyhow::Context::context(
      tokio::fs::create_dir_all(&profiles).await,
      "Failed to create container profile directory",
    )?;
    anyhow::Context::context(
      tokio::fs::create_dir_all(&gcroots).await,
      "Failed to create container gcroots directory",
    )?;

    let system_profile = profiles.join("system");
    let status = anyhow::Context::context(
      tokio::process::Command::new("nix-env")
        .arg("-p")
        .arg(system_profile.to_str().unwrap())
        .arg("--set")
        .arg(artifact.to_str().unwrap())
        .status()
        .await,
      "Failed to run nix-env to set container system profile",
    )?;
    if !status.success() {
      anyhow::bail!("nix-env failed with status: {}", status);
    }

    let mut container = Self {
      name: name.to_owned(),
      artifact,
      root,
      profiles,
      gcroots,
      child: None,
    };

    container.start().await?;
    Ok(container)
  }

  pub async fn start(&mut self) -> anyhow::Result<()> {
    let system_path = self.profiles.join("system");

    let mut cmd = tokio::process::Command::new("systemd-nspawn");
    cmd
      .arg("--quiet")
      .arg("--directory")
      .arg(self.root.to_str().unwrap())
      .arg("--machine")
      .arg(&self.name)
      .arg("--private-users=yes")
      .arg("--notify-ready=yes")
      .arg("--boot")
      .env("SYSTEM_PATH", system_path.to_str().unwrap())
      .arg("--bind-ro=/nix/store")
      .arg("--bind-ro=/nix/var/nix/db")
      .arg("--bind-ro=/nix/var/nix/daemon-socket")
      .arg(format!(
        "--bind={}:{}",
        self.profiles.join("system").to_str().unwrap(),
        "/nix/var/nix/profiles"
      ))
      .arg(format!(
        "--bind={}:{}",
        self.gcroots.to_str().unwrap(),
        "/nix/var/nix/gcroots"
      ));

    let child = anyhow::Context::context(
      cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn(),
      "Failed to spawn systemd-nspawn container",
    )?;
    self.child = Some(child);
    Ok(())
  }

  pub async fn stop(&mut self) -> anyhow::Result<()> {
    let status = anyhow::Context::context(
      tokio::process::Command::new("machinectl")
        .arg("terminate")
        .arg(&self.name)
        .status()
        .await,
      "Failed to terminate container with machinectl",
    )?;
    if !status.success() {
      anyhow::bail!("machinectl terminate exited with status: {}", status);
    }
    self.child = None;
    Ok(())
  }
}

impl Drop for SystemdNspawnContainer {
  fn drop(&mut self) {
    let name = self.name.clone();
    let container_dir = self.root.parent().unwrap().to_owned();
    futures::executor::block_on(async move {
      if let Err(e) = tokio::process::Command::new("machinectl")
        .arg("terminate")
        .arg(&name)
        .status()
        .await
      {
        tracing::error!("Failed to stop container {}: {}", name, e);
      }
      if let Err(e) = tokio::fs::remove_dir_all(&container_dir).await {
        tracing::error!(
          "Failed to remove container directory {}: {}",
          container_dir.display(),
          e
        );
      }
    });
  }
}
