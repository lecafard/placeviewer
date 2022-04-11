use actix_web::{get, middleware, web, App, HttpServer, HttpResponse, Responder};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use clap::Parser;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use std::fs::read_to_string;
use tokio::runtime::Runtime;

use crate::store::config;
use crate::store::config::Dataset;

const INITIAL_IMAGE_SIZE: usize = 8192;

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

type DatasetsMapArc = Arc<HashMap<String, Dataset>>;

#[get("/images/{name}/tiles/{tile_x}/{tile_y}/ts/{timestamp}.png")]
async fn get_image_by_timestamp(
  datasets: web::Data<DatasetsMapArc>,
  path: web::Path<(String, u16, u16, u64)>,
) -> impl Responder {
  let (name, tile_x, tile_y, timestamp) = path.into_inner();
  let dataset = match datasets.get(&name) {
    Some(d) => d,
    None => {
      return HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type(ContentType(mime::TEXT_PLAIN))
        .body("dataset not found");
    }
  };

  let tile = match dataset.get_tile(tile_x, tile_y) {
    Some(t) => t,
    None => {
      return HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type(ContentType(mime::TEXT_PLAIN))
        .append_header(("cache-control", "max-age=2678400"))
        .body("tile not found");
    }
  };
  

  let image = match tile.get_image_at_timestamp(timestamp) {
    Some(t) => t,
    None => {
      return HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type(ContentType(mime::TEXT_PLAIN))
        .append_header(("cache-control", "max-age=2678400"))
        .body("image not found");
    }
  };

  let mut imgdata: Vec<u8> = Vec::with_capacity(INITIAL_IMAGE_SIZE);
  write_image(tile.size, &image, &dataset.palette, &mut imgdata);
  HttpResponse::Ok()
    .content_type(ContentType(mime::IMAGE_PNG))
    .append_header(("cache-control", "max-age=2678400"))
    .body(imgdata)
}

async fn server(host: &str, port: u16, datasets: Arc<HashMap<String, Dataset>>) -> std::io::Result<()> {
  info!("Starting server on {}:{}", host, port);
  HttpServer::new(move || {
      App::new()
        .app_data(web::Data::new(datasets.clone()))
        .wrap(middleware::Logger::default())
        .wrap(middleware::DefaultHeaders::new().add(("content-type", "text/plain")))
        .service(get_image_by_timestamp)
  })
  .bind((host, port))?
  .run()
  .await
}

fn write_image<T: std::io::Write>(size: u16, data: &[u8], palette: &[u8], w: T) {
  let mut encoder = png::Encoder::new(w, size as u32, size as u32);
  encoder.set_color(png::ColorType::Indexed);
  encoder.set_palette(palette);
  let mut writer = encoder.write_header().unwrap();
  writer.write_image_data(&data).unwrap();
}