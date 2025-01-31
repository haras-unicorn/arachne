mod common;

#[cfg(test)]
mod tests {
  #[test]
  fn container_new() -> anyhow::Result<()> {
    let _ = super::common::container::Container::new()?;
    assert!(true);
    Ok(())
  }
}
