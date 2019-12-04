use super::*;

#[derive(StructOpt)]
pub struct Command {
  #[structopt(help = "File names to hash.")]
  files: Vec<PathBuf>,
}

impl Command {
  pub async fn run(self) {
    let mut hasher = delivery::hash::AsyncHasher::new();

    for file in self.files {
      match hasher.hash(&file).await {
        Ok(hash) => println!("{}", hash),
        Err(err) => eprintln!("Failed to compute hash of file {:?}. {}", file, err),
      }
    }
  }
}
