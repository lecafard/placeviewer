use glob::glob;
use log::warn;
use serde::Deserialize;
use std::iter;

use super::dataset::Dataset;
use super::tile::Tile;

#[derive(Debug, Deserialize)]
pub struct ConfigRoot {
  pub datasets: Vec<SerializedDataset>,
}

#[derive(Debug, Deserialize)]
pub struct SerializedDataset {
  pub name: String,
  pub prefix: String,
  pub palette: Vec<u32>,
  pub size_x: u16,
  pub size_y: u16,
  pub size_tile: u16,
}


impl SerializedDataset {
  pub fn load(&self) -> Dataset {
    let palette: Vec<u8> = iter::once(0xffffff).chain(self.palette.clone().into_iter())
      .flat_map(|v| {
        [
          (v >> 16 & 0xff) as u8,
          (v >>  8 & 0xff) as u8,
          (v       & 0xff) as u8
        ]
      })
      .collect();
    let trns_palette: Vec<u8> = iter::once(0)
      .chain(iter::repeat(255).take(self.palette.len()))
      .collect();

    let tiles_x = (self.size_x / self.size_tile) as usize;
    let tiles_y = (self.size_y / self.size_tile) as usize;

    let mut dataset = Dataset {
      name: self.name.clone(),
      palette: palette,
      trns_palette: trns_palette,
      size_x: self.size_x,
      size_y: self.size_y,
      size_tile: self.size_tile,
      tiles: Vec::with_capacity(tiles_x * tiles_y),
    };

    let mut placement_files: Vec<String> = Vec::new();
    let mut frame_files: Vec<String> = Vec::new();

    for entry in glob(&format!("{}_log_*_*.bin", self.prefix)).expect("Failed to read glob pattern") {
      let filename = match entry {
        Ok(s) => s,
        Err(e) => {
          warn!("Error with glob {}", e);
          continue
        }
      };
      placement_files.push(String::from(filename.to_str().unwrap()));
    }

    for entry in glob(&format!("{}_frame_*_*.bin", self.prefix)).expect("Failed to read glob pattern") {
      let filename = match entry {
        Ok(s) => s,
        Err(e) => {
          warn!("Error with glob {}", e);
          continue
        }
      };
      frame_files.push(String::from(filename.to_str().unwrap()));
    }

    if frame_files.len() != placement_files.len() {
      // TODO: use a result for this
      panic!("placement tiles don't correspond with frame tiles");
    }

    for (pf, ff) in placement_files.iter().zip(frame_files.iter()) {
      dataset.tiles.push(match Tile::load_with_frames(&pf, &ff) {
        Ok(t) => t,
        Err(e) => panic!("{}", e)
      });
    }
    
    dataset.tiles.sort_by_key(|t| t.start_x);
    dataset.tiles.sort_by_key(|t| t.start_y);
    return dataset;
  }
}
