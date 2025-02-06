use anyhow::{Context, Result};
use tokio::process::Command;

pub struct SlirpNetwork {
  // The PID of the container process that slirp4netns will attach to.
  pub container_pid: u32,
  // A name for the virtual network interface (tap) that slirp4netns will manage.
  pub tap_name: String,
  // The IP address for the host side of the network (as seen on the virtual network).
  pub host_ip: String,
  // The IP address for the container side.
  pub container_ip: String,
}

impl SlirpNetwork {
  /// Create a new SlirpNetwork instance.
  pub fn new(
    container_pid: u32,
    tap_name: &str,
    host_ip: &str,
    container_ip: &str,
  ) -> Self {
    Self {
      container_pid,
      tap_name: tap_name.to_owned(),
      host_ip: host_ip.to_owned(),
      container_ip: container_ip.to_owned(),
    }
  }

  /// Start the slirp4netns process to configure networking.
  /// This attaches the container’s network namespace to a user-space network.
  pub async fn start(&self) -> Result<()> {
    // Example: slirp4netns <container_pid> <tap_name> --configure --mtu 65520
    let status = Command::new("slirp4netns")
      .arg(self.container_pid.to_string())
      .arg(&self.tap_name)
      .arg("--configure")
      .arg("--mtu")
      .arg("65520")
      .status()
      .await
      .context("Failed to start slirp4netns")?;
    if !status.success() {
      anyhow::bail!("slirp4netns exited with non-zero status: {}", status);
    }
    Ok(())
  }

  /// Optionally forward a port from the host to the container.
  pub async fn forward_port(
    &self,
    host_port: u16,
    container_port: u16,
  ) -> Result<()> {
    // Depending on your setup, you might call an external command or adjust iptables rules.
    // This is a placeholder for your port forwarding logic.
    println!(
      "Forwarding host port {} to container port {}",
      host_port, container_port
    );
    Ok(())
  }
}
