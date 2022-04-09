use clap::Parser;
use serde::Deserialize;

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
    for input in self.inputs.iter(){
      read_csv(input).unwrap();
    }
  }
}

#[derive(Debug, Deserialize)]
struct Record {
  ts: u64,
  user_id: u32,
  x_coordinate: u16,
  y_coordinate: u16,
  x2_coordinate: Option<u16>,
  y2_coordinate: Option<u16>,
  color: u8
}

#[derive(Debug)]
struct CompressedRecord {
  ts: u32,
  uid: u32,
  x: u16,
  y: u16,
  color: u8,
  isblk: bool,
}


fn read_csv(filename_in: &String) -> std::io::Result<()> {
  let mut reader = csv::ReaderBuilder::new()
    .delimiter(b',')
    .buffer_capacity(4 * (1 << 20)) // 4MB
    .from_path(filename_in)
    .expect("oops");

  let mut t0: u64 = 0;
  let mut count = 0;
  for result in reader.deserialize() {
    let record: Record = result?;
    
    if count % 1000000 == 0 {
      println!("Processed {} records", count);
    }

    if t0 == 0 {
      t0 = record.ts;
    }
    
    let ts = (record.ts - t0) as u32;

    if !record.x2_coordinate.is_none() && !record.y2_coordinate.is_none() {
      for y in record.y_coordinate..=record.y2_coordinate.unwrap() {
        for x in record.x_coordinate..=record.x2_coordinate.unwrap() {
          let compressed_record = CompressedRecord {
            ts: ts,
            uid: record.user_id,
            x: x,
            y: y,
            color: record.color,
            isblk: true
          };
          println!("{:?} {}", compressed_record, std::mem::size_of::<CompressedRecord>());
        }
      }
    } else {
      let compressed_record = CompressedRecord {
        ts: ts,
        uid: record.user_id,
        x: record.x_coordinate,
        y: record.y_coordinate,
        color: record.color,
        isblk: true,
      };
    }


    count += 1;
  }
  Ok(())
}