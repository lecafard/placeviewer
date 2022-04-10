use crate::models::Placement;

pub struct Tile {
  pub start: u64,
  pub count: u32,
  pub start_x: u16,
  pub start_y: u16,
  pub size: u16,
  pub placements: Vec<Placement>,
}

impl Tile {
  pub fn finalize(&mut self) {
    self.count = self.placements.len() as u32;
  }
}