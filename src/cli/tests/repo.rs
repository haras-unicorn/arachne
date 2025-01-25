mod common;

#[cfg(test)]
mod tests {
  #[test]
  fn repo_new() -> anyhow::Result<()> {
    let name = "repo_new";
    let repo = super::common::repo::Repo::new(name)?;
    let root = repo.root().to_owned();
    let dir = std::fs::read_dir(&root)?;
    assert!(dir.count() == 1);
    let mut dir = std::fs::read_dir(&root)?;
    assert!(
      dir
        .next()
        .and_then(|f| f.ok())
        .and_then(|f| f.file_name().into_string().ok())
        == Some(".git".to_string())
    );
    std::mem::drop(repo);
    assert!(!std::fs::exists(root).is_ok_and(std::convert::identity));
    Ok(())
  }
}
