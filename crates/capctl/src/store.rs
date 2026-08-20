use capability_protocol::ErrorCode;
use serde::de::DeserializeOwned;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

/// Read-only errors from the rooted document boundary.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("store root is not a real directory: {0}")]
    InvalidRoot(PathBuf),
    #[error("document path must be relative and contain only normal components: {0}")]
    InvalidPath(PathBuf),
    #[error("symbolic-link or reparse-point traversal is denied: {0}")]
    IndirectPath(PathBuf),
    #[error("document exceeds the admitted byte limit: {0}")]
    DocumentTooLarge(PathBuf),
    #[error("filesystem read failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("strict JSON decode failed for {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

impl StoreError {
    pub(crate) fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidRoot(_) => ErrorCode::InvalidRoot,
            Self::InvalidPath(_) => ErrorCode::InvalidPath,
            Self::IndirectPath(_) => ErrorCode::IndirectPath,
            Self::DocumentTooLarge(_) => ErrorCode::DocumentTooLarge,
            Self::Io { .. } => ErrorCode::IoError,
            Self::Json { .. } => ErrorCode::InvalidJson,
        }
    }
}

/// A read-only JSON boundary rooted at one non-indirect directory.
pub struct RootedJsonStore {
    root: PathBuf,
    max_document_bytes: usize,
}

impl RootedJsonStore {
    pub(crate) fn new(root: &Path, max_document_bytes: usize) -> Result<Self, StoreError> {
        let metadata = fs::symlink_metadata(root).map_err(|source| StoreError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        if !metadata.is_dir() {
            return Err(StoreError::InvalidRoot(root.to_path_buf()));
        }
        if is_indirect(&metadata) {
            return Err(StoreError::IndirectPath(root.to_path_buf()));
        }
        let root = fs::canonicalize(root).map_err(|source| StoreError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        Ok(Self {
            root,
            max_document_bytes,
        })
    }

    pub(crate) fn read<T: DeserializeOwned>(&self, relative: &Path) -> Result<T, StoreError> {
        let path = self.resolve(relative)?;
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
            options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
        }
        let file = options.open(&path).map_err(|source| StoreError::Io {
            path: path.clone(),
            source,
        })?;
        let metadata = file.metadata().map_err(|source| StoreError::Io {
            path: path.clone(),
            source,
        })?;
        if is_indirect(&metadata) {
            return Err(StoreError::IndirectPath(path));
        }
        let byte_len = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
        if byte_len > self.max_document_bytes {
            return Err(StoreError::DocumentTooLarge(path));
        }

        // Recheck the name after opening. Reading occurs from the stable handle;
        // an actuator still requires a handle-relative opener rooted at a stable
        // directory handle, which is intentionally outside this read-only V1.
        let reopened = fs::canonicalize(&path).map_err(|source| StoreError::Io {
            path: path.clone(),
            source,
        })?;
        if reopened != path || !reopened.starts_with(&self.root) {
            return Err(StoreError::IndirectPath(path));
        }
        let read_limit = u64::try_from(self.max_document_bytes)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let mut bytes = Vec::with_capacity(byte_len);
        file.take(read_limit)
            .read_to_end(&mut bytes)
            .map_err(|source| StoreError::Io {
                path: path.clone(),
                source,
            })?;
        if bytes.len() > self.max_document_bytes {
            return Err(StoreError::DocumentTooLarge(path));
        }
        serde_json::from_slice(&bytes).map_err(|source| StoreError::Json { path, source })
    }

    fn resolve(&self, relative: &Path) -> Result<PathBuf, StoreError> {
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(StoreError::InvalidPath(relative.to_path_buf()));
        }

        let mut candidate = self.root.clone();
        for component in relative.components() {
            let Component::Normal(segment) = component else {
                return Err(StoreError::InvalidPath(relative.to_path_buf()));
            };
            candidate.push(segment);
            let metadata = fs::symlink_metadata(&candidate).map_err(|source| StoreError::Io {
                path: candidate.clone(),
                source,
            })?;
            if is_indirect(&metadata) {
                return Err(StoreError::IndirectPath(candidate));
            }
        }
        let canonical = fs::canonicalize(&candidate).map_err(|source| StoreError::Io {
            path: candidate.clone(),
            source,
        })?;
        if !canonical.starts_with(&self.root) {
            return Err(StoreError::InvalidPath(relative.to_path_buf()));
        }
        Ok(canonical)
    }
}

fn is_indirect(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Fixture {
        value: String,
    }

    #[test]
    fn rejects_parent_traversal() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let store = RootedJsonStore::new(directory.path(), 1024)?;
        let result = store.read::<Fixture>(Path::new("../outside.json"));
        assert!(matches!(result, Err(StoreError::InvalidPath(_))));
        Ok(())
    }

    #[test]
    fn decodes_strict_json() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("fixture.json");
        fs::write(&path, br#"{"value":"ok"}"#)?;
        let store = RootedJsonStore::new(directory.path(), 1024)?;
        let value: Fixture = store.read(Path::new("fixture.json"))?;
        assert_eq!(value.value, "ok");
        Ok(())
    }

    #[test]
    fn rejects_document_over_read_limit() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("fixture.json");
        fs::write(&path, br#"{"value":"too-large"}"#)?;
        let store = RootedJsonStore::new(directory.path(), 8)?;
        let result = store.read::<Fixture>(Path::new("fixture.json"));
        assert!(matches!(result, Err(StoreError::DocumentTooLarge(_))));
        Ok(())
    }
}
