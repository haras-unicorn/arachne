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
  async fn docker_image_container_run() -> anyhow::Result<()> {
    let name = "docker-image-container-run";
    let image = super::common::image::docker::DockerImage::new(
      name,
      &["hello"],
      r#"
        echo "Hello, world!"
      "#,
    )
    .await?;
    let container = super::common::container::docker::DockerContainer::new(
      image.name(),
      image.tag(),
    )
    .await?;
    let output = container.output().await?;
    assert!(output == "Hello, world!\n");
    Ok(())
  }

  #[tracing_test::traced_test]
  #[tokio::test(flavor = "multi_thread")]
  async fn docker_image_docker_container_wait() -> anyhow::Result<()> {
    let name = "docker-image-container-run";
    let timeout_s = 5;
    let image = super::common::image::docker::DockerImage::new(
      name,
      &["coreutils", "hello"],
      format!(
        r#"
          sleep {}s
          hello
        "#,
        timeout_s
      ),
    )
    .await?;
    let container = super::common::container::docker::DockerContainer::new(
      image.name(),
      image.tag(),
    )
    .await?;
    tokio::time::timeout(
      std::time::Duration::from_secs(timeout_s * 2),
      container.wait("Hello, world!"),
    )
    .await??;
    Ok(())
  }
}
