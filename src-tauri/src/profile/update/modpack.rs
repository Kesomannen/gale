use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
};

use itertools::Itertools;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    profile::{Profile, ThunderstoreMod},
    thunderstore::{BorrowedMod, ModId, Thunderstore, VersionIdent},
};

/// How the installed version of a mod relates to the version pinned by a modpack.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    /// The installed version is older than the pinned one.
    Upgrade,
    /// The installed version is newer than the pinned one, and the modpack
    /// lowered its pinned version compared to the currently installed modpack version.
    Rollback,
    /// The installed version is newer than the pinned one, but the modpack didn't
    /// ask for a rollback, so it was most likely updated by the user.
    Ahead,
}

impl ChangeKind {
    /// Decides how an installed mod relates to the version pinned by a modpack.
    ///
    /// `previous` is the version pinned by the currently installed version of the modpack, if any.
    fn classify(
        installed: &semver::Version,
        previous: Option<&semver::Version>,
        pinned: &semver::Version,
    ) -> Option<Self> {
        match installed.cmp(pinned) {
            Ordering::Equal => None,
            Ordering::Less => Some(ChangeKind::Upgrade),
            Ordering::Greater if previous.is_some_and(|previous| pinned < previous) => {
                Some(ChangeKind::Rollback)
            }
            Ordering::Greater => Some(ChangeKind::Ahead),
        }
    }

    /// Whether the change should be selected by default.
    ///
    /// `conflict` is whether another installed modpack pins a different version, and
    /// `ignored` whether the user ignored updates to the pinned version.
    fn recommended(self, conflict: bool, ignored: bool) -> bool {
        match self {
            ChangeKind::Upgrade => !conflict && !ignored,
            ChangeKind::Rollback => !conflict,
            ChangeKind::Ahead => false,
        }
    }
}

/// An installed mod whose version differs from the one pinned by a modpack.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackChange {
    full_name: VersionIdent,
    /// The pinned version to install.
    id: ModId,
    old: semver::Version,
    new: semver::Version,
    kind: ChangeKind,
    /// Another installed modpack which pins a different version of this mod.
    conflict: Option<ConflictingModpack>,
    /// Whether the user ignored updates to the pinned version.
    ignored: bool,
    recommended: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictingModpack {
    name: String,
    version: String,
}

/// Finds the versions pinned by a modpack, including those of nested modpacks.
///
/// If several modpacks pin the same package, the outermost one wins.
fn pinned_versions<'a>(
    modpack: BorrowedMod<'a>,
    thunderstore: &'a Thunderstore,
) -> HashMap<Uuid, BorrowedMod<'a>> {
    let mut pins = HashMap::new();
    let mut queue = VecDeque::from([modpack]);

    while let Some(current) = queue.pop_front() {
        for ident in current.dependencies() {
            let Ok(dep) = thunderstore.find_ident(ident) else {
                continue; // ignore missing mods
            };

            if dep.package.uuid == modpack.package.uuid || pins.contains_key(&dep.package.uuid) {
                continue;
            }

            pins.insert(dep.package.uuid, dep);

            if dep.package.is_modpack() {
                queue.push_back(dep);
            }
        }
    }

    pins
}

/// Finds the installed version of a mod, even if it's no longer listed by the backend it was installed from.
fn installed_version<'a>(
    ts_mod: &ThunderstoreMod,
    thunderstore: &'a Thunderstore,
) -> Option<BorrowedMod<'a>> {
    ts_mod
        .id
        .borrow(thunderstore)
        .or_else(|_| thunderstore.find_ident(&ts_mod.ident))
        .ok()
}

impl Profile {
    /// Finds the installed mods whose versions differ from the ones pinned by `target`,
    /// a version of a modpack which is about to be installed in place of the current one.
    ///
    /// Normally, only the missing dependencies of the new modpack version are installed,
    /// which leaves the installed mods at whatever version they were before.
    ///
    /// Returns nothing if `target` is not a modpack.
    pub fn modpack_changes(
        &self,
        target: BorrowedMod<'_>,
        thunderstore: &Thunderstore,
    ) -> Vec<ModpackChange> {
        if !target.package.is_modpack() {
            return Vec::new();
        }

        let pins = pinned_versions(target, thunderstore);

        let previous_pins = self
            .get_mod_ok(target.package.uuid)
            .and_then(|profile_mod| profile_mod.as_thunderstore())
            .and_then(|(ts_mod, _)| installed_version(ts_mod, thunderstore))
            .map(|current| pinned_versions(current, thunderstore))
            .unwrap_or_default();

        // other enabled modpacks, excluding the target and the ones nested in either of its versions
        let other_modpacks = self
            .thunderstore_mods()
            .filter(|(ts_mod, enabled)| {
                let uuid = ts_mod.id.package_uuid;
                *enabled
                    && uuid != target.package.uuid
                    && !pins.contains_key(&uuid)
                    && !previous_pins.contains_key(&uuid)
            })
            .filter_map(|(ts_mod, _)| installed_version(ts_mod, thunderstore))
            .filter(|borrowed| borrowed.package.is_modpack())
            .map(|borrowed| (borrowed, pinned_versions(borrowed, thunderstore)))
            .collect_vec();

        pins.iter()
            .filter_map(|(uuid, pinned)| {
                let (ts_mod, _) = self.get_mod_ok(*uuid)?.as_thunderstore()?;

                let installed = ts_mod.ident.version().parse::<semver::Version>().ok()?;
                let previous = previous_pins
                    .get(uuid)
                    .map(|previous| previous.version.parsed_version());
                let new = pinned.version.parsed_version();

                let kind = ChangeKind::classify(&installed, previous.as_ref(), &new)?;

                let conflict = other_modpacks
                    .iter()
                    .find(|(_, other_pins)| {
                        other_pins
                            .get(uuid)
                            .is_some_and(|other| other.ident() != pinned.ident())
                    })
                    .map(|(other, _)| ConflictingModpack {
                        name: other.ident().name().to_owned(),
                        version: other.ident().version().to_owned(),
                    });

                let ignored = self.ignored_package_updates.contains(uuid)
                    || self.ignored_version_updates.contains(&pinned.version.uuid);

                Some(ModpackChange {
                    full_name: pinned.ident().clone(),
                    id: (*pinned).into(),
                    old: installed,
                    new,
                    kind,
                    recommended: kind.recommended(conflict.is_some(), ignored),
                    conflict,
                    ignored,
                })
            })
            .sorted_by_cached_key(|change| change.full_name.name().to_lowercase())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(str: &str) -> semver::Version {
        str.parse().unwrap()
    }

    fn classify(installed: &str, previous: Option<&str>, pinned: &str) -> Option<ChangeKind> {
        ChangeKind::classify(
            &version(installed),
            previous.map(version).as_ref(),
            &version(pinned),
        )
    }

    #[test]
    fn same_version_is_unchanged() {
        assert_eq!(classify("1.0.0", Some("0.9.0"), "1.0.0"), None);
        assert_eq!(classify("1.0.0", None, "1.0.0"), None);
    }

    #[test]
    fn older_version_is_upgraded() {
        assert_eq!(
            classify("1.0.0", Some("1.0.0"), "1.1.0"),
            Some(ChangeKind::Upgrade)
        );
        // pinned version didn't change, but the mod was installed before the modpack
        assert_eq!(
            classify("1.0.0", Some("1.1.0"), "1.1.0"),
            Some(ChangeKind::Upgrade)
        );
        // newly pinned
        assert_eq!(classify("1.0.0", None, "1.1.0"), Some(ChangeKind::Upgrade));
    }

    #[test]
    fn lowered_pin_is_a_rollback() {
        assert_eq!(
            classify("1.2.0", Some("1.2.0"), "1.1.0"),
            Some(ChangeKind::Rollback)
        );
        // the user was even further ahead of the modpack
        assert_eq!(
            classify("1.3.0", Some("1.2.0"), "1.1.0"),
            Some(ChangeKind::Rollback)
        );
    }

    #[test]
    fn user_ahead_of_modpack() {
        // the modpack didn't change its pin
        assert_eq!(
            classify("1.2.0", Some("1.1.0"), "1.1.0"),
            Some(ChangeKind::Ahead)
        );
        // the modpack raised its pin, but not as far as the user
        assert_eq!(
            classify("1.3.0", Some("1.1.0"), "1.2.0"),
            Some(ChangeKind::Ahead)
        );
        // newly pinned
        assert_eq!(classify("1.2.0", None, "1.1.0"), Some(ChangeKind::Ahead));
    }

    #[test]
    fn recommended_changes() {
        assert!(ChangeKind::Upgrade.recommended(false, false));
        assert!(ChangeKind::Rollback.recommended(false, false));
        assert!(!ChangeKind::Ahead.recommended(false, false));

        // another modpack pins a different version
        assert!(!ChangeKind::Upgrade.recommended(true, false));
        assert!(!ChangeKind::Rollback.recommended(true, false));

        // the user ignored this update
        assert!(!ChangeKind::Upgrade.recommended(false, true));
        assert!(ChangeKind::Rollback.recommended(false, true));
    }
}
