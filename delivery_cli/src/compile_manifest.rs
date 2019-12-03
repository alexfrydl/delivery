use super::*;

#[derive(StructOpt)]
pub struct Command {
  #[structopt(help = "Directory to compile a manifest for.")]
  directory: PathBuf,
  #[structopt(long, help = "Formats the manifest for human consumption.")]
  pretty: bool,
}

impl Command {
  pub async fn run(self) {
    match delivery::manifest::compile(self.directory).await {
      Ok(manifest) => {
        if self.pretty {
          println!("{:#}", manifest)
        } else {
          println!("{}", manifest)
        }
      }

      Err(err) => {
        eprintln!("Failed to compile manifest. {}", err);
        exit(1);
      }
    }
  }
}
