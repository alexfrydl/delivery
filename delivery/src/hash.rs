use super::*;

use digest::Digest as _;
use std::hash::Hasher as _;

/// Computes a hash from the contents of the file at the given path.
///
/// The hash is returned as a URL-safe base64-encoded string
pub async fn hash(path: impl AsRef<Path>) -> io::Result<String> {
  // Open file and read metadata.
  let mut file = fs::File::open(path).await?;
  let metadata = file.metadata().await?;

  if !metadata.is_file() {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a file."));
  }

  // Use meowhash on large files and seahash on small files.
  let mut hasher = if metadata.len() > 1024 {
    Hasher::Meow(default())
  } else {
    Hasher::Sea(default())
  };

  // Write file contents to hasher.
  let mut buffer = [0u8; 1024];

  loop {
    let n = file.read(&mut buffer).await?;

    if n == 0 {
      break;
    }

    hasher.write(&mut buffer[0..n]);
  }

  // Compute encoded hash.
  Ok(hasher.finish())
}

/// Either a meowhash or seahash hasher.
enum Hasher {
  Meow(Box<meowhash::MeowHasher>),
  Sea(seahash::SeaHasher),
}

impl Hasher {
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
