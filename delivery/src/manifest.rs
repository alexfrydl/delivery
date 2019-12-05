use super::*;
use std::collections::BTreeMap;

/// Description of the files and directories in a package.
#[derive(Debug)]
pub struct Manifest {
  pub entries: Entries,
}

/// Map of relative paths to manifest entries.
pub type Entries = BTreeMap<String, Entry>;

// An entry in a package manifest.
#[derive(Debug)]
pub enum Entry {
  /// A directory.
  Directory(Entries),
  /// A file identified by a hash of its contents.
  File { size: u64, hash: Hash },
}

/// Compiles a manifest of a directory.
pub async fn compile(path: impl AsRef<Path>) -> io::Result<Manifest> {
  let mut hasher = hash::AsyncHasher::new();
  let entries = compile_entries(path.as_ref(), &mut hasher).await?;

  Ok(Manifest { entries })
}

/// Creates manifest entries for the contents of a directory and its
/// subdirectories.
fn compile_entries<'a>(
  root_path: &'a Path,
  hasher: &'a mut hash::AsyncHasher,
) -> Pin<Box<dyn Future<Output = io::Result<Entries>> + 'a>> {
  Box::pin(async move {
    let mut entries = Entries::default();

    for fs_entry in fs::read_dir(root_path)? {
      let fs_entry = fs_entry?;
      let metadata = fs_entry.metadata()?;

      let path = fs_entry.path();

      let name = match path.file_name().and_then(std::ffi::OsStr::to_str) {
        Some(n) => n.to_owned(),
        None => continue,
      };

      if metadata.is_file() {
        entries.insert(
          name,
          Entry::File {
            size: metadata.len(),
            hash: hasher.hash(path).await?,
          },
        );
      } else if metadata.is_dir() {
        entries.insert(
          name,
          Entry::Directory(compile_entries(&path, hasher).await?),
        );
      }
    }

    Ok(entries)
  })
}

// Implement `fmt::Display` for serializing the manifest.
impl fmt::Display for Manifest {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "version 1")?;

    let mut entries_stack: Vec<_> = self
      .entries
      .iter()
      .rev()
      .map(|(name, entry)| (PathBuf::from(name), entry))
      .collect();

    while let Some((path, entry)) = entries_stack.pop() {
      match entry {
        Entry::File { hash, size } => write!(f, "\n{} {} {}", path.display(), size, hash)?,

        Entry::Directory(entries) if entries.is_empty() => write!(f, "\n{}/", path.display())?,

        Entry::Directory(entries) => {
          entries_stack.extend(
            entries
              .iter()
              .rev()
              .map(|(name, entry)| (path.join(name), entry)),
          );
        }
      }
    }

    Ok(())
  }
}
