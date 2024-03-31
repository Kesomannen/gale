use std::{fs, io, path::{Path, PathBuf}};

pub fn flatten_if_exists(path: &Path) -> Result<bool, io::Error> {
    if !path.try_exists()? {
        return Ok(false);
    }

    let parent = path.parent().unwrap();

    for entry in fs::read_dir(path)? {
        let entry_path = entry?.path();
        let file_name = entry_path.file_name().unwrap();
        let new_path = parent.join(file_name);

        fs::rename(entry_path, new_path)?;
    }

    fs::remove_dir(path)?;

    Ok(true)
}

pub fn copy_dir(src: &Path, dest: &Path) -> Result<(), io::Error> {
    fs::create_dir_all(dest)?;
    copy_contents(src, dest)?;
    Ok(())
}

pub fn copy_contents(src: &Path, dest: &Path) -> Result<(), io::Error> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry_path.file_name().unwrap();
        let new_path = dest.join(file_name);

        if entry_path.is_dir() {
            fs::create_dir(&new_path)?;
            copy_contents(&entry_path, &new_path)?;
        } else {
            fs::copy(&entry_path, &new_path)?;
        }
    }

    Ok(())
}

pub fn add_extension<P: AsRef<Path>>(path: &mut PathBuf, extension: P) {
    match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_os_string();
            ext.push(".");
            ext.push(extension.as_ref());
            path.set_extension(ext)
        }
        None => path.set_extension(extension.as_ref()),
    };
}