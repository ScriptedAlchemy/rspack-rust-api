#![deny(warnings)]

use std::fs;
use tokio::fs as tokio_fs;
use futures::future::BoxFuture;
use rspack_fs::{
    r#async::{AsyncReadableFileSystem, AsyncWritableFileSystem},
    sync::{ReadableFileSystem, WritableFileSystem},
    Result,
};
use rspack_paths::Utf8Path;
use std::path::{Path, PathBuf};
use std::fs::Metadata;

#[derive(Clone)]
pub struct RealFileSystem;

impl RealFileSystem {
    pub fn new() -> Self {
        dbg!("Creating new RealFileSystem");
        Self
    }
}

impl WritableFileSystem for RealFileSystem {
    fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
        let dir_ref = dir.to_path_buf();
        dbg!(&dir_ref);
        fs::create_dir(&dir_ref)?;
        Ok(())
    }

    fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
        let dir_ref = dir.to_path_buf();
        dbg!(&dir_ref);
        fs::create_dir_all(&dir_ref)?;
        Ok(())
    }

    fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
        let file_ref = file.to_path_buf();
        dbg!(&file_ref);
        fs::write(&file_ref, data)?;
        Ok(())
    }
}

impl ReadableFileSystem for RealFileSystem {
    fn read(&self, file: &Path) -> std::io::Result<Vec<u8>> {
        let data = fs::read(file)?;
        Ok(data)
    }

    fn metadata(&self, path: &Path) -> std::io::Result<Metadata> {
        fs::metadata(path)
    }

    fn symlink_metadata(&self, path: &Path) -> std::io::Result<Metadata> {
        fs::symlink_metadata(path)
    }

    fn canonicalize(&self, path: &Path) -> std::io::Result<PathBuf> {
        fs::canonicalize(path)
    }
}

impl AsyncWritableFileSystem for RealFileSystem {
    fn create_dir(&self, dir: &Utf8Path) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.to_path_buf();
        dbg!(&dir_ref);
        Box::pin(async move {
            tokio_fs::create_dir(&dir_ref).await?;
            Ok(())
        })
    }

    fn create_dir_all(&self, dir: &Utf8Path) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.to_path_buf();
        dbg!(&dir_ref);
        Box::pin(async move {
            tokio_fs::create_dir_all(&dir_ref).await?;
            Ok(())
        })
    }

    fn write(&self, file: &Utf8Path, data: &[u8]) -> BoxFuture<'_, Result<()>> {
        let file_ref = file.to_path_buf();
        let data = data.to_vec();
        dbg!(&file_ref);
        Box::pin(async move {
            tokio_fs::write(&file_ref, &data).await?;
            Ok(())
        })
    }

    fn remove_file(&self, file: &Utf8Path) -> BoxFuture<'_, Result<()>> {
        let file_ref = file.to_path_buf();
        dbg!(&file_ref);
        Box::pin(async move {
            tokio_fs::remove_file(&file_ref).await?;
            Ok(())
        })
    }

    fn remove_dir_all(&self, dir: &Utf8Path) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.to_path_buf();
        dbg!(&dir_ref);
        Box::pin(async move {
            tokio_fs::remove_dir_all(&dir_ref).await?;
            Ok(())
        })
    }
}

impl AsyncReadableFileSystem for RealFileSystem {
    fn read(&self, file: &Utf8Path) -> BoxFuture<'_, rspack_fs::Result<Vec<u8>>> {
        let file_ref = file.to_path_buf();
        dbg!(&file_ref);
        Box::pin(async move {
            let data = tokio_fs::read(&file_ref).await?;
            Ok(data)
        })
    }
}