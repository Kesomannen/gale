use std::{
    fmt::Display,
    fs::{self, DirEntry},
    iter,
};

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use log::warn;

use super::ModpackArgs;
use crate::{
    games::Game,
    manager::Profile,
    thunderstore::{models::PackageListing, BorrowedMod, ModRef, Thunderstore},
    util::{
        self,
        fs::{JsonStyle, PathExt},
    },
};

pub fn generate_all(
    args: &ModpackArgs,
    profile: &Profile,
    game: &'static Game,
    thunderstore: &Thunderstore,
) -> Result<String> {
    let current_version: semver::Version = args
        .version_number
        .parse()
        .context("invalid version number")?;

    let mut snapshots = profile
        .find_snapshots()?
        .filter(|(_, version)| *version < current_version)
        .map(|(entry, version)| {
            let mods = util::fs::read_json::<Vec<ModRef>>(entry.path())
                .with_context(|| format!("failed to read snapshot for version {}", version))?;

            let mods = borrow_mods(mods, thunderstore);

            Ok((mods, version))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut changelog = "# Changelog".to_string();

    if snapshots.is_empty() {
        push_diff(
            &mut changelog,
            &args.version_number,
            "\n\n- Initial release",
        );

        return Ok(changelog);
    }

    snapshots.sort_by(|(_, a), (_, b)| a.cmp(b).reverse());

    // list is now sorted in descending order
    // (current version: 1.2.0) [1.1.0, 1.0.0, 0.2.0, 0.1.0]

    // first generate diff to current version
    let current_mods = borrow_mods(profile.mods_to_pack(args).cloned(), thunderstore);
    let diff = generate_diff(&snapshots[0].0, &current_mods, game);

    push_diff(&mut changelog, &args.version_number, &diff);

    // then for all other versions (except the first one)
    for i in 0..snapshots.len().saturating_sub(1) {
        let (new_mods, new_version) = &snapshots[i];
        let (old_mods, _) = &snapshots[i + 1];

        let diff = generate_diff(old_mods, new_mods, game);
        push_diff(&mut changelog, &new_version.to_string(), &diff);
    }

    push_diff(
        &mut changelog,
        &snapshots.last().unwrap().1.to_string(),
        "\n\n- Initial release",
    );

    return Ok(changelog);

    fn push_diff(changelog: &mut String, version_number: &str, diff: &str) {
        if !diff.is_empty() {
            changelog.push_str("\n\n## ");
            changelog.push_str(version_number);
            changelog.push_str(diff);
        }
    }
}

pub fn generate_latest(
    args: &mut ModpackArgs,
    profile: &Profile,
    game: &'static Game,
    thunderstore: &Thunderstore,
) -> Result<()> {
    let version = args
        .version_number
        .parse()
        .context("invalid version number")?;

    let latest_snapshot = profile
        .find_snapshots()?
        .filter(|(_, v)| *v < version)
        .max_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(entry, _)| util::fs::read_json::<Vec<ModRef>>(entry.path()))
        .transpose()?;

    let latest_snapshot = match latest_snapshot {
        Some(snapshot) => snapshot,
        None => bail!("no previous version found to compare against"),
    };

    let old_mods = borrow_mods(latest_snapshot, thunderstore);
    let current_mods = borrow_mods(profile.mods_to_pack(args).cloned(), thunderstore);

    let version_header = format!("## {}", args.version_number);
    let index = match args.changelog.find(&version_header) {
        Some(index) => {
            // if there's an existing diff, replace it
            let offset = index + version_header.len();

            // find the next version header to see where the old diff ends
            let next_index = args.changelog[offset..]
                .find("\n## ")
                .map(|next_index| next_index + offset)
                .unwrap_or_else(|| args.changelog.len());

            args.changelog.drain(index..next_index);

            index
        }
        None => {
            // if there's no existing diff, insert a new one right below the header
            const HEADER: &str = "# Changelog\n\n";

            args.changelog
                .find(HEADER)
                .map(|index| index + HEADER.len())
                .unwrap_or_default()
        }
    };

    let mut diff = generate_diff(&old_mods, &current_mods, game);

    if diff.is_empty() {
        return Ok(());
    }

    diff.push('\n');

    args.changelog.insert_str(index, &diff);
    args.changelog.insert_str(index, &version_header);

    Ok(())
}

fn borrow_mods<T>(mods: T, thunderstore: &Thunderstore) -> Vec<BorrowedMod<'_>>
where
    T: IntoIterator<Item = ModRef>,
{
    mods.into_iter()
        .filter_map(|mod_ref| mod_ref.borrow(thunderstore).ok())
        .collect()
}

impl Profile {
    pub fn take_snapshot(&self, args: &ModpackArgs) -> Result<()> {
        let mut path = self.path.join("snapshots");
        fs::create_dir_all(&path)?;

        path.push(&args.version_number);
        path.add_extension("json");

        util::fs::write_json(
            path,
            &self.mods_to_pack(args).collect_vec(),
            JsonStyle::Compact,
        )
    }

    fn find_snapshots(&self) -> Result<Box<dyn Iterator<Item = (DirEntry, semver::Version)>>> {
        let path = self.path.join("snapshots");

        if !path.exists() {
            return Ok(Box::new(iter::empty()));
        }

        let iter = path
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                match entry
                    .file_name()
                    .to_string_lossy()
                    .trim_end_matches(".json")
                    .parse::<semver::Version>()
                {
                    Ok(version) => Some((entry, version)),
                    _ => {
                        warn!(
                            "snapshot file is not a valid version (at {})",
                            entry.path().display()
                        );
                        None
                    }
                }
            });

        Ok(Box::new(iter))
    }
}

fn generate_diff(old: &[BorrowedMod<'_>], new: &[BorrowedMod<'_>], game: &'static Game) -> String {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut updated = Vec::new();

    for new in new {
        if let Some(old) = old.iter().find(|old| old.package == new.package) {
            if old.version != new.version {
                updated.push((old, new));
            }
        } else {
            added.push(new);
        }
    }

    for old in old {
        if !new.iter().any(|new| new.package == old.package) {
            removed.push(old);
        }
    }

    let mut changelog = String::new();

    write_changelog_section(&mut changelog, "Added", added.into_iter(), |item| {
        format!(
            "{} by {} ({})",
            package_link(item.package, game),
            author_link(item.package, game),
            item.version.version_number
        )
    });

    write_changelog_section(&mut changelog, "Removed", removed.into_iter(), |item| {
        format!(
            "{} by {}",
            package_link(item.package, game),
            author_link(item.package, game)
        )
    });

    write_changelog_section(
        &mut changelog,
        "Updated",
        updated.into_iter(),
        |(old, new)| {
            format!(
                "{} {} ⇒ {}",
                package_link(old.package, game),
                old.version.version_number,
                new.version.version_number
            )
        },
    );

    changelog
}

fn markdown_link(url: impl Display, text: impl Display) -> String {
    format!("[{}]({})", text, url)
}

fn package_link(package: &PackageListing, game: &'static Game) -> String {
    markdown_link(package.url(game), &package.name)
}

fn author_link(package: &PackageListing, game: &'static Game) -> String {
    markdown_link(package.owner_url(game), &package.owner)
}

fn write_changelog_section<T, F>(
    changelog: &mut String,
    title: &str,
    mut items: impl Iterator<Item = T>,
    mut text: F,
) where
    F: FnMut(&T) -> String,
{
    if let Some(item) = items.next() {
        changelog.push_str(&format!("\n\n### {}", title));
        changelog.push_str(&format!("\n\n- {}", text(&item)));

        for item in items {
            changelog.push_str(&format!("\n- {}", text(&item)));
        }
    }
}
