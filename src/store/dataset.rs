use serde::Serialize;

use super::tile::Tile;

#[derive(Debug, Serialize)]
pub struct Dataset {
  pub name: String,
  pub palette: Vec<u8>,
  pub trns_palette: Vec<u8>,
  pub size_x: u16,
  pub size_y: u16,
  pub size_tile: u16,
  pub tiles: Vec<Tile>,
}

impl Dataset {
  pub fn get_tile(&self, x: u16, y: u16) -> Option<&Tile> {
    let sx = self.size_x / self.size_tile;
    let sy = self.size_y / self.size_tile;
    if x >= sx || y >= sy { 
      return None
    }
    return Some(&self.tiles[x as usize + y as usize * sx as usize]);
  }
}
