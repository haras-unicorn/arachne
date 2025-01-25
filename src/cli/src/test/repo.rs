const NAME: &str = env!("CARGO_PKG_NAME");

pub(crate) struct Repo {
  root: std::path::PathBuf,
  inner: git2::Repository,
}

impl Repo {
  pub(crate) async fn new(name: &str) -> anyhow::Result<Self> {
    let root: std::path::PathBuf =
      std::env::var(NAME.to_uppercase() + "_TEST_TEMP")? + name;
    if std::fs::exists(root)? {
      std::fs::remove_dir_all(root)?;
    }
    std::fs::create_dir_all(root)?;
    let inner = git2::Repository::init(root)?;
    Ok(Self { root, inner })
  }

  pub(crate) fn root(&self) -> &Path {
    &self.root
  }
}

impl Drop for Repo {
  fn drop(&mut self) {
    if let Err(err) = std::fs::remove_dir_all(self.root) {
      eprintln!(format!("Failed removing repo at {self.root}"));
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn repo_new() -> anyhow::Result<()> {
    let name = "repo_new";
    let mut repo = Repo::new(name);
    let root = repo.root().to_owned();
    let dir = std::fs::read_dir(root)?;
    assert!(dir.count() == 1);
    let dir = std::fs::read_dir(root)?;
    assert!(
      dir
        .next()
        .and_then(|f| f.ok())
        .and_then(|f| f.file_name().to_str())
        == Some(".git")
    );
    std::mem::drop(repo);
    assert(!std::fs::exists(root).is_some_and(std::convert::identity));
    Ok(())
  }
}
