use anyhow::{bail, Context};
use gale_core::prelude::*;
use gale_thunderstore::api::VersionId;
use std::{
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
};

mod github;
use crate::Progress;
pub use github::insert as insert_github;

pub async fn root(state: &AppState) -> Result<PathBuf> {
    let cache_path = sqlx::query!("SELECT cache_path FROM settings")
        .fetch_one(&state.db)
        .await?
        .cache_path;

    Ok(cache_path.into())
}

pub async fn check(
    version_id: impl AsRef<Path>,
    subdir: impl AsRef<Path>,
    state: &AppState,
) -> Result<(PathBuf, bool)> {
    let mut path = root(state).await?;
    path.push(subdir);
    path.push(version_id);

    let exists = path.exists();
    Ok((path, exists))
}

pub async fn insert_thunderstore<F>(
    id: &VersionId,
    dest: PathBuf,
    mut on_progress: F,
    state: &AppState,
) -> Result<()>
where
    F: FnMut(Progress),
{
    let (len, stream) = gale_thunderstore::api::download(&state.reqwest, id)
        .await
        .context("failed to download package")?;

    let data = super::stream_download(len, &mut on_progress, stream)
        .await
        .context("error while downloading package")?;

    on_progress(Progress::Extract);

    crate::common::extract(Cursor::new(data), id.full_name(), dest)
        .context("failed to extract package")?;

    Ok(())
}

pub async fn insert_local(
    src: &Path,
    dest: PathBuf,
    package_id: &str,
    mut on_progress: impl FnMut(Progress),
) -> Result<()> {
    on_progress(Progress::Extract);

    let reader = std::fs::File::open(src)
        .map(BufReader::new)
        .context("failed to open package")?;

    match src.extension().and_then(|ext| ext.to_str()) {
        Some("zip") => {
            crate::common::extract(reader, package_id, dest)
                .context("failed to extract package")?;
        }
        Some("dll") => {
            insert_local_dll(reader, package_id, src.file_name().unwrap(), dest)
                .context("failed to cache dll")?;
        }
        _ => bail!("unsupported package format"),
    }

    Ok(())
}

fn insert_local_dll(
    mut src: impl Read,
    package_id: impl AsRef<Path>,
    file_name: impl AsRef<Path>,
    mut dest: PathBuf,
) -> Result<()> {
    dest.push("BepInEx");
    dest.push("plugins");
    dest.push(package_id);
    std::fs::create_dir_all(&dest)?;

    dest.push(file_name);
    let mut target_file = std::fs::File::create(&dest)?;
    std::io::copy(&mut src, &mut target_file)?;

    Ok(())
}
