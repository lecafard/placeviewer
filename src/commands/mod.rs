use clap::Parser;

pub mod keyframe;
pub mod parse;
pub mod serve;

#[derive(Parser)]
pub enum SubCommand {
  Keyframe(keyframe::KeyframeCommand),
  Parse(parse::ParseCommand),
  Serve(serve::ServeCommand),
}

pub fn run_command(sub: SubCommand) {
  match sub {
    SubCommand::Keyframe(cmd) => cmd.execute(),
    SubCommand::Parse(cmd) => cmd.execute(),
    SubCommand::Serve(cmd) => cmd.execute()
  }
}