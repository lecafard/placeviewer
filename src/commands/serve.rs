use clap::Parser;

#[derive(Parser)]
pub struct ServeCommand {
  // Port to listen on
  #[clap(short, long, default_value_t = 3000)]
  port: i16,
  // Host to listen on
  #[clap(short, long, default_value = "localhost")]
  host: String,
}

impl ServeCommand {
  pub fn execute(&self) {
    println!("serve")
  }
}