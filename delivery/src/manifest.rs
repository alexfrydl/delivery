use super::*;
use std::collections::BTreeMap;

/// Description of the files and directories in a package.
#[derive(Debug)]
pub struct Manifest {
  entries: Entries,
}

/// Map of relative paths to entries.
type Entries = BTreeMap<String, Entry>;

// An entry in a package manifest.
#[derive(Debug)]
enum Entry {
  /// A directory.
  Directory(Entries),
  /// A file identified by a hash of its contents.
  File(Hash),
}

#[derive(Debug)]
pub enum Change {
  RemoveDirectory(PathBuf),
  CreateDirectory(PathBuf),
  RemoveFile(PathBuf),
  CreateFile(PathBuf, Hash),
}

pub fn diff(before: &Manifest, after: &Manifest) -> Vec<Change> {
  let mut path_stack = PathBuf::new();
  let mut changes = default();

  diff_entries(
    &before.entries,
    &after.entries,
    &mut path_stack,
    &mut changes,
  );
  changes
}

fn diff_entries(
  before: &Entries,
  after: &Entries,
  path_stack: &mut PathBuf,
  changes: &mut Vec<Change>,
) {
  for (name, before_entry) in before {
    match (before_entry, after.get(name)) {
      (Entry::Directory(before_entries), Some(Entry::Directory(after_entries))) => {
        path_stack.push(name);
        diff_entries(before_entries, after_entries, path_stack, changes);
        path_stack.pop();
      }

      (Entry::Directory(_), Some(Entry::File(after_hash))) => {
        let path = path_stack.join(name);

        changes.push(Change::RemoveDirectory(path.clone()));
        changes.push(Change::CreateFile(path, after_hash.clone()));
      }

      (Entry::Directory(_), None) => {
        changes.push(Change::RemoveDirectory(path_stack.join(name)));
      }

      (Entry::File(_), Some(Entry::Directory(after_entries))) => {
        path_stack.push(name);
        changes.push(Change::RemoveFile(path_stack.clone()));
        diff_entries(&default(), after_entries, path_stack, changes);
        path_stack.pop();
      }

      (Entry::File(before_hash), Some(Entry::File(after_hash))) => {
        if after_hash != before_hash {
          changes.push(Change::CreateFile(
            path_stack.join(name),
            after_hash.clone(),
          ));
        }
      }

      (Entry::File(_), None) => {
        changes.push(Change::RemoveFile(path_stack.join(name)));
      }
    }
  }

  for (name, after_entry) in after {
    match (before.get(name), after_entry) {
      (None, Entry::Directory(after_entries)) => {
        path_stack.push(name);
        changes.push(Change::CreateDirectory(path_stack.clone()));
        diff_entries(&default(), after_entries, path_stack, changes);
        path_stack.pop();
      }

      (None, Entry::File(after_hash)) => {
        changes.push(Change::CreateFile(path_stack.clone(), after_hash.clone()));
      }

      _ => {}
    }
  }
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
      let file_type = fs_entry.file_type()?;

      let path = fs_entry.path();

      let name = match path.file_name().and_then(std::ffi::OsStr::to_str) {
        Some(n) => n.to_owned(),
        None => continue,
      };

      if file_type.is_file() {
        entries.insert(name, Entry::File(hasher.hash(path).await?));
      } else if file_type.is_dir() {
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
        Entry::File(hash) => write!(f, "\n{} {}", path.display(), hash)?,

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
