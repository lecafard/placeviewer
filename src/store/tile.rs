use log::{debug, info};
use memmap::{Mmap, MmapOptions};
use std::{cmp, mem, ptr, slice};
use std::fs::File;
use std::result::Result;
use std::time::Instant;
use crate::models::FrameData;
use crate::models::record::{TileKeyframeHeader, TilePlacementHeader, Placement, TILE_PLACEMENT_VERSION_ID, TILE_KEYFRAME_VERSION_ID};
use serde::Serialize;


#[derive(Debug, Serialize)]
pub struct Tile {
  pub start: u64,
  pub count: u32,
  pub uid_count: u32,
  pub start_x: u16,
  pub start_y: u16,
  pub size: u16,
  pub frame_count: u32,
  pub frame_interval: u32,

  #[serde(skip_serializing)]
  mmap_placements: Option<Mmap>,

  #[serde(skip_serializing)]
  mmap_frames: Option<Mmap>,
}


impl Tile {
  pub fn load(placement_filename: &str) -> Result<Tile, String> {
    let file = match File::open(&placement_filename) {
      Ok(f) => f,
      Err(e) => {
        return Err(e.to_string());
      }
    };
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let header: TilePlacementHeader = unsafe { ptr::read(mmap.as_ptr() as *const _) };
    info!("loading tile {:?} with header: {:?}", &placement_filename, header);
    if header.version != TILE_PLACEMENT_VERSION_ID {
      return Err(String::from("header version is wrong"));
    }
    
    Ok(Tile{
      start: header.start,
      count: header.count,
      uid_count: header.uid_count,
      start_x: header.start_x,
      start_y: header.start_y,
      size: header.size,
      frame_count: 0,
      frame_interval: 0,
      mmap_placements: Some(mmap),
      mmap_frames: None,
    })
  }

  pub fn load_with_frames(placement_filename: &str, frame_filename: &str) -> Result<Tile, String> {
    let placements_file = match File::open(&placement_filename) {
      Ok(f) => f,
      Err(e) => {
        return Err(e.to_string());
      }
    };

    let frame_file = match File::open(&frame_filename) {
      Ok(f) => f,
      Err(e) => {
        return Err(e.to_string());
      }
    };

    let mmap_placements = unsafe { MmapOptions::new().map(&placements_file).unwrap() };
    let header_placements: TilePlacementHeader = unsafe {
      ptr::read(mmap_placements.as_ptr() as *const _)
    };
    info!("loading placement {:?} with header: {:?}", &placement_filename, header_placements);
    if header_placements.version != TILE_PLACEMENT_VERSION_ID {
      return Err(String::from("header version for placement is wrong"));
    }

    let mmap_frames = unsafe { MmapOptions::new().map(&frame_file).unwrap() };
    let header_frames: TileKeyframeHeader = unsafe { ptr::read(mmap_frames.as_ptr() as *const _) };
    info!("loading frame {:?} with header: {:?}", &placement_filename, header_frames);
    if header_frames.version != TILE_KEYFRAME_VERSION_ID {
      return Err(String::from("header version for frame is wrong"));
    }
    
    if header_placements.start_x != header_frames.start_x ||
      header_placements.start_y != header_frames.start_y {
      return Err(String::from("header hismatch between placements and frames"));
    }
    
    Ok(Tile{
      start: header_placements.start,
      count: header_placements.count,
      uid_count: header_placements.uid_count,
      start_x: header_placements.start_x,
      start_y: header_placements.start_y,
      size: header_placements.size,
      frame_count: header_frames.count,
      frame_interval: header_frames.interval,
      mmap_placements: Some(mmap_placements),
      mmap_frames: Some(mmap_frames),
    })
  }

  pub fn placements<'a>(&'a self) -> &'a [Placement] {
    match &self.mmap_placements {
      Some(mmap) => {
        unsafe {
          slice::from_raw_parts(
            mmap.as_ptr()
              .offset(mem::size_of::<TilePlacementHeader>() as isize) as *const _,
            self.count as usize
          )
        }
      },
      None => &[]
    }
  }

  fn frame(&self, id: u32) -> Option<(usize, FrameData)> {
    match &self.mmap_frames {
      Some(mmap) => {
        let idx = cmp::min(self.frame_count - 1, id/self.frame_interval);
        let size = self.size as usize * self.size as usize;
        Some((idx as usize * self.frame_interval as usize, unsafe {
          slice::from_raw_parts(
            mmap.as_ptr()
              .offset(
                mem::size_of::<TileKeyframeHeader>() as isize + idx as isize * (size * 4) as isize
              ) as *const _,
            size as usize
          )
        }.to_vec()))
      },
      None => None
    }
  }

  pub fn get_image_at_timestamp(&self, timestamp: u64) -> Option<FrameData> {
    let now = Instant::now();

    let placements = self.placements();
    if (timestamp as i64 - self.start as i64) < 0 {
      return None;
    }
    
    let ts = match u32::try_from(timestamp - self.start){
      Ok(ts) => ts,
      Err(_) => placements[placements.len() - 1].ts // default to last pixel
    };

    let idx = placements.partition_point(|p| ts > p.ts);
    debug!("index for timestamp is {}/{}", idx, placements.len());
    if idx >= placements.len() {
      return None;
    }
    
    let mut start = 0;
    let mut output = match self.frame(idx as u32) {
      Some((s, x)) => {
        start = s;
        x
      },
      None => vec![1; self.size as usize * self.size as usize]
    };
    self.apply(&mut output, &placements[start..=idx]);

    debug!("Tile took {:?} to render, replayed {} placements", now.elapsed(), idx - start);

    return Some(output);
  }

  pub fn get_diff_for_timestamps(&self, timestamp1: u64, timestamp2: u64) -> Option<FrameData> {
    let img1 = self.get_image_at_timestamp(timestamp1)?;
    let img2 = self.get_image_at_timestamp(timestamp2)?;

    return Some(img1.iter().zip(img2.iter())
      .map(|(a, b)| if &a == &b { 0 } else { *b })
      .collect());
  }

  pub fn get_image_for_user(&self, user_id: u32) -> Option<FrameData> {
    if user_id >= self.uid_count {
      return None;
    }
    let mut img = vec![0u32; self.size as usize * self.size as usize];
    for p in self.placements().iter().filter(|p| p.uid == user_id) {
      img[p.x as usize + p.y as usize * self.size as usize] = (p.uid << 8) + (p.color + 1) as u32;
    }
    return Some(img);
  }


  pub fn apply(&self, img: &mut FrameData, placements: &[Placement]) {
    for p in placements.iter() {  
      img[p.x as usize + p.y as usize * self.size as usize] = (p.uid << 8) + (p.color + 1) as u32;
    }
  }
}
