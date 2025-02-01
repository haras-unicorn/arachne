const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub struct Repository {
  root: std::path::PathBuf,
  inner: git2::Repository,
}

impl Repository {
  pub(crate) async fn new(name: &str) -> anyhow::Result<Self> {
    let tmp: std::path::PathBuf =
      std::env::var(CARGO_PKG_NAME.to_uppercase() + "_TEST_TEMP")?.into();
    let root: std::path::PathBuf = tmp.join(name);
    if tokio::fs::try_exists(&root).await? {
      tokio::fs::remove_dir_all(&root).await?;
    }
    tokio::fs::create_dir_all(&root).await?;
    let inner = git2::Repository::init(&root)?;
    tracing::info!("Created repository at {root:?}");
    Ok(Self { root, inner })
  }

  pub(crate) fn root(&self) -> &std::path::Path {
    &self.root
  }
}

impl Drop for Repository {
  fn drop(&mut self) {
    if let Err(err) = std::fs::remove_dir_all(&self.root) {
      tracing::error!(
        "Failed removing repo at {:?} because {:?}",
        self.root,
        err
      );
    }
  }
}
