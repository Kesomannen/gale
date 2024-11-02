use std::cmp::Ordering;

use chrono::{DateTime, Utc};

use crate::thunderstore::{
    self,
    models::{FrontendProfileMod, IntoFrontendMod},
    query::{QueryModsArgs, Queryable, SortBy, SortOrder},
    BorrowedMod, Thunderstore,
};

use super::{Dependant, LocalMod, Profile, ProfileMod, ProfileModKind};
use anyhow::Result;

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

impl<'a> Queryable for QueryableProfileMod<'a> {
    fn full_name(&self) -> &str {
        use QueryableProfileModKind as Kind;

        match &self.kind {
            Kind::Local(local) => &local.name,
            Kind::Thunderstore(remote) => remote.package.ident.as_str(),
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
                match QueryableProfileMod::create(profile_mod, index, thunderstore) {
                    Ok(queryable) => Some(queryable),
                    Err(_) => {
                        unknown.push(Dependant::from(profile_mod));
                        None
                    }
                }
            });

        let found = thunderstore::query::query_mods(args, mods)
            .map(|queryable| {
                let (data, uuid) = match queryable.kind {
                    QueryableProfileModKind::Local(local) => (local.clone().into(), local.uuid),
                    QueryableProfileModKind::Thunderstore(remote) => {
                        (remote.into_frontend(self), remote.package.uuid4)
                    }
                };

                FrontendProfileMod {
                    data,
                    enabled: queryable.enabled,
                    config_file: self.linked_config.get(&uuid).cloned(),
                }
            })
            .collect();

        (found, unknown)
    }
}