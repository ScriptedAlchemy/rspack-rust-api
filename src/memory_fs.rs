#![deny(warnings)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use futures::future::BoxFuture;
use tokio::sync::RwLock as AsyncRwLock;
use rspack_fs::{
    r#async::{AsyncReadableFileSystem, AsyncWritableFileSystem},
    sync::{ReadableFileSystem, WritableFileSystem},
    Result,
};

#[derive(Clone)]
pub struct MockFileSystem {
    pub files: Arc<AsyncRwLock<HashMap<PathBuf, Vec<u8>>>>,
    pub directories: Arc<AsyncRwLock<HashMap<PathBuf, ()>>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        dbg!("Creating new MockFileSystem");
        Self {
            files: Arc::new(AsyncRwLock::new(HashMap::new())),
            directories: Arc::new(AsyncRwLock::new(HashMap::new())),
        }
    }
}

impl WritableFileSystem for MockFileSystem {
    fn create_dir(&self, dir: &Path) -> Result<()> {
        let dir_ref = dir.to_path_buf();
        dbg!("Creating directory: {}", dir_ref.display());
        let mut directories = self.directories.blocking_write();
        directories.insert(dir_ref, ());
        Ok(())
    }

    fn create_dir_all(&self, dir: &Path) -> Result<()> {
        let dir_ref = dir.to_path_buf();
        dbg!("Creating directory recursively: {}", dir_ref.display());
        let mut directories = self.directories.blocking_write();
        directories.insert(dir_ref, ());
        Ok(())
    }

    fn write(&self, file: &Path, data: &[u8]) -> Result<()> {
        let file_ref = file.to_path_buf();
        dbg!("Writing to file: {}", file_ref.display());
        let mut files = self.files.blocking_write();
        files.insert(file_ref, data.to_vec());
        Ok(())
    }
}

impl ReadableFileSystem for MockFileSystem {
    fn read(&self, file: &Path) -> Result<Vec<u8>> {
        let file_ref = file.to_path_buf();
        dbg!("Reading file: {}", file_ref.display());
        let files = self.files.blocking_read();
        files.get(&file_ref).cloned().ok_or_else(|| rspack_fs::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")))
    }
}

impl AsyncWritableFileSystem for MockFileSystem {
    fn create_dir(&self, dir: &Path) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.to_path_buf();
        dbg!("Async creating directory: {}", dir_ref.display());
        let directories = self.directories.clone();
        Box::pin(async move {
            let mut directories = directories.write().await;
            directories.insert(dir_ref, ());
            Ok(())
        })
    }

    fn create_dir_all(&self, dir: &Path) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.to_path_buf();
        dbg!("Async creating directory recursively: {}", dir_ref.display());
        let directories = self.directories.clone();
        Box::pin(async move {
            let mut directories = directories.write().await;
            directories.insert(dir_ref, ());
            Ok(())
        })
    }

    fn write(&self, file: &Path, data: &[u8]) -> BoxFuture<'_, Result<()>> {
        let file_ref = file.to_path_buf();
        let data = data.to_vec();
        dbg!("Async writing to file: {}", file_ref.display());
        let files = self.files.clone();
        Box::pin(async move {
            let mut files = files.write().await;
            files.insert(file_ref, data);
            Ok(())
        })
    }

    fn remove_file(&self, file: &Path) -> BoxFuture<'_, Result<()>> {
        let file_ref = file.to_path_buf();
        dbg!("Async removing file: {}", file_ref.display());
        let files = self.files.clone();
        Box::pin(async move {
            let mut files = files.write().await;
            files.remove(&file_ref);
            Ok(())
        })
    }

    fn remove_dir_all(&self, dir: &Path) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.to_path_buf();
        dbg!(dir_ref.display());
        let directories = self.directories.clone();
        Box::pin(async move {
            let mut directories = directories.write().await;
            directories.remove(&dir_ref);
            Ok(())
        })
    }
}

impl AsyncReadableFileSystem for MockFileSystem {
    fn read(&self, file: &Path) -> BoxFuture<'_, rspack_fs::Result<Vec<u8>>> {
        let file_ref = file.to_path_buf();
        dbg!(file_ref.display());
        let files = self.files.clone();
        Box::pin(async move {
            let files = files.read().await;
            files.get(&file_ref).cloned().ok_or_else(|| rspack_fs::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")))
        })
    }
}