pub use {
    github::install as from_github, local::install as from_local,
    thunderstore::install as from_thunderstore,
};

mod cache;
mod common;
mod github;
mod local;
mod thunderstore;
