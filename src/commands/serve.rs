use actix_web::{get, post, App, HttpServer, HttpResponse, Responder};
use clap::Parser;
use log::{info};
use std::collections::HashMap;
use std::fs::{File, read_to_string};
use std::io::BufWriter;
use std::time::Instant;
use tokio::runtime::Runtime;

use crate::store::config;
use crate::store::config::{Dataset, Tile};

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
    rt.block_on(server(&self.host, self.port))
      .unwrap();
  }
}

#[get("/")]
async fn hello() -> impl Responder {
  HttpResponse::Ok().body("Hello world!")
}
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}


async fn server(host: &str, port: u16) -> std::io::Result<()> {
  info!("Starting server on {}:{}", host, port);
  HttpServer::new(|| {
      App::new()
          .service(hello)
          .service(echo)
  })
  .bind((host, port))?
  .run()
  .await
}

fn generate_images(dataset: &Dataset) {
  for (i, tile) in dataset.tiles.iter().enumerate() {

    let now = Instant::now();
    let mut data: Vec<u8> = vec![0; tile.size as usize * tile.size as usize];
    for p in tile.placements().iter() {
      let i = p.x as usize + (p.y as usize * dataset.size_tile as usize);
      data[i] = p.color;
    }

    println!("Frame took {:?} seconds to render", now.elapsed());

    let file = File::create(format!("data/pics/{}_{}.png", dataset.name, i)).unwrap();
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, dataset.size_tile as u32, dataset.size_tile as u32);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_palette(&dataset.palette);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&data).unwrap();
  }
}