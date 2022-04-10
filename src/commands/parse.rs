use clap::Parser;
use log::{error, info, warn};
use serde::Deserialize;
use std::{mem, slice};
use std::io::{self, BufWriter, Write};
use std::fs::File;

use crate::models::record::{TileHeader, Placement, Record};

#[derive(Parser)]
pub struct ParseCommand {
    // Input CSV
    #[clap(required=true)]
    input: String,

    // Output Prefix
    #[clap(required=true)]
    output_prefix: String,

    // X Size of full canvas
    #[clap(required=true)]
    size_x: u16,

    // Y Size of full canvas
    #[clap(required=true)]
    size_y: u16,
    
    // Tile size, tiles are square
    #[clap(required=true)]
    size_tile: u16,
}

impl ParseCommand {
  pub fn execute(&self) {
    read_csv(&self.input, &self.output_prefix, self.size_x, self.size_y, self.size_tile).unwrap();
  }
}

#[derive(Debug, Deserialize)]
struct CSVRecord {
  ts: u64,
  user_id: u32,
  x_coordinate: u16,
  y_coordinate: u16,
  x2_coordinate: Option<u16>,
  y2_coordinate: Option<u16>,
  color: u8
}


fn read_csv(
  input: &String,
  output_prefix: &String,
  size_x: u16,
  size_y: u16,
  size_tile: u16
) -> Result<(), ()> {
  info!("{}", mem::size_of::<TileHeader>());
  if size_x == 0 || size_y == 0 || size_x % size_tile != 0 || size_y % size_tile != 0 {
    error!("The size of the canvas must be divisible by the tile size");
    return Err(())
  }
  let tiles_x = size_x / size_tile;
  let tiles_y = size_y / size_tile;
  let mut tiles: Vec<TileHeader> = Vec::with_capacity((tiles_x * tiles_y) as usize);
  let mut placements: Vec<Vec<Placement>> = Vec::with_capacity((tiles_x * tiles_y) as usize);

  for ty in 0..tiles_y {
    for tx in 0..tiles_x {
      tiles.push(TileHeader{
        size: size_tile,
        start_x: tx * size_tile,
        start_y: ty * size_tile,
        start: 0,
        count: 0,
        version: 0x6969,
      });
      placements.push(Vec::with_capacity(10000));
    }
  }

  let mut reader = csv::ReaderBuilder::new()
    .delimiter(b',')
    .buffer_capacity(4 * (1 << 20)) // 4MB
    .from_path(input)
    .expect("oops");

  let mut first = false;
  let mut t0: u64 = 0;
  let mut count = 0;
  for result in reader.deserialize() {
    let record = match result as Result<CSVRecord, csv::Error> {
      Ok (r) => r,
      Err (err) => {
        warn!("error processing record: {}", err);
        continue
      }, 
    };
    
    if count % 1000000 == 0 {
      info!("Processed {} records", count);
    }

    if !first {
      first = true;
      t0 = record.ts;
      for tile in tiles.iter_mut() {
        tile.start = t0;
      }
    }
    
    let ts = (record.ts - t0) as u32;
    if !record.x2_coordinate.is_none() && !record.y2_coordinate.is_none() {
      for y in record.y_coordinate..=record.y2_coordinate.unwrap() {
        for x in record.x_coordinate..=record.x2_coordinate.unwrap() {
          let tile_x = x / size_tile;
          let tile_y = y / size_tile;
          if tile_x >= tiles_x || tile_y >= tiles_y {
            warn!("position {},{} does not belong to a tile", x, y);
            continue;
          }
          let placement = Placement {
            ts: ts,
            uid: record.user_id,
            x: x - tile_x * size_tile,
            y: y - tile_y * size_tile,
            color: record.color,
            isblk: true,
          };
          placements[(tile_y * tiles_x + tile_x) as usize].push(placement);
        }
      }
    } else {
      let tile_x = record.x_coordinate / size_tile;
      let tile_y = record.y_coordinate / size_tile;
      if tile_x >= tiles_x || tile_y >= tiles_y {
        warn!("position {},{} does not belong to a tile", record.x_coordinate, record.y_coordinate);
        continue;
      }
      let placement = Placement {
        ts: ts,
        uid: record.user_id,
        x: record.x_coordinate - tile_x * size_tile,
        y: record.y_coordinate - tile_y * size_tile,
        color: record.color,
        isblk: false,
      };
      placements[(tile_y * tiles_x + tile_x) as usize].push(placement);
    }
    count += 1;
  }

  for ty in 0..tiles_y {
    for tx in 0..tiles_x {
      let idx = (tx + ty * tiles_x) as usize;
      tiles[idx].count = placements[idx].len() as u32;
      let filename = format!("{}_log_{}_{}.bin", output_prefix, tx, ty);
      let fw = File::create(filename).unwrap();
      let mut w = BufWriter::new(fw);
      write_record(&TileHeader{
        count: tiles[idx].count,
        start: tiles[idx].start,
        size: tiles[idx].size,
        start_x: tiles[idx].start_x,
        start_y: tiles[idx].start_y,
        version: 0x6969,
      }, &mut w).unwrap();
      for p in placements[idx].iter() {
        write_record(p, &mut w).unwrap();
      }
    }
  }

  Ok(())
}

fn write_record<S: Record, T: Write>(data: &S, writer: &mut BufWriter<T>) -> io::Result<usize> {
  unsafe {
    let buffer = slice::from_raw_parts(
      data as *const S as *const u8,
      mem::size_of::<S>()
    );
    writer.write(buffer)
  }
}