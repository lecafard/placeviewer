pub mod config;

use memmap::Mmap;
use std::sync::Arc;
use std::{mem, ptr, slice};
use crate::models::record::TileHeader;

use crate::models::record::Placement;

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
  pub mmap: Arc<Mmap>,
}

impl Tile {
  pub fn placements<'a>(&self) -> &'a [Placement] {
    return unsafe {
      slice::from_raw_parts(
        self.mmap.clone().as_ptr().offset(mem::size_of::<TileHeader>() as isize) as *const _,
        self.count as usize
      )
    };
  }
}