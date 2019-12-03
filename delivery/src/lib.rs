mod hash;

use async_std::io::prelude::*;
use async_std::path::Path;
use async_std::{fs, io};

pub use self::hash::hash;

/// Shorthand function equivalent to `T::default()`.
fn default<T: Default>() -> T {
  T::default()
}
