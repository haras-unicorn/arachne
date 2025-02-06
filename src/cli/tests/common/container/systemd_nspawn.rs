use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::process::Command;

pub struct SystemdNspawnContainer {
  pub name: String,
  pub artifact: PathBuf,
}

impl SystemdNspawnContainer {
  /// Create a new container instance from a NixOS artifact.
  pub fn new(name: &str, artifact: PathBuf) -> Self {
    Self {
      name: name.to_owned(),
      artifact,
    }
  }

  /// Start the container using systemd-nspawn in rootless mode.
  /// (Assumes that your subuid/subgid mappings are properly set up.)
  pub async fn start(&self) -> Result<()> {
    let artifact_str = self
      .artifact
      .to_str()
      .context("Artifact path must be valid UTF-8")?;
    // Build the basic systemd-nspawn command.
    let mut cmd = Command::new("systemd-nspawn");
    cmd
      .arg("--quiet")
      .arg("--directory")
      .arg(artifact_str)
      .arg("--machine")
      .arg(&self.name)
      // Use private user namespace: adjust the range as needed.
      .arg("--private-users=100000-165535")
      // Boot the container (systemd-nspawn will execute the init system).
      .arg("--boot");

    // If you want to attach additional arguments (for networking, logging, etc.),
    // you can do so here.
    let status = cmd
      .status()
      .await
      .context("Failed to start systemd-nspawn container")?;
    if !status.success() {
      anyhow::bail!("systemd-nspawn exited with non-zero status: {}", status);
    }
    Ok(())
  }

  /// Stop the container. This example uses machinectl to terminate the container.
  pub async fn stop(&self) -> Result<()> {
    let status = Command::new("machinectl")
      .arg("terminate")
      .arg(&self.name)
      .status()
      .await
      .context("Failed to stop systemd-nspawn container")?;
    if !status.success() {
      anyhow::bail!(
        "Failed to stop container with machinectl: status {}",
        status
      );
    }
    Ok(())
  }
}
