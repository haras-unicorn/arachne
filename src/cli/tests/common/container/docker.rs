pub struct DockerContainer {
  inner: std::sync::Arc<
    tokio::sync::Mutex<
      Option<testcontainers::ContainerAsync<testcontainers::GenericImage>>,
    >,
  >,
}

impl DockerContainer {
  pub(crate) async fn new(name: &str, tag: &str) -> anyhow::Result<Self> {
    let inner = testcontainers::runners::AsyncRunner::start(
      testcontainers::GenericImage::new(name, tag),
    )
    .await?;
    tracing::info!("Started container {}", inner.id());
    Ok(Self {
      inner: std::sync::Arc::new(tokio::sync::Mutex::new(Some(inner))),
    })
  }

  pub(crate) async fn id(&self) -> anyhow::Result<String> {
    let inner = self.inner.clone().lock_owned().await;
    if let Some(inner) = &*inner {
      Ok(inner.id().to_owned())
    } else {
      Err(anyhow::anyhow!("Container dropped"))
    }
  }

  pub(crate) async fn host(&self) -> anyhow::Result<std::net::IpAddr> {
    let inner = self.inner.clone().lock_owned().await;
    if let Some(inner) = &*inner {
      let result = inner.get_bridge_ip_address().await?;
      Ok(result)
    } else {
      Err(anyhow::anyhow!("Container dropped"))
    }
  }

  pub(crate) async fn start(&self) -> anyhow::Result<()> {
    let inner = self.inner.clone().lock_owned().await;
    if let Some(inner) = &*inner {
      inner.start().await?;
      Ok(())
    } else {
      Err(anyhow::anyhow!("Container dropped"))
    }
  }

  pub(crate) async fn stop(&self) -> anyhow::Result<()> {
    let inner = self.inner.clone().lock_owned().await;
    if let Some(inner) = &*inner {
      inner.stop().await?;
      Ok(())
    } else {
      Err(anyhow::anyhow!("Container dropped"))
    }
  }

  pub(crate) async fn output(&self) -> anyhow::Result<String> {
    let inner = self.inner.clone().lock_owned().await;
    if let Some(inner) = &*inner {
      let mut result = String::new();
      tokio::io::AsyncReadExt::read_to_string(
        &mut inner.stdout(false),
        &mut result,
      )
      .await?;
      Ok(result)
    } else {
      Err(anyhow::anyhow!("Container dropped"))
    }
  }

  pub(crate) async fn wait(&self, marker: &str) -> anyhow::Result<()> {
    let inner = self.inner.clone().lock_owned().await;
    let inner = match &*inner {
      Some(inner) => inner,
      None => return Err(anyhow::anyhow!("Continer droppped")),
    };

    let marker_bytes = marker.as_bytes();
    let marker_len = marker_bytes.len();
    let mut window = vec![0u8; marker_len * 2];
    let window_len = window.len();
    let mut buf = vec![0u8; marker_len];

    let mut reader = inner.stdout(true);
    loop {
      let read = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await?;
      tracing::trace!(
        "Read '{}' from container {}",
        buf
          .iter()
          .take(read)
          .map(|x| x.escape_ascii())
          .flatten()
          .map(|x| x as char)
          .collect::<String>(),
        inner.id()
      );
      if read == 0 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
      }
      window.copy_within(read..window_len, 0);
      window
        .split_at_mut(window_len - read)
        .1
        .copy_from_slice(buf.split_at(read).0);
      if window
        .windows(marker.len())
        .any(|window| window == marker_bytes)
      {
        break;
      }
    }
    Ok(())
  }
}

impl Drop for DockerContainer {
  fn drop(&mut self) {
    futures::executor::block_on(async {
      let mut inner = self.inner.clone().lock_owned().await;
      let inner = match inner.take() {
        Some(inner) => inner,
        None => {
          tracing::debug!("Attempted to drop dropped container");
          return;
        }
      };
      tracing::info!("Removing container {}", inner.id());
      if let Err(err) = inner.stop().await {
        tracing::error!("Failed stopping container because {}", err);
      }
      if let Err(err) = inner.rm().await {
        tracing::error!("Failed removing container because {}", err);
      }
    });
  }
}
