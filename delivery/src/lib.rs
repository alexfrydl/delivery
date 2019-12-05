pub mod diff;
pub mod hash;
pub mod manifest;

use derive_more::*;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::{fmt, fs, io};

pub use self::diff::diff;
pub use self::hash::{hash, Hash};
pub use self::manifest::Manifest;

/// Shorthand function equivalent to `T::default()`.
fn default<T: Default>() -> T {
  T::default()
}
