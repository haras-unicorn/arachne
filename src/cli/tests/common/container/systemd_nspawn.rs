const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub struct SystemdNspawnContainer {
  pub name: String,
  pub artifact: std::path::PathBuf,
  pub root: std::path::PathBuf,
  pub profiles: std::path::PathBuf,
  pub gcroots: std::path::PathBuf,
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

    let container = Self {
      name: name.to_owned(),
      artifact,
      root,
      profiles,
      gcroots,
    };

    container.start().await?;
    Ok(container)
  }

  pub async fn start(&self) -> anyhow::Result<()> {
    CapabilityGuard::new(caps::Capability::CAP_SYS_ADMIN)?;

    let mut cmd = std::process::Command::new("systemd-nspawn");
    cmd
      .arg("--machine")
      .arg(&self.name)
      .arg("--directory")
      .arg(self.root.to_str().unwrap())
      .arg("--keep-unit")
      .arg("--private-users=pick")
      .arg("--private-users-ownership=auto")
      .arg("--private-network")
      .arg("--notify-ready=yes")
      .arg("--boot")
      .arg("--bind-ro=/nix/store:/nix/store:idmap")
      .arg("--bind-ro=/nix/var/nix/db:/nix/var/nix/db:idmap")
      .arg("--bind-ro=/nix/var/nix/db:/nix/var/nix/daemon-socket:idmap")
      .arg(format!(
        "--bind={}:{}:idmap",
        self.profiles.to_str().unwrap(),
        "/nix/var/nix/profiles"
      ))
      .arg(format!(
        "--bind={}:{}:idmap",
        self.gcroots.to_str().unwrap(),
        "/nix/var/nix/gcroots"
      ))
      .arg("/nix/var/nix/profiles/system/init");

    let status = anyhow::Context::context(
      cmd.status(),
      "Failed to spawn systemd-nspawn container",
    )?;
    if !status.success() {
      anyhow::bail!("Failed starting container with status: {}", status);
    }

    Ok(())
  }

  pub async fn status(&self) -> anyhow::Result<String> {
    let output = anyhow::Context::context(
      tokio::process::Command::new("machinectl")
        .arg("status")
        .arg(&self.name)
        .output()
        .await,
      "Failed to get status of container with machinectl",
    )?;
    if !output.status.success() {
      anyhow::bail!("machinectl status exited with status: {}", output.status);
    }
    Ok(String::from_utf8(output.stdout)?)
  }

  pub async fn stop(&self) -> anyhow::Result<()> {
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

struct CapabilityGuard {
  capability: caps::Capability,
}

impl CapabilityGuard {
  fn new(capability: caps::Capability) -> anyhow::Result<Self> {
    if !caps::has_cap(None, caps::CapSet::Permitted, capability)? {
      println!(
        "Permitted caps: {:?}",
        caps::read(None, caps::CapSet::Permitted)
      );
      println!(
        "Effective caps: {:?}",
        caps::read(None, caps::CapSet::Effective)
      );
      println!(
        "Inheritable: {:?}",
        caps::read(None, caps::CapSet::Inheritable)
      );
      anyhow::bail!("Binary needs {} capability", capability);
    }
    caps::raise(None, caps::CapSet::Effective, capability)?;
    Ok(Self { capability })
  }
}

impl Drop for CapabilityGuard {
  fn drop(&mut self) {
    if let Err(err) = caps::drop(None, caps::CapSet::Effective, self.capability)
    {
      tracing::error!(
        "Failed dropping capability {} because {}",
        self.capability,
        err,
      );
    }
  }
}
