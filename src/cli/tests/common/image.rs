const NAME: &str = env!("CARGO_PKG_NAME");

pub(crate) struct Image {
  nix_path: std::path::PathBuf,
}

impl Image {
  pub(crate) async fn new() -> anyhow::Result<Self> {
    let nix_path: std::path::PathBuf =
      std::env::var(NAME.to_uppercase() + "_NIX_PATH")?.try_into()?;
    Ok(Self { nix_path })
  }
}
