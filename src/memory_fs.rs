#![deny(warnings)]

use std::{
    collections::HashMap,
    // io::{Read, Write},
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

#[macro_export]
macro_rules! cfg_async {
    ($($item:item)*) => {
        $( #[cfg(feature = "async")] $item )*
    }
}

#[macro_export]
macro_rules! cfg_native {
    ($($item:item)*) => {
        $( #[cfg(feature = "native")] $item )*
    }
}

#[derive(Clone)]
pub struct MockFileSystem {
    pub files: Arc<AsyncRwLock<HashMap<PathBuf, Vec<u8>>>>,
    pub directories: Arc<AsyncRwLock<HashMap<PathBuf, ()>>>, // Changed type to ()
}

impl MockFileSystem {
    pub fn new() -> Self {
        dbg!("Creating new MockFileSystem");
        Self {
            files: Arc::new(AsyncRwLock::new(HashMap::new())),
            directories: Arc::new(AsyncRwLock::new(HashMap::new())), // Changed type to ()
        }
    }
}

impl WritableFileSystem for MockFileSystem {
    fn create_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir_ref = dir.as_ref().to_path_buf();
        dbg!("Creating directory: {}", dir_ref.display());
        let mut directories = self.directories.blocking_write();
        directories.insert(dir_ref, ()); // Changed value to ()
        Ok(())
    }

    fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir_ref = dir.as_ref().to_path_buf();
        dbg!("Creating directory recursively: {}", dir_ref.display());
        let mut directories = self.directories.blocking_write();
        directories.insert(dir_ref, ()); // Changed value to ()
        Ok(())
    }

    fn write<P: AsRef<Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> Result<()> {
        let file_ref = file.as_ref().to_path_buf();
        dbg!("Writing to file: {}", file_ref.display());
        let mut files = self.files.blocking_write();
        files.insert(file_ref, data.as_ref().to_vec());
        Ok(())
    }
}

impl ReadableFileSystem for MockFileSystem {
    fn read<P: AsRef<Path>>(&self, file: P) -> Result<Vec<u8>> {
        let file_ref = file.as_ref().to_path_buf();
        dbg!("Reading file: {}", file_ref.display());
        let files = self.files.blocking_read();
        files.get(&file_ref).cloned().ok_or_else(|| rspack_fs::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")))
    }
}

impl AsyncWritableFileSystem for MockFileSystem {
    fn create_dir<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.as_ref().to_path_buf();
        dbg!("Async creating directory: {}", dir_ref.display());
        let directories = self.directories.clone();
        Box::pin(async move {
            let mut directories = directories.write().await;
            directories.insert(dir_ref, ()); // Changed value to ()
            Ok(())
        })
    }

    fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.as_ref().to_path_buf();
        dbg!("Async creating directory recursively: {}", dir_ref.display());
        let directories = self.directories.clone();
        Box::pin(async move {
            let mut directories = directories.write().await;
            directories.insert(dir_ref, ()); // Changed value to ()
            Ok(())
        })
    }

    fn write<P: AsRef<Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> BoxFuture<'_, Result<()>> {
        let file_ref = file.as_ref().to_path_buf();
        let data = data.as_ref().to_vec();
        dbg!("Async writing to file: {}", file_ref.display());
        let files = self.files.clone();
        Box::pin(async move {
            let mut files = files.write().await;
            files.insert(file_ref, data);
            Ok(())
        })
    }

    fn remove_file<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<()>> {
        let file_ref = file.as_ref().to_path_buf();
        dbg!("Async removing file: {}", file_ref.display());
        let files = self.files.clone();
        Box::pin(async move {
            let mut files = files.write().await;
            files.remove(&file_ref);
            Ok(())
        })
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.as_ref().to_path_buf();
        dbg!("Async removing directory recursively: {}", dir_ref.display());
        let directories = self.directories.clone();
        Box::pin(async move {
            let mut directories = directories.write().await;
            directories.remove(&dir_ref);
            Ok(())
        })
    }
}

impl AsyncReadableFileSystem for MockFileSystem {
    fn read<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<Vec<u8>>> {
        let file_ref = file.as_ref().to_path_buf();
        dbg!("Async reading file: {}", file_ref.display());
        let files = self.files.clone();
        Box::pin(async move {
            let files = files.read().await;
            files.get(&file_ref).cloned().ok_or_else(|| rspack_fs::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")))
        })
    }
}
