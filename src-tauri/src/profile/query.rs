use std::{cmp::Ordering, path::PathBuf};

use chrono::{DateTime, Utc};
use eyre::Result;
use serde::Serialize;
use tracing::warn;
use uuid::Uuid;

use super::{Dependant, LocalMod, Profile, ProfileMod, ProfileModKind};
use crate::thunderstore::{
    self, Backend, BorrowedMod, FrontendMod, Thunderstore,
    query::{QueryModsArgs, Queryable, SortBy, SortOrder},
};

struct QueryableProfileMod<'a> {
    enabled: bool,
    install_time: DateTime<Utc>,
    kind: QueryableProfileModKind<'a>,
    index: usize,
}

enum QueryableProfileModKind<'a> {
    Local(&'a LocalMod),
    Thunderstore(BorrowedMod<'a>),
}

impl<'a> QueryableProfileMod<'a> {
    fn create(
        profile_mod: &'a ProfileMod,
        index: usize,
        thunderstore: &'a Thunderstore,
    ) -> Result<QueryableProfileMod<'a>> {
        let kind = match &profile_mod.kind {
            ProfileModKind::Local(local) => QueryableProfileModKind::Local(local),
            ProfileModKind::Thunderstore(ts_mod) => {
                let borrow = ts_mod.id.borrow(thunderstore)?;
                QueryableProfileModKind::Thunderstore(borrow)
            }
        };

        Ok(QueryableProfileMod {
            enabled: profile_mod.enabled,
            install_time: profile_mod.install_time,
            kind,
            index,
        })
    }
}

impl Queryable for QueryableProfileMod<'_> {
    fn uuid(&self) -> Uuid {
        match &self.kind {
            QueryableProfileModKind::Local(local) => local.uuid(),
            QueryableProfileModKind::Thunderstore(remote) => remote.uuid(),
        }
    }

    fn full_name(&self) -> &str {
        use QueryableProfileModKind as Kind;

        match &self.kind {
            Kind::Local(local) => local.full_name(),
            Kind::Thunderstore(remote) => remote.full_name(),
        }
    }

    fn version(&self) -> Option<semver::Version> {
        use QueryableProfileModKind as Kind;

        match &self.kind {
            Kind::Local(local) => local.version(),
            Kind::Thunderstore(remote) => Some(remote.version.parsed_version()),
        }
    }

    fn matches(&self, args: &QueryModsArgs) -> bool {
        use QueryableProfileModKind as Kind;

        if !args.include_disabled && !self.enabled {
            return false;
        }

        if !args.include_enabled && self.enabled {
            return false;
        }

        match &self.kind {
            Kind::Local(local) => local.matches(args),
            Kind::Thunderstore(remote) => remote.matches(args),
        }
    }

    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering {
        use QueryableProfileModKind as Kind;

        let overridden = match args.sort_by {
            SortBy::InstallDate => Some(self.install_time.cmp(&other.install_time)),
            SortBy::Custom => Some(self.index.cmp(&other.index)),
            _ => None,
        };

        if let Some(order) = overridden {
            return match args.sort_order {
                SortOrder::Ascending => order,
                SortOrder::Descending => order.reverse(),
            };
        }

        match (&self.kind, &other.kind) {
            (Kind::Thunderstore(a), Kind::Thunderstore(b)) => a.cmp(b, args),
            (Kind::Local(a), Kind::Local(b)) => a.cmp(b, args),
            (Kind::Local(_), _) => Ordering::Less,
            (_, Kind::Local(_)) => Ordering::Greater,
        }
    }

    fn backend(&self) -> thunderstore::Backend {
        use QueryableProfileModKind as Kind;

        match &self.kind {
            Kind::Local(local) => local.backend(),
            Kind::Thunderstore(remote) => remote.backend(),
        }
    }
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FrontendProfileMod {
    pub enabled: bool,
    pub config_file: Option<PathBuf>,
    /// Whether the mod is also available on another backend. If so, the frontend
    /// can display a context option to switch to the other backend.
    pub alternate_backend: Option<AlternateBackend>,
    pub data: FrontendMod,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlternateBackend {
    pub backend: Backend,
    pub latest_version: String,
    pub latest_version_uuid: Uuid,
}

impl Profile {
    pub(super) fn query_mods(
        &self,
        args: &QueryModsArgs,
        thunderstore: &Thunderstore,
    ) -> (Vec<FrontendProfileMod>, Vec<Dependant>) {
        let mut unknown = Vec::new();

        let mods = self
            .mods
            .iter()
            .enumerate()
            .filter_map(|(index, profile_mod)| {
                if let Ok(queryable) = QueryableProfileMod::create(profile_mod, index, thunderstore)
                {
                    Some(queryable)
                } else {
                    warn!(
                        "unknown mod: {} while querying {}",
                        profile_mod.ident(),
                        self.name
                    );
                    unknown.push(Dependant::from(profile_mod));
                    None
                }
            });

        let query_result = thunderstore::query::query_mods(args, mods)
            .into_iter()
            .take(args.max_count.unwrap_or(usize::MAX))
            .map(|queryable| {
                let (data, uuid, alternate_backend) = match queryable.kind {
                    QueryableProfileModKind::Local(local) => {
                        (FrontendMod::from(local.clone()), local.uuid, None)
                    }
                    QueryableProfileModKind::Thunderstore(remote) => {
                        // check if the mod also exists on the other backend
                        let alternate_backend = remote.backend().other();
                        let alternate_backend = thunderstore
                            .get_package(remote.package.uuid, alternate_backend)
                            .ok()
                            .map(|pkg| {
                                let latest = pkg.latest_released();
                                AlternateBackend {
                                    backend: alternate_backend,
                                    latest_version: latest.version().to_string(),
                                    latest_version_uuid: latest.uuid,
                                }
                            });

                        (
                            FrontendMod::from(remote),
                            remote.package.uuid,
                            alternate_backend,
                        )
                    }
                };

                FrontendProfileMod {
                    data,
                    enabled: queryable.enabled,
                    alternate_backend,
                    config_file: self.linked_config.get(&uuid).cloned(),
                }
            })
            .collect();

        (query_result, unknown)
    }
}

impl Queryable for LocalMod {
    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn full_name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> Option<semver::Version> {
        self.version.clone()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn matches(&self, _args: &QueryModsArgs) -> bool {
        true
    }

    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering {
        let order = match args.sort_by {
            SortBy::Name => other.name.cmp(&self.name),
            SortBy::Author => match (&other.author, &self.author) {
                (Some(a), Some(b)) => a.cmp(b),
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (None, None) => Ordering::Equal,
            },
            _ => Ordering::Equal,
        };

        match args.sort_order {
            SortOrder::Ascending => order,
            SortOrder::Descending => order.reverse(),
        }
    }

    fn backend(&self) -> thunderstore::Backend {
        thunderstore::Backend::Thunderstore
    }
}
