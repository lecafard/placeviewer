use std::io::{self, BufWriter, Write};
use std::marker::PhantomData;
use std::mem;
use std::slice;

pub const TILE_PLACEMENT_VERSION_ID: u16 = 0x4200;
pub const TILE_KEYFRAME_VERSION_ID: u16 = 0x6900;

pub trait Record {}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Placement<T> {
  pub ts: u32,
  pub uid: u32,
  pub x: u16,
  pub y: u16,
  pub color: u8,
  pub isblk: bool,
  pub marker: PhantomData<*const T>
}

#[derive(Debug)]
pub struct TilePlacementHeader {
  pub version: u16,
  pub size: u16,
  pub start_x: u16,
  pub start_y: u16,
  pub start: u64,
  pub count: u32,
  pub uid_count: u32,
}

#[derive(Debug)]
pub struct TileKeyframeHeader {
  pub version: u16,
  pub size: u16,
  pub start_x: u16,
  pub start_y: u16,
  pub interval: u32,
  pub count: u32,
}

impl<T> Record for Placement<T> {}
impl Record for TileKeyframeHeader {}
impl Record for TilePlacementHeader {}

pub fn write_record<S: Record, T: Write>(data: &S, writer: &mut BufWriter<T>) -> io::Result<usize> {
  unsafe {
    let buffer = slice::from_raw_parts(
      data as *const S as *const u8,
      mem::size_of::<S>()
    );
    writer.write(buffer)
  }
}