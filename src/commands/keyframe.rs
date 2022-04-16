use clap::Parser;
use log::{info, warn};
use std::io::BufWriter;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::thread::sleep_ms;
use regex::Regex;
use tokio::runtime::Runtime;

use crate::store::config::Tile;
use crate::models::record::{TileKeyframeHeader, write_record, TILE_KEYFRAME_VERSION_ID};

const REGEX_LOG: &str = r"^([A-Za-z0-9-]+)_log_([0-9]+_[0-9]+).bin$";

#[derive(Parser, Clone)]
pub struct KeyframeCommand {
  #[clap(required=true)]
  interval: u32,

  // Tile size, tiles are square
  #[clap(required=true, min_values=1)]
  inputs: Vec<String>,
}

impl KeyframeCommand {
  pub fn execute(&self) {
    let rt = Runtime::new().unwrap();
    for input in self.inputs.iter() {
      rt.spawn(export(self.clone(), String::from(input)));
    }
    // TODO: fix this hack
    sleep_ms(10000);
  }
}


async fn export(cmd: KeyframeCommand, input: String) {
  let re = Regex::new(REGEX_LOG).unwrap();
  let path = Path::new(&input);
  let filename = path.file_name().unwrap().to_str().unwrap();
  let capture = match re.captures(filename) {
    Some(c) => c,
    None => {
      warn!("unable to match filename for {}", filename);
      return
    }
  };

  let tile = Tile::load(&input).unwrap();

  let name = capture.get(1).map_or("", |m| m.as_str());
  let position = capture.get(2).map_or("", |m| m.as_str());
  let out_path = path.parent().unwrap().join(format!("{}_frame_{}.bin", name, position));
  let fw = File::create(&out_path).unwrap();
  let mut w = BufWriter::new(fw);

  let header = TileKeyframeHeader {
    version: TILE_KEYFRAME_VERSION_ID,
    size: tile.size,
    start_x: tile.start_x,
    start_y: tile.start_y,
    interval: cmd.interval,
    count: if tile.count % cmd.interval == 0 { tile.count/cmd.interval }
      else { (tile.count/cmd.interval) + 1 }
  };
  info!("Writing out {:?} with header {:?}", out_path, header);
  write_record(&header, &mut w).unwrap();
  
  let mut output: Vec<u8> = vec![1; tile.size as usize * tile.size as usize];
  w.write(&output).unwrap();
  for i in tile.placements().chunks(cmd.interval as usize) {
    tile.apply(&mut output, i);
    w.write(&output).unwrap();
  }
}