use crate::cache;
use anyhow::{bail, Context};
use gale_core::prelude::*;
use std::{io::BufReader, path::Path, time::Instant};

pub async fn install(
    src: &Path,
    full_name: &str,
    version: &str,
    profile_path: &Path,
    state: &AppState,
) -> Result<()> {
    let start = Instant::now();

    let id = format!("{}-{}", full_name, version);

    let (cache_path, cache_hit) = cache::check(&id, "local", state)
        .await
        .context("failed to check cache")?;

    if !cache_hit {
        let reader = std::fs::File::open(src)
            .map(BufReader::new)
            .context("failed to open package")?;

        match src.extension().and_then(|ext| ext.to_str()) {
            Some("zip") => {
                crate::common::extract(reader, full_name, cache_path.clone())
                    .context("failed to extract package")?;
            }
            Some("dll") => {
                crate::common::cache_dll(
                    reader,
                    full_name,
                    src.file_name().unwrap(),
                    cache_path.clone(),
                )
                .context("failed to cache dll")?;
            }
            _ => bail!("unsupported package format"),
        }
    }

    crate::common::install(&cache_path, profile_path).context("failed to install package")?;

    log::info!(
        "installed {} in {}s (cache {})",
        id,
        start.elapsed().as_secs_f64(),
        if cache_hit { "hit" } else { "miss" }
    );

    Ok(())
}
