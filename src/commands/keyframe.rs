use clap::Parser;
use log::error;

#[derive(Parser)]
pub struct KeyframeCommand {
  #[clap(required=true)]
  interval: u32,

  // Tile size, tiles are square
  #[clap(required=true, min_values=1)]
  inputs: Vec<String>,
}

impl KeyframeCommand {
  pub fn execute(&self) {
    error!("Not yet implemented")
  }
}