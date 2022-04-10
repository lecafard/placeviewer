use clap::Parser;
use log::{info};
use memmap::MmapOptions;
use serde::Deserialize;
use std::{mem, ptr, slice};
use std::fs::{File, read_to_string};
use std::io::BufWriter;
use std::time::Instant;
use yaml_rust::YamlLoader;

use crate::models::{Header, Placement};

#[derive(Parser)]
pub struct ServeCommand {
  // Port to listen on
  #[clap(long, default_value_t = 3000)]
  port: i16,
  
  // Host to listen on
  #[clap(long, default_value = "localhost")]
  host: String,

  // Tile data
  #[clap(required=true)]
  config_file: String
}

#[derive(Debug, Deserialize)]
struct Config {
  datasets: Vec<Dataset>,
}

#[derive(Debug, Deserialize)]
struct Dataset {
  name: String,
  tiles: Vec<String>,
  palette: Vec<u32>,
  size_x: u16,
  size_y: u16,
  size_tile: u16,
  interval: u32
}

impl ServeCommand {
  pub fn execute(&self) {
    let config_str = read_to_string(&self.config_file).unwrap();
    let config: Config = serde_yaml::from_str(&config_str).unwrap();
    info!("{:?}", config);
    for dataset in config.datasets.iter() {
      let palette: Vec<u8> = dataset.palette.clone().into_iter()
        .flat_map(|v| {
          [
            (v >> 16 & 0xff) as u8,
            (v >>  8 & 0xff) as u8,
            (v       & 0xff) as u8
          ]
         })
        .collect();

      for (i, filename) in dataset.tiles.iter().enumerate() {
        let file = File::open(filename).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let header: Header = unsafe { ptr::read(mmap.as_ptr() as *const _) };
        info!("Header: {:?}", header);
        let placements: &[Placement] = unsafe {
          slice::from_raw_parts(
            mmap.as_ptr().offset(mem::size_of::<Header>() as isize) as *const _,
            header.count as usize
          )
        };

        let now = Instant::now();
        let mut data: Vec<u8> = vec![0; header.size as usize * header.size as usize];
        for p in placements.iter() {
          let i = p.x as usize + (p.y as usize * header.size as usize);
          data[i] = p.color;
        }

        println!("Frame took {:?} seconds to render", now.elapsed());

        let file = File::create(format!("{}_{}.png", dataset.name, i)).unwrap();
        let w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, header.size as u32, header.size as u32);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_palette(&palette);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&data).unwrap();
      }
    }
  }
}