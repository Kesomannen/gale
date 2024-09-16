use crate::cache;
use anyhow::Context;
use gale_core::prelude::*;
use serde::Deserialize;
use std::{fmt::Display, path::Path, time::Instant};

pub async fn install(
    owner: &str,
    repo: &str,
    tag: &str,
    profile_path: &Path,
    state: &AppState,
) -> Result<()> {
    let start = Instant::now();

    let full_name = format!("{}-{}", owner, repo);
    let id = format!("{}-{}-{}", owner, repo, tag);

    let (cache_path, cache_hit) = cache::check(&id, "github", state)
        .await
        .context("failed to check cache")?;

    if !cache_hit {
        let assets = get_assets(owner, repo, tag, &state.reqwest)
            .await
            .context("failed to retrieve assets")?;

        let (asset, ty) = guess_asset_to_install(&assets).context("no suitable asset found")?;

        let data = state
            .reqwest
            .get(&asset.url)
            .header("Accept", "application/octet-stream")
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let reader = std::io::Cursor::new(data);

        match ty {
            AssetType::Zip => {
                crate::common::extract(reader, &full_name, cache_path.clone())
                    .context("failed to extract package")?;
            }
            AssetType::Dll => {
                crate::common::cache_dll(reader, &full_name, &asset.name, cache_path.clone())
                    .context("failed to cache dll")?;
            }
        }
    }

    crate::common::install(&cache_path, profile_path).context("failed to install package")?;

    log::info!(
        "installed {} in {}s (cache {})",
        id,
        start.elapsed().as_secs_f32(),
        if cache_hit { "hit" } else { "miss" }
    );

    Ok(())
}

enum AssetType {
    Zip,
    Dll,
}

fn guess_asset_to_install(assets: &[Asset]) -> Option<(&Asset, AssetType)> {
    assets
        .iter()
        .filter_map(|asset| {
            let ext = asset.name.split('.').last()?;
            match ext {
                "zip" => Some((asset, AssetType::Zip)),
                "dll" => Some((asset, AssetType::Dll)),
                _ => None,
            }
        })
        .max_by_key(|(asset, _)| asset.download_count)
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    url: String,
    size: u64,
    download_count: u64,
}

async fn get_assets(
    owner: &str,
    repo: &str,
    tag: &str,
    client: &reqwest::Client,
) -> Result<Vec<Asset>> {
    #[derive(Debug, Deserialize)]
    struct Response {
        assets: Vec<Asset>,
    }

    let url = format!("repos/{}/{}/releases/tags/{}", owner, repo, tag);

    let response = github_request(url, reqwest::Method::GET, client)
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?;

    Ok(response.assets)
}

fn github_request(
    tail: impl Display,
    method: reqwest::Method,
    client: &reqwest::Client,
) -> reqwest::RequestBuilder {
    let url = format!("https://api.github.com/{}", tail);
    client
        .request(method, &url)
        .header("Accept", "application/vnd.github+json")
        .header("X-Github-Api-Version", "2022-11-28")
}
