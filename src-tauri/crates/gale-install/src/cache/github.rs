use crate::Progress;
use anyhow::Context;
use gale_core::prelude::*;
use serde::Deserialize;
use std::{fmt::Display, io::Cursor, path::PathBuf};

pub async fn insert(
    owner: &str,
    repo: &str,
    tag: &str,
    dest: PathBuf,
    mut on_progress: impl FnMut(Progress),
    state: &AppState,
) -> Result<()> {
    let assets = get_assets(owner, repo, tag, &state.reqwest)
        .await
        .context("failed to retrieve assets")?;

    let (asset, ty) = guess_asset_to_install(&assets).context("no suitable asset found")?;

    let response = state
        .reqwest
        .get(&asset.url)
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .and_then(|res| res.error_for_status())
        .context("failed to download package")?;

    let data = crate::stream_download_res(response, &mut on_progress)
        .await
        .context("error while downloading package")?;

    on_progress(Progress::Extract);

    let reader = Cursor::new(data);
    let package_id = format!("{}-{}", owner, repo);

    match ty {
        AssetType::Zip => {
            crate::common::extract(reader, &package_id, dest)
                .context("failed to extract package")?;
        }
        AssetType::Dll => {
            super::insert_local_dll(reader, &package_id, &asset.name, dest)
                .context("failed to cache dll")?;
        }
    }

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
