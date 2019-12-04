mod compile_manifest;
mod hash;

use std::path::PathBuf;
use std::process::exit;
use structopt::*;

#[derive(StructOpt)]
pub enum Command {
  #[structopt(about = "Compiles a manifest of directory contents.")]
  CompileManifest(compile_manifest::Command),
  #[structopt(about = "Computes a unique hash of file contents.")]
  Hash(hash::Command),
}

pub fn main() {
  futures::executor::block_on(async {
    let command = Command::from_args();

    match command {
      Command::CompileManifest(cmd) => cmd.run().await,
      Command::Hash(cmd) => cmd.run().await,
    }
  });
}
