use super::*;
use crate::manifest::{Entries, Entry};

/// A change operation returned by `diff()`.
#[derive(Debug)]
pub enum Change {
  RemoveDirectory(PathBuf),
  CreateDirectory(PathBuf),
  RemoveFile(PathBuf),
  CreateFile(PathBuf, Hash),
}

/// Builds a list of changes needed to make a given `before` manifest match a
/// given `after` manifest.
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

/// Builds a list of changes needed to make a given set of `before` entries
/// match a given set of `after` entries.
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
        changes.push(Change::CreateDirectory(path_stack.clone()));
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
        changes.push(Change::CreateFile(
          path_stack.join(name),
          after_hash.clone(),
        ));
      }

      _ => {}
    }
  }
}

impl fmt::Display for Change {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::CreateDirectory(path) => write!(f, "mkdir {}", path.display()),
      Self::CreateFile(path, hash) => write!(f, "create {} {}", path.display(), hash),
      Self::RemoveDirectory(path) => write!(f, "rmdir {}", path.display()),
      Self::RemoveFile(path) => write!(f, "rm {}", path.display()),
    }
  }
}
