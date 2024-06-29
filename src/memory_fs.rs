#![deny(warnings)]

use std::{borrow::{Borrow}, collections::HashMap, path::Path, sync::RwLock};
use futures::future::BoxFuture;
use tokio::sync::RwLock as AsyncRwLock;
use rspack_fs::{
    sync::{ReadableFileSystem, WritableFileSystem},
    r#async::{AsyncReadableFileSystem, AsyncWritableFileSystem},
    Error, Result,
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

pub struct NativeFileSystem {
    files: RwLock<HashMap<String, Vec<u8>>>,
    directories: RwLock<HashMap<String, ()>>,
}


impl NativeFileSystem {
    #[allow(dead_code)]
    pub fn new() -> Self {
        dbg!("Creating new NativeFileSystem");
        Self {
            files: RwLock::new(HashMap::new()),
            directories: RwLock::new(HashMap::new()),
        }
    }
}

impl WritableFileSystem for NativeFileSystem {
  fn create_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
    let dir_ref = dir.as_ref().to_string_lossy().to_string();
    let dir_ref_clone = dir_ref.clone(); // Clone dir_ref before using it
    dbg!("Creating directory: {}", dir_ref);
    let mut directories = self.directories.write().unwrap();
    directories.insert(dir_ref_clone, ()); // Use the cloned version here
    Ok(())
}

    fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        dbg!("Creating directory recursively: {}", dir.as_ref().to_string_lossy());
        self.create_dir(dir)
    }

    fn write<P: AsRef<Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> Result<()> {
        let file = file.as_ref().to_string_lossy().to_string();
        let file_clone = file.clone(); // Clone the file variable
        dbg!("Writing to file: {}", file);
        let mut files = self.files.write().unwrap();
        files.insert(file_clone, data.as_ref().to_vec()); // Use the cloned version here
        Ok(())
    }
}

impl ReadableFileSystem for NativeFileSystem {
    fn read<P: AsRef<Path>>(&self, file: P) -> Result<Vec<u8>> {
      let file = file.as_ref().to_string_lossy().to_string();
      let file_clone = file.clone(); // Clone the file variable
      dbg!("Reading file: {}", file);
      let files = self.files.read().unwrap();
      files.get(&file_clone).cloned().ok_or_else(|| { // Use the cloned version here
          dbg!("File not found: {}", file_clone); // Use the cloned version here
          Error::from(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
      })
    }
}


pub struct AsyncNativeFileSystem {
    files: AsyncRwLock<HashMap<String, Vec<u8>>>,
    directories: AsyncRwLock<HashMap<String, ()>>,
}

impl AsyncNativeFileSystem {
    pub fn new() -> Self {
        dbg!("Creating new AsyncNativeFileSystem");
        Self {
            files: AsyncRwLock::new(HashMap::new()),
            directories: AsyncRwLock::new(HashMap::new()),
        }
    }
    pub fn get_instance(&self) -> &Self {
        self.borrow()
    }
  
    pub async fn get_resources(&self) -> (HashMap<String, Vec<u8>>, HashMap<String, ()>) {
      let files = self.files.read().await.clone();
      let directories = self.directories.read().await.clone();
      (files, directories)
  }

}

impl AsyncWritableFileSystem for AsyncNativeFileSystem {
    fn create_dir<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.as_ref().to_string_lossy().to_string();
        let dir_ref_clone = dir_ref.clone(); // Clone dir_ref before moving it into the closure
        dbg!("Async creating directory: {}", dir_ref);
        let fut = async move {
            let mut directories = self.directories.write().await;
            directories.insert(dir_ref_clone, ()); // Use the cloned version here
            Ok(())
        };
        Box::pin(fut)
    }

    fn create_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
        dbg!("Async creating directory recursively: {}", dir.as_ref().to_string_lossy());
        self.create_dir(dir)
    }

    // Inside the AsyncWritableFileSystem implementation for AsyncNativeFileSystem
    fn write<P: AsRef<Path>, D: AsRef<[u8]>>(&self, file: P, data: D) -> BoxFuture<'_, Result<()>> {
        let file = file.as_ref().to_string_lossy().to_string();
        let file_clone = file.clone(); // Clone the file variable
        dbg!("Async writing to file: {}", file);
        let data = data.as_ref().to_vec();
        let fut = async move {
            let mut files = self.files.write().await;
            files.insert(file_clone, data); // Use the cloned version here
            Ok(())
        };
        Box::pin(fut)
    }

    fn remove_file<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<()>> {
      let file = file.as_ref().to_string_lossy().to_string();
      let file_clone = file.clone(); // Clone the file variable
      dbg!("Async removing file: {}", file);
      let fut = async move {
          let mut files = self.files.write().await;
          files.remove(&file_clone).ok_or_else(|| { // Use the cloned version here
              dbg!("File not found: {}", file_clone); // Use the cloned version here
              Error::from(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
          })?;
          Ok(())
      };
      Box::pin(fut)
  }

    fn remove_dir_all<P: AsRef<Path>>(&self, dir: P) -> BoxFuture<'_, Result<()>> {
        let dir_ref = dir.as_ref().to_string_lossy().to_string();
        let dir_ref_clone = dir_ref.clone(); // Clone dir_ref before moving it into the closure
        dbg!("Async removing directory recursively: {}", dir_ref);
        let fut = async move {
            let mut directories = self.directories.write().await;
            directories.remove(&dir_ref_clone).ok_or_else(|| { // Use the cloned version here
                dbg!("Directory not found: {}", dir_ref_clone); // Use the cloned version here
                Error::from(std::io::Error::new(std::io::ErrorKind::NotFound, "Directory not found"))
            })?;
            Ok(())
        };
        Box::pin(fut)
    }
}

impl AsyncReadableFileSystem for AsyncNativeFileSystem {
    // Inside the AsyncReadableFileSystem implementation for AsyncNativeFileSystem
    fn read<P: AsRef<Path>>(&self, file: P) -> BoxFuture<'_, Result<Vec<u8>>> {
        let file = file.as_ref().to_string_lossy().to_string();
        let file_clone = file.clone(); // Clone the file variable
        dbg!("Async reading file: {}", file);
        let fut = async move {
            let files = self.files.read().await;
            files.get(&file_clone).cloned().ok_or_else(|| { // Use the cloned version here
                dbg!("File not found: {}", file_clone); // Use the cloned version here
                Error::from(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
            })
        };
        Box::pin(fut)
    }
}



