#![forbid(rust_2018_idioms, unsafe_code, missing_docs)]
#![deny(clippy::all, clippy::perf, clippy::nursery, clippy::pedantic)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]
#![allow(dead_code, reason = "because other modules in common get flagged")]

mod common;

#[cfg(test)]
mod tests {
  #[tracing_test::traced_test]
  #[tokio::test(flavor = "multi_thread")]
  async fn docker_container_new() -> anyhow::Result<()> {
    let container = super::common::container::docker::DockerContainer::new(
      "alpine", "latest",
    )
    .await?;
    let id = container.id().await?;
    assert!(!id.is_empty());
    Ok(())
  }

  #[tracing_test::traced_test]
  #[tokio::test(flavor = "multi_thread")]
  async fn systemd_nspawn_container_new() -> anyhow::Result<()> {
    let name = "nixos-image-new";
    let image = super::common::image::nixos::NixosImage::new(
      name,
      &["{ pkgs, ... }: { environment.systemPackages = [ pkgs.hello ]; }"],
    )
    .await?;

    let container =
      super::common::container::systemd_nspawn::SystemdNspawnContainer::new(
        name,
        image.artifact().to_path_buf(),
      )
      .await?;
    container.status().await?;
    Ok(())
  }
}
