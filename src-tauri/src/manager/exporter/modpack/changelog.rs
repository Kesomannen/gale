use std::fs;

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use log::warn;

use super::ModpackArgs;
use crate::{
    games::Game,
    manager::Profile,
    thunderstore::{models::PackageListing, BorrowedMod, ModRef, Thunderstore},
    util::{self, fs::JsonStyle},
};

pub fn generate(
    args: &mut ModpackArgs,
    profile: &Profile,
    game: &'static Game,
    thunderstore: &Thunderstore,
) -> Result<()> {
    let version = args
        .version_number
        .parse()
        .context("invalid version number")?;

    let snapshot = match profile.read_snapshot(version)? {
        Some(snapshot) => snapshot,
        None => bail!("no previous version found to compare against"),
    };

    let old = borrow_mods(snapshot, thunderstore);
    let current = borrow_mods(profile.mods_to_pack(args).cloned(), thunderstore);

    let version_header = format!("## {}\n\n", args.version_number);
    let index = match args.changelog.find(&version_header) {
        Some(index) => {
            let offset = index + version_header.len();

            // find the next header
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

    let diff = generate_diff(old, current, game);

    if diff.is_empty() {
        return Ok(());
    }

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
        util::fs::add_extension(&mut path, "json");

        util::fs::write_json(
            path,
            &self.mods_to_pack(args).collect_vec(),
            JsonStyle::Compact,
        )
    }

    fn read_snapshot(&self, below_version: semver::Version) -> Result<Option<Vec<ModRef>>> {
        let path = self.path.join("snapshots");

        if !path.exists() {
            return Ok(None);
        }

        // find the latest snapshot that is below the given version
        path.read_dir()?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                match entry
                    .file_name()
                    .to_string_lossy()
                    .trim_end_matches(".json")
                    .parse::<semver::Version>()
                {
                    Ok(version) if version >= below_version => None,
                    Ok(version) => Some((entry, version)),
                    _ => {
                        warn!(
                            "snapshot file is not a valid version (at {})",
                            entry.path().display()
                        );
                        None
                    }
                }
            })
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(entry, _)| util::fs::read_json(&entry.path()))
            .transpose()
    }
}

fn generate_diff(
    old: Vec<BorrowedMod<'_>>,
    new: Vec<BorrowedMod<'_>>,
    game: &'static Game,
) -> String {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut updated = Vec::new();

    for new in &new {
        if let Some(old) = old.iter().find(|old| old.package == new.package) {
            if old.version != new.version {
                updated.push((old, new));
            }
        } else {
            added.push(new);
        }
    }

    for old in &old {
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

fn markdown_link(url: &str, text: &str) -> String {
    format!("[{}]({})", text, url)
}

fn package_link(package: &PackageListing, game: &'static Game) -> String {
    markdown_link(&package.url(game), &package.name)
}

fn author_link(package: &PackageListing, game: &'static Game) -> String {
    markdown_link(&package.author_url(game), package.author())
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
        changelog.push_str(&format!("### {}\n\n", title));
        changelog.push_str(&format!("- {}\n", text(&item)));

        for item in items {
            changelog.push_str(&format!("- {}\n", text(&item)));
        }

        changelog.push('\n');
    }
}
