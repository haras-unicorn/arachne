pub struct Container {
  inner: testcontainers::ContainerAsync<testcontainers::GenericImage>,
}

impl Container {
  pub(crate) async fn new(name: &str, tag: &str) -> anyhow::Result<Self> {
    let inner = testcontainers::runners::AsyncRunner::start(
      testcontainers::GenericImage::new(name, tag),
    )
    .await?;
    tracing::info!("Created container {name}/{tag}");
    Ok(Self { inner })
  }
}
