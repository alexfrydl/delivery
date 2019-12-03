mod hash;

use async_std::task;
use std::path::PathBuf;
use structopt::*;

#[derive(StructOpt)]
pub enum Command {
  #[structopt(about = "Computes a unique hash of file contents.")]
  Hash(hash::Command),
}

pub fn main() {
  task::block_on(async {
    let command = Command::from_args();

    match command {
      Command::Hash(cmd) => cmd.run().await,
    }
  });
}
