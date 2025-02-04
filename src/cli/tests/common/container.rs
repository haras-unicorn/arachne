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

  pub(crate) fn id(&self) -> &str {
    return self.inner.id();
  }

  pub(crate) async fn host(&self) -> anyhow::Result<std::net::IpAddr> {
    #[allow(clippy::todo, reason = "Implement via hickory_dns")]
    return match self.inner.get_host().await? {
      url::Host::Domain(_) => todo!("Domain container host not implemented"),
      url::Host::Ipv4(v4) => Ok(std::net::IpAddr::V4(v4)),
      url::Host::Ipv6(v6) => Ok(std::net::IpAddr::V6(v6)),
    };
  }
}
