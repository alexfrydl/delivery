pub mod manifest;

mod hash;

use async_std::io::prelude::*;
use async_std::prelude::*;
use async_std::{fs, io};
use std::fmt;
use std::path::{Path, PathBuf};
use std::pin::Pin;

pub use self::hash::hash;
pub use self::manifest::Manifest;

/// Shorthand function equivalent to `T::default()`.
fn default<T: Default>() -> T {
  T::default()
}
