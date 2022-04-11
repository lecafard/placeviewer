use log::debug;
use memmap::{Mmap, MmapOptions};
use serde::Deserialize;
use std::{mem, ptr, slice};
use std::fs::File;
use std::marker::PhantomData;

use crate::models::record::{TileHeader, Placement};

#[derive(Debug, Deserialize)]
pub struct Root {
  pub datasets: Vec<SerializedDataset>,
}

#[derive(Debug, Deserialize)]
pub struct SerializedDataset {
  pub name: String,
  pub tiles: Vec<String>,
  pub palette: Vec<u32>,
  pub size_x: u16,
  pub size_y: u16,
  pub size_tile: u16,
  pub interval: u32
}

#[derive(Debug)]
pub struct Dataset {
  pub name: String,
  pub palette: Vec<u8>,
  pub size_x: u16,
  pub size_y: u16,
  pub size_tile: u16,
  pub tiles: Vec<Tile>,
}

#[derive(Debug)]
pub struct Tile {
  pub start: u64,
  pub count: u32,
  pub start_x: u16,
  pub start_y: u16,
  pub size: u16,
  mmap: Mmap,
}

impl SerializedDataset {
  pub fn load(&self) -> Dataset {
    let palette: Vec<u8> = self.palette.clone().into_iter()
      .flat_map(|v| {
        [
          (v >> 16 & 0xff) as u8,
          (v >>  8 & 0xff) as u8,
          (v       & 0xff) as u8
        ]
      })
      .collect();

    let tiles_x = (self.size_x / self.size_tile) as usize;
    let tiles_y = (self.size_y / self.size_tile) as usize;

    let mut dataset = Dataset {
      name: self.name.clone(),
      palette: palette,
      size_x: self.size_x,
      size_y: self.size_y,
      size_tile: self.size_tile,
      tiles: Vec::with_capacity(tiles_x * tiles_y),
    };

    for filename in self.tiles.iter() {
      let file = File::open(filename).unwrap();
      let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
      let header: TileHeader = unsafe { ptr::read(mmap.as_ptr() as *const _) };
      debug!("Header: {:?}", header);
      if header.version != 0x6969 {
        panic!("header version is wrong");
      }
      if header.size != self.size_tile {
        panic!("header size does not match tile size");
      }


      dataset.tiles.push(Tile{
        start: header.start,
        count: header.count,
        start_x: header.start_x,
        start_y: header.start_y,
        size: header.size,
        mmap: mmap,
      });
    }

    return dataset;
  }
}

impl Tile {
  pub fn placements(&self) -> &[Placement<Tile>] {
    return unsafe {
      slice::from_raw_parts(
        self.mmap.as_ptr().offset(mem::size_of::<TileHeader>() as isize) as *const _,
        self.count as usize
      )
    };
  }
}