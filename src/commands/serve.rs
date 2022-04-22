use actix_web::{error, get, middleware, web, App, HttpServer, HttpResponse, Responder};
use actix_web::http::header::ContentType;
use clap::Parser;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use std::fs::read_to_string;
use tokio::runtime::Runtime;

use crate::store::config;
use crate::store::config::{Dataset, Tile};

const INITIAL_IMAGE_SIZE: usize = 8192;
const CACHE_CONTROL_VALUE: &str = "max-age=2678400";

#[derive(Parser)]
pub struct ServeCommand {
  // Port to listen on
  #[clap(long, default_value_t = 3000)]
  port: u16,
  
  // Host to listen on
  #[clap(long, default_value = "localhost")]
  host: String,

  // Tile data
  #[clap(required=true)]
  config_file: String
}

type DatasetsMapArc = Arc<HashMap<String, Dataset>>;

impl ServeCommand {
  pub fn execute(&self) {
    let config_str = read_to_string(&self.config_file).unwrap();
    let config: config::Root = serde_yaml::from_str(&config_str).unwrap();
    let mut datasets: HashMap<String, Dataset> = HashMap::new();
    
    for serialized_dataset in config.datasets.iter() {
      if datasets.contains_key(&serialized_dataset.name) {
        panic!("Dataset {} already exists in map", serialized_dataset.name);
      }
      let dataset: Dataset = serialized_dataset.load();
      datasets.insert(serialized_dataset.name.clone(), dataset);
    }

    // create http server
    let rt = Runtime::new().unwrap();
    rt.block_on(server(&self.host, self.port, Arc::new(datasets)))
      .unwrap();
  }
}

async fn server(host: &str, port: u16, datasets: Arc<HashMap<String, Dataset>>) -> std::io::Result<()> {
  info!("Starting server on {}:{}", host, port);
  HttpServer::new(move || {
      App::new()
        .app_data(web::Data::new(datasets.clone()))
        .wrap(middleware::Logger::default())
        .wrap(middleware::DefaultHeaders::new()
          .add(("content-type", "text/plain"))
          .add(("x-content-type-options", "nosniff"))
        )
        .service(get_image_by_timestamp)
        .service(get_image_by_timestamp_diff)
        .service(get_image_by_user_id)
        .service(get_image_by_user_id_remainder)
  })
  .bind((host, port))?
  .run()
  .await
}

#[get("/images/{name}/tiles/{tile_x}/{tile_y}/ts/{timestamp}.png")]
async fn get_image_by_timestamp(
  datasets: web::Data<DatasetsMapArc>,
  path: web::Path<(String, u16, u16, u64)>,
) -> Result<impl Responder, error::Error> {
  let (name, tile_x, tile_y, timestamp) = path.into_inner();
  let (dataset, tile) = get_tile(&datasets, name, tile_x, tile_y).await?;

  let image: Vec<u8> = match tile.get_image_at_timestamp(timestamp) {
    Some(t) => t.iter().map(|v| (v & 0xff) as u8).collect(),
    None => return Err(error::ErrorNotFound("timestamp not found"))
  };
  let mut imgdata: Vec<u8> = Vec::with_capacity(INITIAL_IMAGE_SIZE);
  write_image(tile.size, &image, &dataset.palette, &dataset.trns_palette, &mut imgdata);
  Ok(HttpResponse::Ok()
    .content_type(ContentType(mime::IMAGE_PNG))
    .append_header(("cache-control", CACHE_CONTROL_VALUE))
    .body(imgdata))
}

#[get("/images/{name}/tiles/{tile_x}/{tile_y}/diff-ts/{timestamp1}_{timestamp2}.png")]
async fn get_image_by_timestamp_diff(
  datasets: web::Data<DatasetsMapArc>,
  path: web::Path<(String, u16, u16, u64, u64)>,
) -> Result<impl Responder, error::Error> {
  let (name, tile_x, tile_y, timestamp1, timestamp2) = path.into_inner();
  let (dataset, tile) = get_tile(&datasets, name, tile_x, tile_y).await?;

  let image: Vec<u8> = match tile.get_diff_for_timestamps(timestamp1, timestamp2) {
    Some(t) => t.iter().map(|v| (v & 0xff) as u8).collect(),
    None => return Err(error::ErrorNotFound("both timestamps not found"))
  };
  let mut imgdata: Vec<u8> = Vec::with_capacity(INITIAL_IMAGE_SIZE);
  write_image(tile.size, &image, &dataset.palette, &dataset.trns_palette, &mut imgdata);
  Ok(HttpResponse::Ok()
    .content_type(ContentType(mime::IMAGE_PNG))
    .append_header(("cache-control", CACHE_CONTROL_VALUE))
    .body(imgdata))
}

#[get("/images/{name}/tiles/{tile_x}/{tile_y}/uid-rem/{user_id}_{timestamp}.png")]
async fn get_image_by_user_id_remainder(
  datasets: web::Data<DatasetsMapArc>,
  path: web::Path<(String, u16, u16, u32, u64)>,
) -> Result<impl Responder, error::Error> {
  let (name, tile_x, tile_y, user_id, timestamp) = path.into_inner();
  let (dataset, tile) = get_tile(&datasets, name, tile_x, tile_y).await?;

  let image: Vec<u8> = match tile.get_image_at_timestamp(timestamp) {
    Some(t) => t.iter().map(|v| if (v >> 8) == user_id { v & 0xff } else { 0 } as u8).collect(),
    None => return Err(error::ErrorNotFound("timestamp not found"))
  };
  let mut imgdata: Vec<u8> = Vec::with_capacity(INITIAL_IMAGE_SIZE);
  write_image(tile.size, &image, &dataset.palette, &dataset.trns_palette, &mut imgdata);
  Ok(HttpResponse::Ok()
    .content_type(ContentType(mime::IMAGE_PNG))
    .append_header(("cache-control", CACHE_CONTROL_VALUE))
    .body(imgdata))
}

#[get("/images/{name}/tiles/{tile_x}/{tile_y}/uid/{user_id}.png")]
async fn get_image_by_user_id(
  datasets: web::Data<DatasetsMapArc>,
  path: web::Path<(String, u16, u16, u32)>,
) -> Result<impl Responder, error::Error> {
  let (name, tile_x, tile_y, user_id) = path.into_inner();
  let (dataset, tile) = get_tile(&datasets, name, tile_x, tile_y).await?;

  let image: Vec<u8> = match tile.get_image_for_user(user_id) {
    Some(t) => t.iter().map(|v| (v & 0xff) as u8).collect(),
    None => return Err(error::ErrorNotFound("user id not found"))
  };
  let mut imgdata: Vec<u8> = Vec::with_capacity(INITIAL_IMAGE_SIZE);
  write_image(tile.size, &image, &dataset.palette, &dataset.trns_palette, &mut imgdata);
  Ok(HttpResponse::Ok()
    .content_type(ContentType(mime::IMAGE_PNG))
    .append_header(("cache-control", CACHE_CONTROL_VALUE))
    .body(imgdata))
}

async fn get_tile(datasets: &DatasetsMapArc, name: String, tile_x: u16, tile_y: u16) -> Result<(&Dataset, &Tile), error::Error> {
  let dataset = match datasets.get(&name) {
    Some(d) => d,
    None => {
      return Err(error::ErrorNotFound("dataset not found"));
    }
  };

  match dataset.get_tile(tile_x, tile_y) {
    Some(t) => return Ok((dataset, t)),
    None => {
      return Err(error::ErrorNotFound("tile not found"));
    }
  };
}

fn write_image<T: std::io::Write>(size: u16, data: &[u8], palette: &[u8], trns_palette: &[u8],  w: T) {
  let mut encoder = png::Encoder::new(w, size as u32, size as u32);
  encoder.set_color(png::ColorType::Indexed);
  encoder.set_palette(palette);
  encoder.set_trns(trns_palette);
  let mut writer = encoder.write_header().unwrap();
  writer.write_image_data(&data).unwrap();
}