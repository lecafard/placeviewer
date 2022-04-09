use clap::Parser;
use log::{debug, info, warn};
use serde::Deserialize;
use std::{mem, slice};
use std::io::{self, BufWriter, Write};
use std::fs::File;

use crate::models::Placement;

#[derive(Parser)]
pub struct ParseCommand {
    // Output file
    #[clap(short, long, required(true))]
    output: String,

    #[clap(multiple_values(true))]
    inputs: Vec<String>,
}

impl ParseCommand {
  pub fn execute(&self) {
    let wf = File::create(self.output.clone()).unwrap();
    let mut writer = BufWriter::new(wf);
    for input in self.inputs.iter(){
      read_csv(input, &mut writer).unwrap();
    }
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


fn read_csv<T: Write>(filename_in: &String, writer: &mut BufWriter<T>) -> std::io::Result<()> {
  let mut reader = csv::ReaderBuilder::new()
    .delimiter(b',')
    .buffer_capacity(4 * (1 << 20)) // 4MB
    .from_path(filename_in)
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
      // pad out the first record to 16 bytes
      writer.write(&t0.to_le_bytes()).unwrap();
      writer.write(&(0 as u64).to_le_bytes()).unwrap();
    }
    
    let ts = (record.ts - t0) as u32;


    if !record.x2_coordinate.is_none() && !record.y2_coordinate.is_none() {
      for y in record.y_coordinate..=record.y2_coordinate.unwrap() {
        for x in record.x_coordinate..=record.x2_coordinate.unwrap() {
          write_placement(Placement {
            ts: ts,
            uid: record.user_id,
            x: x,
            y: y,
            color: record.color,
            isblk: true
          }, writer).unwrap();
        }
      }
    } else {
      write_placement(Placement {
        ts: ts,
        uid: record.user_id,
        x: record.x_coordinate,
        y: record.y_coordinate,
        color: record.color,
        isblk: false,
      }, writer).unwrap();
    }


    count += 1;
  }
  Ok(())
}

fn write_placement<T: Write>(placement: Placement, writer: &mut BufWriter<T>) -> io::Result<usize> {
  unsafe {
    let buffer = slice::from_raw_parts(
      &placement as *const Placement as *const u8,
      mem::size_of::<Placement>()
    );
    writer.write(buffer)
  }
}