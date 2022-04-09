use clap::Parser;

pub mod parse;
pub mod serve;

#[derive(Parser)]
pub enum SubCommand {
  Parse(parse::ParseCommand),
  Serve(serve::ServeCommand),
}

pub fn run_command(sub: SubCommand) {
  match sub {
    SubCommand::Parse(cmd) => cmd.execute(),
    SubCommand::Serve(cmd) => cmd.execute()
  }
}