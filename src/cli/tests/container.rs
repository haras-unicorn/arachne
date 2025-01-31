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
  #[tokio::test]
  async fn container_new() -> anyhow::Result<()> {
    let _ = super::common::container::Container::new().await?;
    assert!(true);
    Ok(())
  }
}
