use super::*;

use async_std::{sync, task};
use digest::Digest as _;
use std::hash::Hasher as _;
use std::thread;

/// Computes the content hash of the file at the given path
///
/// The hash is returned as a URL-safe base64-encoded string
pub fn hash(path: impl AsRef<Path>) -> io::Result<String> {
  // Open file and read metadata.
  let mut file = fs::File::open(path.as_ref())?;
  let metadata = file.metadata()?;

  if !metadata.is_file() {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a file."));
  }

  // Use meowhash on large files and seahash on small files.
  let mut hasher = if metadata.len() > 1024 {
    Hash::Meow(default())
  } else {
    Hash::Sea(default())
  };

  // Write file contents to hasher.
  let mut buffer = [0u8; 1024];

  loop {
    use io::Read;

    let n = file.read(&mut buffer)?;

    if n == 0 {
      break;
    }

    hasher.write(&mut buffer[0..n]);
  }

  // Compute encoded hash.
  Ok(hasher.finish())
}

/// Asynchronously computes the content hash of files using a worker thread.
#[derive(Clone)]
pub struct AsyncHasher {
  request_sender: sync::Sender<(PathBuf, sync::Sender<io::Result<String>>)>,
}

impl Default for AsyncHasher {
  fn default() -> Self {
    let (request_sender, requests) = sync::channel(8);
    let hasher = Self { request_sender };

    thread::spawn(move || {
      task::block_on(async move {
        while let Some((path, result_sender)) = requests.recv().await {
          result_sender.send(hash(path)).await;
        }
      })
    });

    hasher
  }
}

impl AsyncHasher {
  pub fn new() -> Self {
    default()
  }

  /// Compute the content hash of the file at the given path.
  ///
  /// The hash is returned as a URL-safe base64-encoded string
  pub async fn hash(&mut self, path: impl Into<PathBuf>) -> io::Result<String> {
    let (result_sender, result) = sync::channel(8);

    self.request_sender.send((path.into(), result_sender)).await;

    result.recv().await.expect("failed awaiting hash result")
  }
}

/// Either a meowhash or seahash hasher.
enum Hash {
  Meow(Box<meowhash::MeowHasher>),
  Sea(seahash::SeaHasher),
}

impl Hash {
  /// Write bytes to the hasher.
  fn write(&mut self, bytes: impl AsRef<[u8]>) {
    match self {
      Self::Meow(hasher) => hasher.input(bytes),
      Self::Sea(hasher) => hasher.write(bytes.as_ref()),
    }
  }

  /// Finish the hash and return it as an encoded string.
  fn finish(self) -> String {
    match self {
      Self::Meow(hasher) => {
        // Use only the first 288 bits of meowhash results (48 character string
        // when encoded).
        base64::encode_config(&hasher.result()[0..36], base64::URL_SAFE_NO_PAD)
      }

      Self::Sea(hasher) => {
        base64::encode_config(&hasher.finish().to_le_bytes(), base64::URL_SAFE_NO_PAD)
      }
    }
  }
}
