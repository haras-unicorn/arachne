#![forbid(rust_2018_idioms, unsafe_code, missing_docs)]
#![deny(clippy::all, clippy::perf, clippy::nursery, clippy::pedantic)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]
#![allow(dead_code)]

mod common;

#[cfg(test)]
mod tests {
  #[tracing_test::traced_test]
  #[tokio::test]
  async fn repository_new() -> anyhow::Result<()> {
    let name = "repo-new";
    let repo = super::common::repository::Repository::new(name).await?;
    let root = repo.root().to_owned();
    let dir = std::fs::read_dir(&root)?;
    assert!(dir.count() == 1);
    let mut dir = tokio::fs::read_dir(&root).await?;
    assert!(
      dir
        .next_entry()
        .await
        .ok()
        .flatten()
        .and_then(|f| f.file_name().into_string().ok())
        == Some(".git".to_string())
    );
    std::mem::drop(repo);
    assert!(!std::fs::exists(root).is_ok_and(std::convert::identity));
    Ok(())
  }
}
