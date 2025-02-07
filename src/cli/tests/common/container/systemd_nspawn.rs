pub struct SystemdNspawnContainer {
  pub name: String,
  pub artifact: std::path::PathBuf,
  // Optionally track the spawned container process.
  child: Option<tokio::process::Child>,
}

impl SystemdNspawnContainer {
  pub async fn new(
    name: &str,
    artifact: std::path::PathBuf,
  ) -> anyhow::Result<Self> {
    let mut container = Self {
      name: name.to_owned(),
      artifact,
      child: None,
    };
    container.start().await?;
    Ok(container)
  }

  pub async fn start(&mut self) -> anyhow::Result<()> {
    let child = anyhow::Context::context(
      tokio::process::Command::new("systemd-nspawn")
        .arg("--quiet")
        .arg("--directory")
        .arg(self.artifact.as_os_str())
        .arg("--machine")
        .arg(&self.name)
        .arg("--private-users=yes")
        .arg("--ephemeral")
        .arg("--notify-ready=yes")
        .arg("--boot")
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
      "Failed to stop container",
    )?;
    if !status.success() {
      anyhow::bail!("machinectl terminated container with status: {}", status);
    }
    self.child = None;
    Ok(())
  }

  pub async fn exec(&self, cmd_args: &[&str]) -> anyhow::Result<()> {
    let pid = self.get_pid().await?;
    let status = anyhow::Context::context(
      tokio::process::Command::new("nsenter")
        .arg("-t")
        .arg(pid.to_string())
        .arg("-m")
        .arg("-u")
        .arg("-i")
        .arg("-n")
        .arg("-p")
        .arg("--")
        .args(cmd_args)
        .status()
        .await,
      "Failed to exec in container",
    )?;
    if !status.success() {
      anyhow::bail!(
        "Exec command {:?} failed with status: {}",
        cmd_args,
        status
      );
    }
    Ok(())
  }

  pub async fn get_pid(&self) -> anyhow::Result<u32> {
    let output = anyhow::Context::context(
      tokio::process::Command::new("machinectl")
        .arg("show")
        .arg(&self.name)
        .arg("-p")
        .arg("Leader")
        .output()
        .await,
      "Failed to get container PID using machinectl",
    )?;
    if !output.status.success() {
      anyhow::bail!("machinectl show failed");
    }
    let stdout = String::from_utf8(output.stdout)?;
    let trimmed = stdout.trim();
    if trimmed.starts_with("Leader=") {
      let pid_str = trimmed.trim_start_matches("Leader=");
      let pid: u32 =
        anyhow::Context::context(pid_str.parse(), "Failed to parse PID")?;
      Ok(pid)
    } else {
      anyhow::bail!("Unexpected output from machinectl show: {}", stdout);
    }
  }
}

impl Drop for SystemdNspawnContainer {
  fn drop(&mut self) {
    if self.child.is_some() {
      let name = self.name.clone();
      tokio::spawn(async move {
        let _ = tokio::process::Command::new("machinectl")
          .arg("terminate")
          .arg(&name)
          .status()
          .await;
      });
    }
  }
}
