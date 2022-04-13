use std::marker::PhantomData;

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

