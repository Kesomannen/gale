use std::path::Path;

use eyre::Context;

pub trait IoResultExt<T> {
    fn fs_context(self, op: &str, path: &Path) -> eyre::Result<T>;
}

impl<T, E> IoResultExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn fs_context(self, op: &str, path: &Path) -> eyre::Result<T> {
        self.with_context(|| format!("error {} (at {})", op, path.display()))
    }
}
