use super::*;
use std::collections::BTreeMap;

/// Description of the files and directories in a package.
#[derive(Debug)]
pub struct Manifest {
  entries: Entries,
}

/// Map of relative paths to entries.
type Entries = BTreeMap<PathBuf, Entry>;

// An entry in a package manifest.
#[derive(Debug)]
enum Entry {
  /// A directory.
  Directory,
  /// A file identified by a hash of its contents.
  File(String),
}

/// Compiles a manifest of a directory.
pub async fn compile(path: impl AsRef<Path>) -> io::Result<Manifest> {
  let path = path.as_ref();
  let mut entries = default();

  compile_into(&path, &path, &mut entries).await?;

  Ok(Manifest { entries })
}

/// Creates manifest entries for the contents of a directory and its
/// subdirectories.
fn compile_into<'a>(
  root_path: &'a Path,
  path: &'a Path,
  entries: &'a mut Entries,
) -> Pin<Box<dyn Future<Output = io::Result<()>> + 'a>> {
  Box::pin(async move {
    let mut fs_entries = fs::read_dir(path).await?;

    while let Some(fs_entry) = fs_entries.next().await {
      let fs_entry = fs_entry?;
      let file_type = fs_entry.file_type().await?;

      if file_type.is_symlink() {
        continue;
      }

      let path = fs_entry.path();
      let relative_path = path.strip_prefix(root_path).unwrap().into();

      if file_type.is_file() {
        entries.insert(relative_path, Entry::File(hash(path).await?));
      } else {
        entries.insert(relative_path, Entry::Directory);

        compile_into(root_path, path.as_ref(), entries).await?;
      }
    }

    Ok(())
  })
}

// Implement `fmt::Display` for serializing the manifest.
//
// Use the alternate format flag (`#`) for pretty formatting.
impl fmt::Display for Manifest {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if f.alternate() {
      write!(f, "{:>48} 1", "version")?;

      for (path, entry) in &self.entries {
        match entry {
          Entry::File(hash) => write!(f, "\n{:>48} {}", hash, path.display())?,
          Entry::Directory => write!(f, "\n{:>48} {}/", "", path.display())?,
        }
      }
    } else {
      write!(f, "version 1")?;

      for (path, entry) in &self.entries {
        match entry {
          Entry::File(hash) => write!(f, "\n{} {}", hash, path.display())?,
          Entry::Directory => write!(f, "\n{}/", path.display())?,
        }
      }
    }

    Ok(())
  }
}
