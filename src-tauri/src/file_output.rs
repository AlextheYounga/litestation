use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::{BackendError, DESTINATION_EXISTS, INVALID_PATH};

static TEMP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

/// A file staged beside its final destination, so installation cannot cross filesystems.
pub struct StagedOutput {
    destination: PathBuf,
    staging: PathBuf,
    file: Option<File>,
}

impl StagedOutput {
    pub fn create(destination: &str) -> Result<Self, BackendError> {
        let destination = destination_path(destination)?;
        let parent = destination
            .parent()
            .ok_or_else(|| BackendError::new(INVALID_PATH, "Destination must have a parent directory"))?;
        let (staging, file) = staging_file(parent, destination.file_name().unwrap_or_default())?;
        Ok(Self { destination, staging, file: Some(file) })
    }

    pub fn file(&mut self) -> Result<&mut File, BackendError> {
        self.file.as_mut().ok_or_else(|| BackendError::new("staging_file_closed", "Staging file is already closed"))
    }

    pub fn close_for_external_writer(&mut self) -> Result<(), BackendError> {
        if let Some(file) = self.file.take() {
            file.sync_all().map_err(output_error)?;
        }
        Ok(())
    }

    pub fn path_for_sqlite(&self) -> PathBuf {
        self.staging.clone()
    }

    pub fn destination_for_comparison(&self) -> PathBuf {
        self.destination.clone()
    }

    pub fn install(mut self) -> Result<PathBuf, BackendError> {
        if let Some(file) = self.file.take() {
            file.sync_all().map_err(output_error)?;
            drop(file);
        }

        // `hard_link` atomically fails if another entry won the destination name.
        // Staging is in the same directory, so this never crosses filesystems.
        fs::hard_link(&self.staging, &self.destination).map_err(|error| install_error(&error))?;
        // The hard link is the commit point: the destination now exists with the
        // fully synced content, so cleanup and directory durability are best effort.
        // Drop retries staging removal if this immediate cleanup cannot remove it.
        let _ = fs::remove_file(&self.staging);
        // Directory sync is supplementary once the destination link exists.
        let _ = sync_parent(self.destination.parent());
        Ok(self.destination.clone())
    }
}

impl Drop for StagedOutput {
    fn drop(&mut self) {
        let _ = self.file.take();
        let _ = fs::remove_file(&self.staging);
    }
}

pub fn write_new(
    destination: &str,
    write: impl FnOnce(&mut File) -> Result<(), BackendError>,
) -> Result<PathBuf, BackendError> {
    let mut staged = StagedOutput::create(destination)?;
    write(staged.file()?)?;
    staged.file()?.flush().map_err(output_error)?;
    staged.install()
}

pub fn destination_path(destination: &str) -> Result<PathBuf, BackendError> {
    let path = Path::new(destination);
    let name = path.file_name().filter(|name| !name.is_empty());
    if destination.is_empty() || name.is_none() {
        return Err(BackendError::new(INVALID_PATH, "Destination must name a file"));
    }
    match fs::symlink_metadata(path) {
        Ok(_) => {
            return Err(BackendError::new(DESTINATION_EXISTS, "Destination already exists and will not be overwritten"))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(output_error(error)),
    }
    let parent =
        path.parent().ok_or_else(|| BackendError::new(INVALID_PATH, "Destination must have a parent directory"))?;
    if !parent.is_dir() {
        return Err(BackendError::new(INVALID_PATH, "Destination parent directory does not exist"));
    }
    let parent = fs::canonicalize(parent).map_err(output_error)?;
    Ok(parent.join(name.unwrap_or_default()))
}

fn staging_file(parent: &Path, name: &OsStr) -> Result<(PathBuf, File), BackendError> {
    for _ in 0..100 {
        let number = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".{}.litestation-{}-{number}.tmp", name.to_string_lossy(), process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(output_error(error)),
        }
    }
    Err(BackendError::new("staging_file_failed", "Could not allocate a staging output file"))
}

fn install_error(error: &io::Error) -> BackendError {
    if error.kind() == io::ErrorKind::AlreadyExists {
        BackendError::new(DESTINATION_EXISTS, "Destination was created concurrently and will not be overwritten")
    } else {
        BackendError::new(
            "atomic_install_failed",
            format!("Could not atomically install destination without replacing an existing file: {error}"),
        )
    }
}

#[cfg(unix)]
fn sync_parent(parent: Option<&Path>) -> Result<(), BackendError> {
    let parent = parent.ok_or_else(|| BackendError::new(INVALID_PATH, "Destination must have a parent directory"))?;
    File::open(parent).and_then(|directory| directory.sync_all()).map_err(output_error)
}

#[cfg(not(unix))]
fn sync_parent(_parent: Option<&Path>) -> Result<(), BackendError> {
    // This platform has no portable directory fsync primitive; file content is
    // still synced, while directory-entry durability remains a platform limit.
    Ok(())
}

#[expect(clippy::needless_pass_by_value)]
fn output_error(error: io::Error) -> BackendError {
    BackendError::new("file_write_failed", format!("Could not write destination file: {error}"))
}
