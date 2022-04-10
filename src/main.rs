mod commands;
mod models;
mod store;

use clap::Parser;

use crate::commands::{run_command, SubCommand};


/// reddit place viewer
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
  #[clap(subcommand)]
  command: SubCommand,
}



fn main() {
  env_logger::init();
  let args = Args::parse();
  run_command(args.command);
}