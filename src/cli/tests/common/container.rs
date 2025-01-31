pub(crate) struct Container {
  inner: testcontainers::ContainerAsync<testcontainers::GenericImage>,
}

impl Container {
  pub(crate) async fn new() -> anyhow::Result<Self> {
    let inner = testcontainers::runners::AsyncRunner::start(
      testcontainers::GenericImage::new("alpine", "latest"),
    )
    .await?;
    Ok(Self { inner })
  }
}
