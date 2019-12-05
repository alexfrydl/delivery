use super::*;

#[derive(StructOpt)]
pub struct Command {
  #[structopt(help = "Path to the “before” directory.")]
  before: PathBuf,
  #[structopt(help = "Path to the “after” directory.")]
  after: PathBuf,
}

impl Command {
  pub async fn run(self) {
    let before = match delivery::manifest::compile(&self.before).await {
      Ok(b) => b,

      Err(err) => {
        eprintln!(
          "A manifest could not be compiled for the given “before” directory. {}",
          err
        );

        exit(1);
      }
    };

    let after = match delivery::manifest::compile(&self.after).await {
      Ok(b) => b,

      Err(err) => {
        eprintln!(
          "A manifest could not be compiled for the given “after” directory. {}",
          err
        );

        exit(1);
      }
    };

    for change in delivery::diff(&before, &after) {
      println!("{}", change);
    }
  }
}
