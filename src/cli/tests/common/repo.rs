const NAME: &str = env!("CARGO_PKG_NAME");

pub(crate) struct Repo {
  root: std::path::PathBuf,
  #[allow(dead_code)]
  inner: git2::Repository,
}

impl Repo {
  pub(crate) fn new(name: &str) -> anyhow::Result<Self> {
    let root: std::path::PathBuf =
      (std::env::var(NAME.to_uppercase() + "_TEST_TEMP")? + name).into();
    if std::fs::exists(&root)? {
      std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir_all(&root)?;
    let inner = git2::Repository::init(&root)?;
    Ok(Self { root, inner })
  }

  pub(crate) fn root(&self) -> &std::path::Path {
    &self.root
  }
}

impl Drop for Repo {
  fn drop(&mut self) {
    if let Err(err) = std::fs::remove_dir_all(&self.root) {
      eprintln!("Failed removing repo at {:?} because {:?}", self.root, err);
    }
  }
}
