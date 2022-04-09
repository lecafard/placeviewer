#[derive(Debug)]
pub struct Placement {
  pub ts: u32,
  pub uid: u32,
  pub x: u16,
  pub y: u16,
  pub color: u8,
  pub isblk: bool,
}