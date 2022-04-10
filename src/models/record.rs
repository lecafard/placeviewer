pub trait Record {}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Placement {
  pub ts: u32,
  pub uid: u32,
  pub x: u16,
  pub y: u16,
  pub color: u8,
  pub isblk: bool,
}

#[derive(Debug)]
pub struct TileHeader {
  pub version: u16,
  pub size: u16,
  pub start_x: u16,
  pub start_y: u16,
  pub start: u64,
  pub count: u32,
}

impl Record for Placement {}
impl Record for TileHeader {}

