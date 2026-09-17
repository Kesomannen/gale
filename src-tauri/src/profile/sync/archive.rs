use std::{
    collections::{BTreeMap, HashSet},
    io::{Cursor, Read},
    str,
};

use eyre::{Context, OptionExt, Result, ensure};
use zip::ZipArchive;

use crate::profile::export::{
    ConfigPath, ContentHash, ProfileManifest, SyncManifest, manifest_revision,
};

#[derive(Debug)]
pub(crate) enum SyncArchiveFormat {
    Selective(SyncManifest),
    Legacy,
}

#[derive(Debug, Clone)]
pub(crate) struct ValidatedConfigFile {
    pub hash: ContentHash,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct ValidatedSyncArchive {
    pub manifest: ProfileManifest,
    pub format: SyncArchiveFormat,
    pub config: BTreeMap<ConfigPath, ValidatedConfigFile>,
}

pub(crate) fn validate(bytes: &[u8]) -> Result<ValidatedSyncArchive> {
    let names = central_directory_names(bytes)?;

    let mut archive = ZipArchive::new(Cursor::new(bytes)).context("failed to open sync archive")?;
    ensure!(
        archive.len() == names.len(),
        "archive entry count does not match central directory"
    );

    let mut seen: HashSet<String> = HashSet::new();
    let mut manifest_index: Option<usize> = None;
    let mut entries: Vec<(usize, ConfigPath)> = Vec::new();

    for (index, raw_name) in names.into_iter().enumerate() {
        let entry = archive
            .by_index(index)
            .context("failed to read archive entry")?;

        ensure!(
            entry.is_file(),
            "archive entry at index {index} is not a regular file"
        );
        ensure!(
            entry.name_raw() == raw_name,
            "archive entry name does not match central directory"
        );

        let name = str::from_utf8(raw_name).context("archive entry name is not valid UTF-8")?;
        ensure!(
            !name.contains('\\'),
            "archive entry name uses backslash separators: {name}"
        );

        let path = if name == "export.r2x" {
            None
        } else {
            Some(
                ConfigPath::try_from(name.to_owned())
                    .with_context(|| format!("invalid archive entry path: {name}"))?,
            )
        };

        let key = path.as_ref().map_or("export.r2x", ConfigPath::as_str);
        ensure!(
            seen.insert(key.to_ascii_lowercase()),
            "duplicate archive entry path: {name}"
        );

        match path {
            Some(path) => entries.push((index, path)),
            None => manifest_index = Some(index),
        }
    }

    let manifest_index = manifest_index.ok_or_eyre("archive is missing export.r2x")?;

    let manifest: ProfileManifest = {
        let bytes = read_entry(&mut archive, manifest_index)?;
        serde_yaml::from_slice(&bytes).context("failed to parse profile manifest")?
    };

    let mut payloads: BTreeMap<ConfigPath, Vec<u8>> = BTreeMap::new();
    for (index, path) in entries {
        let bytes = read_entry(&mut archive, index)?;
        payloads.insert(path, bytes);
    }

    let (format, config) = match &manifest.sync {
        Some(sync) => {
            ensure!(
                sync.version == 1,
                "unsupported sync archive version: {}",
                sync.version
            );
            ensure!(
                manifest_revision(&manifest)? == sync.mods_revision,
                "advertised mods revision does not match manifest"
            );

            let mut advertised = HashSet::new();
            for path in sync.config.keys() {
                ensure!(
                    advertised.insert(path.as_str().to_ascii_lowercase()),
                    "duplicate advertised config path: {path}"
                );
            }

            ensure!(
                payloads.len() == sync.config.len() && payloads.keys().eq(sync.config.keys()),
                "archive contents do not match advertised config files"
            );

            let config = payloads
                .into_iter()
                .map(|(path, bytes)| {
                    let entry = &sync.config[&path];
                    let computed = ContentHash::from_hash(blake3::hash(&bytes));
                    ensure!(
                        computed == entry.hash,
                        "config file {path} does not match its advertised hash"
                    );
                    Ok((
                        path,
                        ValidatedConfigFile {
                            hash: entry.hash.clone(),
                            bytes,
                        },
                    ))
                })
                .collect::<Result<_>>()?;

            (SyncArchiveFormat::Selective(sync.clone()), config)
        }
        None => {
            let config = payloads
                .into_iter()
                .map(|(path, bytes)| {
                    (
                        path,
                        ValidatedConfigFile {
                            hash: ContentHash::from_hash(blake3::hash(&bytes)),
                            bytes,
                        },
                    )
                })
                .collect();

            (SyncArchiveFormat::Legacy, config)
        }
    };

    Ok(ValidatedSyncArchive {
        manifest,
        format,
        config,
    })
}

fn central_directory_names(bytes: &[u8]) -> Result<Vec<&[u8]>> {
    const EOCD_LEN: usize = 22;
    const CD_HEADER_LEN: usize = 46;

    let tail_start = bytes.len().saturating_sub(EOCD_LEN + u16::MAX as usize);
    let eocd_pos = bytes[tail_start..]
        .windows(4)
        .rposition(|window| u32::from_le_bytes(window.try_into().unwrap()) == 0x06054b50)
        .map(|index| tail_start + index)
        .ok_or_eyre("archive is missing end of central directory")?;

    let eocd = bytes
        .get(eocd_pos..eocd_pos + EOCD_LEN)
        .ok_or_eyre("truncated end of central directory")?;

    let disk = u32::from_le_bytes(eocd[4..8].try_into().unwrap());
    let entries_on_disk = u16::from_le_bytes(eocd[8..10].try_into().unwrap());
    let count = u16::from_le_bytes(eocd[10..12].try_into().unwrap());
    ensure!(
        disk == 0 && entries_on_disk == count,
        "multi-disk archives are not supported"
    );
    ensure!(count != u16::MAX, "zip64 archives are not supported");

    let cd_size = u32::from_le_bytes(eocd[12..16].try_into().unwrap());
    ensure!(cd_size != u32::MAX, "zip64 archives are not supported");

    let cd_start = eocd_pos
        .checked_sub(cd_size as usize)
        .ok_or_eyre("malformed central directory")?;

    let mut names = Vec::with_capacity(count as usize);
    let mut cursor = cd_start;

    for _ in 0..count {
        let header = bytes
            .get(cursor..cursor + CD_HEADER_LEN)
            .ok_or_eyre("truncated central directory")?;
        ensure!(
            u32::from_le_bytes(header[..4].try_into().unwrap()) == 0x02014b50,
            "malformed central directory"
        );

        let name_len = u16::from_le_bytes(header[28..30].try_into().unwrap()) as usize;
        let extra_len = u16::from_le_bytes(header[30..32].try_into().unwrap()) as usize;
        let comment_len = u16::from_le_bytes(header[32..34].try_into().unwrap()) as usize;

        let name = bytes
            .get(cursor + CD_HEADER_LEN..cursor + CD_HEADER_LEN + name_len)
            .ok_or_eyre("truncated central directory")?;
        names.push(name);

        cursor += CD_HEADER_LEN + name_len + extra_len + comment_len;
    }

    ensure!(cursor == eocd_pos, "central directory size mismatch");

    Ok(names)
}

fn read_entry(archive: &mut ZipArchive<Cursor<&[u8]>>, index: usize) -> Result<Vec<u8>> {
    let mut entry = archive
        .by_index(index)
        .context("failed to read archive entry")?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .context("failed to read archive entry contents")?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::*;
    use crate::{
        profile::export::{ModRevision, R2Mod, SyncFileEntry},
        thunderstore::{Backend, PackageIdent},
    };

    fn base_manifest() -> ProfileManifest {
        ProfileManifest {
            name: "Test".to_owned(),
            mods: vec![R2Mod {
                ident: PackageIdent::from(("Author", "Mod")),
                version: semver::Version::new(1, 0, 0).into(),
                enabled: true,
                source: Backend::Thunderstore,
            }],
            game: Some("risk-of-rain-2".to_owned()),
            ignored_version_updates: Vec::new(),
            ignored_package_updates: Vec::new(),
            sync: None,
        }
    }

    fn selective_manifest(config: &[(&str, &[u8])]) -> ProfileManifest {
        let mut manifest = base_manifest();
        manifest.sync = Some(SyncManifest {
            version: 1,
            mods_revision: manifest_revision(&manifest).unwrap(),
            config: config
                .iter()
                .map(|(path, bytes)| {
                    (
                        ConfigPath::try_from(*path).unwrap(),
                        SyncFileEntry {
                            hash: ContentHash::from_hash(blake3::hash(bytes)),
                        },
                    )
                })
                .collect(),
        });
        manifest
    }

    fn manifest_yaml(manifest: &ProfileManifest) -> Vec<u8> {
        serde_yaml::to_string(manifest).unwrap().into_bytes()
    }

    fn zip_of(entries: &[(&str, Option<&[u8]>)]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            for (name, contents) in entries {
                match contents {
                    Some(contents) => {
                        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
                        zip.write_all(contents).unwrap();
                    }
                    None => {
                        zip.add_directory(*name, SimpleFileOptions::default())
                            .unwrap();
                    }
                }
            }
            zip.finish().unwrap();
        }
        cursor.into_inner()
    }

    fn zip_files(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let entries: Vec<(&str, Option<&[u8]>)> = entries
            .iter()
            .map(|(name, contents)| (*name, Some(*contents)))
            .collect();
        zip_of(&entries)
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = !0u32;
        for &byte in bytes {
            crc ^= byte as u32;
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
            }
        }
        !crc
    }

    fn raw_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();

        for (name, contents) in entries {
            let name = name.as_bytes();
            let crc = crc32(contents);
            let offset = out.len() as u32;
            let len = contents.len() as u32;

            out.extend_from_slice(&0x04034b50u32.to_le_bytes());
            out.extend_from_slice(&20u16.to_le_bytes());
            out.extend_from_slice(&[0; 8]);
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name);
            out.extend_from_slice(contents);

            central.extend_from_slice(&0x02014b50u32.to_le_bytes());
            central.extend_from_slice(&20u16.to_le_bytes());
            central.extend_from_slice(&20u16.to_le_bytes());
            central.extend_from_slice(&[0; 8]);
            central.extend_from_slice(&crc.to_le_bytes());
            central.extend_from_slice(&len.to_le_bytes());
            central.extend_from_slice(&len.to_le_bytes());
            central.extend_from_slice(&(name.len() as u16).to_le_bytes());
            central.extend_from_slice(&[0; 12]);
            central.extend_from_slice(&offset.to_le_bytes());
            central.extend_from_slice(name);
        }

        let cd_offset = out.len() as u32;
        out.extend_from_slice(&central);
        out.extend_from_slice(&0x06054b50u32.to_le_bytes());
        out.extend_from_slice(&[0; 4]);
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(central.len() as u32).to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());

        out
    }

    #[test]
    fn valid_selective_archive() {
        let manifest = selective_manifest(&[("BepInEx/config/example.cfg", b"custom")]);
        let yaml = manifest_yaml(&manifest);
        let archive = zip_files(&[
            ("export.r2x", yaml.as_slice()),
            ("BepInEx/config/example.cfg", b"custom"),
        ]);

        let validated = validate(&archive).unwrap();

        let SyncArchiveFormat::Selective(sync) = &validated.format else {
            panic!("expected selective archive");
        };
        assert_eq!(sync.version, 1);

        let path = ConfigPath::try_from("BepInEx/config/example.cfg").unwrap();
        let file = &validated.config[&path];
        assert_eq!(file.bytes, b"custom");
        assert_eq!(file.hash, ContentHash::from_hash(blake3::hash(b"custom")));
    }

    #[test]
    fn valid_legacy_archive() {
        let yaml = manifest_yaml(&base_manifest());
        let archive = zip_files(&[
            ("export.r2x", yaml.as_slice()),
            ("BepInEx/config/example.cfg", b"data"),
        ]);

        let validated = validate(&archive).unwrap();

        assert!(matches!(validated.format, SyncArchiveFormat::Legacy));

        let path = ConfigPath::try_from("BepInEx/config/example.cfg").unwrap();
        let file = &validated.config[&path];
        assert_eq!(file.bytes, b"data");
        assert_eq!(file.hash, ContentHash::from_hash(blake3::hash(b"data")));
    }

    #[test]
    fn rejects_unsafe_paths() {
        let yaml = manifest_yaml(&base_manifest());
        for path in [
            "../x.cfg",
            "a\\b.cfg",
            "_state/x.cfg",
            "_STATE/x.cfg",
            "/abs/x.cfg",
            "C:/abs/x.cfg",
        ] {
            let archive = zip_files(&[("export.r2x", yaml.as_slice()), (path, b"x")]);
            assert!(validate(&archive).is_err(), "{path}");
        }
    }

    #[test]
    fn rejects_symlink_entry() {
        let yaml = manifest_yaml(&base_manifest());

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            zip.start_file("export.r2x", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&yaml).unwrap();
            zip.add_symlink(
                "BepInEx/config/link.cfg",
                "/etc/passwd",
                SimpleFileOptions::default(),
            )
            .unwrap();
            zip.finish().unwrap();
        }

        assert!(validate(cursor.get_ref()).is_err());
    }

    #[test]
    fn rejects_duplicate_paths() {
        let yaml = manifest_yaml(&base_manifest());
        let archive = raw_zip(&[
            ("export.r2x", yaml.as_slice()),
            ("a.cfg", b"1"),
            ("a.cfg", b"2"),
        ]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_case_colliding_paths() {
        let yaml = manifest_yaml(&base_manifest());
        let archive = zip_files(&[
            ("export.r2x", yaml.as_slice()),
            ("BepInEx/config/a.cfg", b"1"),
            ("bepinex/config/a.cfg", b"2"),
        ]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_duplicate_manifest() {
        let yaml = manifest_yaml(&base_manifest());
        let archive = raw_zip(&[
            ("export.r2x", yaml.as_slice()),
            ("export.r2x", yaml.as_slice()),
        ]);
        assert!(validate(&archive).is_err());

        let archive = zip_files(&[
            ("export.r2x", yaml.as_slice()),
            ("Export.r2x", yaml.as_slice()),
        ]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_missing_manifest() {
        let archive = zip_files(&[("BepInEx/config/example.cfg", b"x")]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_directory_entry() {
        let yaml = manifest_yaml(&base_manifest());
        let archive = zip_of(&[
            ("export.r2x", Some(yaml.as_slice())),
            ("BepInEx/config/", None),
        ]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_case_colliding_advertised_keys() {
        let mut manifest = selective_manifest(&[("BepInEx/a.cfg", b"1"), ("bepinex/a.cfg", b"2")]);
        manifest.sync.as_mut().unwrap().config.insert(
            ConfigPath::try_from("BEPINEX/a.cfg").unwrap(),
            SyncFileEntry {
                hash: ContentHash::from_hash(blake3::hash(b"3")),
            },
        );

        let yaml = manifest_yaml(&manifest);
        let archive = zip_files(&[
            ("export.r2x", yaml.as_slice()),
            ("BepInEx/a.cfg", b"1"),
            ("bepinex/a.cfg", b"2"),
        ]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_missing_advertised_entry() {
        let manifest =
            selective_manifest(&[("BepInEx/a.cfg", b"1"), ("BepInEx/missing.cfg", b"2")]);

        let yaml = manifest_yaml(&manifest);
        let archive = zip_files(&[("export.r2x", yaml.as_slice()), ("BepInEx/a.cfg", b"1")]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_unadvertised_payload() {
        let manifest = selective_manifest(&[("BepInEx/a.cfg", b"1")]);

        let yaml = manifest_yaml(&manifest);
        let archive = zip_files(&[
            ("export.r2x", yaml.as_slice()),
            ("BepInEx/a.cfg", b"1"),
            ("BepInEx/extra.cfg", b"2"),
        ]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_bad_advertised_hash() {
        let mut manifest = selective_manifest(&[("BepInEx/a.cfg", b"1")]);
        manifest.sync.as_mut().unwrap().config.insert(
            ConfigPath::try_from("BepInEx/a.cfg").unwrap(),
            SyncFileEntry {
                hash: ContentHash::from_hash(blake3::hash(b"other")),
            },
        );

        let yaml = manifest_yaml(&manifest);
        let archive = zip_files(&[("export.r2x", yaml.as_slice()), ("BepInEx/a.cfg", b"1")]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_unsupported_version() {
        let mut manifest = selective_manifest(&[]);
        manifest.sync.as_mut().unwrap().version = 2;

        let yaml = manifest_yaml(&manifest);
        let archive = zip_files(&[("export.r2x", yaml.as_slice())]);
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_bad_mods_revision() {
        let mut manifest = selective_manifest(&[]);
        manifest.sync.as_mut().unwrap().mods_revision =
            ModRevision::from_hash(blake3::hash(b"wrong"));

        let yaml = manifest_yaml(&manifest);
        let archive = zip_files(&[("export.r2x", yaml.as_slice())]);
        assert!(validate(&archive).is_err());
    }
}
