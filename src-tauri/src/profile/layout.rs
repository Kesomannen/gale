use std::collections::{HashMap, HashSet};

use eyre::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::profile::{Profile, ProfileMod};

/// A folder in the profile mod list. `mods` holds the ordered package uuids
/// of the mods in the folder, in display order. The folder may be empty.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileFolder {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub mods: Vec<Uuid>,
}

/// One entry of the profile's top-level layout: a loose mod or a folder.
/// The order of the entries is the custom order of the mod list.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LayoutItem {
    Mod { uuid: Uuid },
    Folder(ProfileFolder),
}

impl LayoutItem {
    pub fn as_mod(&self) -> Option<Uuid> {
        match self {
            LayoutItem::Mod { uuid } => Some(*uuid),
            LayoutItem::Folder(_) => None,
        }
    }

    pub fn as_folder(&self) -> Option<&ProfileFolder> {
        match self {
            LayoutItem::Mod { .. } => None,
            LayoutItem::Folder(folder) => Some(folder),
        }
    }
}

impl Profile {
    /// The folders in the profile, in layout order.
    pub fn folders(&self) -> impl Iterator<Item = &ProfileFolder> {
        self.layout.iter().filter_map(LayoutItem::as_folder)
    }

    /// Makes the layout consistent with the currently installed mods.
    ///
    /// - derives the layout from the mods if it is empty (e.g. old saves),
    /// - drops dangling mod items and folder members,
    /// - removes duplicate folders,
    /// - reorders `mods` to match the layout order, appending any mods that
    ///   aren't referenced by the layout as loose mods at the end.
    ///
    /// Empty folders are preserved.
    pub fn reconcile_layout(&mut self) {
        if self.layout.is_empty() {
            self.layout = self
                .mods
                .iter()
                .map(|m| LayoutItem::Mod { uuid: m.uuid() })
                .collect();
            return;
        }

        let mods = std::mem::take(&mut self.mods);
        let mod_uuids: HashSet<Uuid> = mods.iter().map(ProfileMod::uuid).collect();

        let mut seen_folder_ids = HashSet::new();
        self.layout.retain(|item| match item {
            LayoutItem::Mod { uuid } => mod_uuids.contains(uuid),
            LayoutItem::Folder(folder) => seen_folder_ids.insert(folder.id),
        });

        // reorder the mods to match the layout (including folder members),
        // appending unreferenced mods as loose
        let mut by_uuid: HashMap<Uuid, ProfileMod> =
            mods.into_iter().map(|m| (m.uuid(), m)).collect();
        let mut ordered: Vec<ProfileMod> = Vec::with_capacity(by_uuid.len());

        for item in &self.layout {
            match item {
                LayoutItem::Mod { uuid } => {
                    if let Some(profile_mod) = by_uuid.remove(uuid) {
                        ordered.push(profile_mod);
                    }
                }
                LayoutItem::Folder(folder) => {
                    for uuid in &folder.mods {
                        if let Some(profile_mod) = by_uuid.remove(uuid) {
                            ordered.push(profile_mod);
                        }
                    }
                }
            }
        }

        self.layout
            .extend(by_uuid.keys().cloned().map(|uuid| LayoutItem::Mod { uuid }));
        ordered.extend(by_uuid.into_values());
        self.mods = ordered;
    }

    /// Removes a mod from the layout after it has been removed from `mods`,
    /// keeping any folders (which may be left empty).
    pub(super) fn remove_from_layout(&mut self, uuid: Uuid) {
        self.layout.retain_mut(|item| match item {
            LayoutItem::Mod { uuid: u } => *u != uuid,
            LayoutItem::Folder(folder) => {
                folder.mods.retain(|m| *m != uuid);
                true
            }
        });
    }

    /// Replaces the profile's layout with the given items.
    ///
    /// This is the single source of truth for the custom order of the mod list
    /// and for folder membership. The `mods` list is rebuilt to match the
    /// layout, so mods that aren't part of the layout (e.g. ones installed
    /// concurrently) are appended at the end as loose mods.
    pub fn set_layout(&mut self, items: Vec<LayoutItem>) -> Result<()> {
        self.validate_layout(&items).context("invalid layout")?;

        let mods = std::mem::take(&mut self.mods);
        let mut by_uuid: HashMap<Uuid, ProfileMod> =
            mods.into_iter().map(|m| (m.uuid(), m)).collect();

        let mut ordered: Vec<ProfileMod> = Vec::with_capacity(by_uuid.len());
        let mut layout: Vec<LayoutItem> = Vec::with_capacity(items.len());

        for item in items {
            match item {
                LayoutItem::Mod { uuid } => {
                    ordered.push(by_uuid.remove(&uuid).expect("mod was validated to exist"));
                    layout.push(LayoutItem::Mod { uuid });
                }
                LayoutItem::Folder(folder) => {
                    for uuid in &folder.mods {
                        ordered.push(by_uuid.remove(uuid).expect("mod was validated to exist"));
                    }
                    layout.push(LayoutItem::Folder(folder));
                }
            }
        }

        let remaining: Vec<ProfileMod> = by_uuid.into_values().collect();
        layout.extend(remaining.iter().map(|m| LayoutItem::Mod { uuid: m.uuid() }));
        ordered.extend(remaining);

        self.mods = ordered;
        self.layout = layout;

        Ok(())
    }

    fn validate_layout(&self, items: &[LayoutItem]) -> Result<()> {
        let mut folder_ids = HashSet::new();
        let mut used_mods = HashSet::new();

        for item in items {
            match item {
                LayoutItem::Mod { uuid } => {
                    ensure!(
                        self.has_mod(*uuid),
                        "mod {uuid} is not installed in this profile"
                    );
                    ensure!(
                        used_mods.insert(*uuid),
                        "mod {uuid} appears multiple times in the layout"
                    );
                }
                LayoutItem::Folder(folder) => {
                    ensure!(
                        folder_ids.insert(folder.id),
                        "folder {} appears multiple times in the layout",
                        folder.id,
                    );
                    ensure!(
                        !folder.name.trim().is_empty(),
                        "folder name cannot be empty"
                    );
                    for uuid in &folder.mods {
                        ensure!(
                            self.has_mod(*uuid),
                            "mod {uuid} is not installed in this profile"
                        );
                        ensure!(
                            used_mods.insert(*uuid),
                            "mod {uuid} appears multiple times in the layout"
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
