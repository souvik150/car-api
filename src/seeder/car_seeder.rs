use crate::config::config::Config;
use crate::utils::read_json::read_json_file;
use crate::db::db::MongoDbClient;
use crate::models::car_model as models;
use actix_web::web::Data;
use serde::{ Deserialize};

#[derive(Debug, Deserialize)]
struct Car {
    brand: String,
    name: String,
    year: i32,
    r#type: String,
}

async fn setup(config: Config) -> Data<MongoDbClient> {
  let cars_db = MongoDbClient::new(config).await;
  let app_data = Data::new(cars_db);
  app_data
}

pub async fn seed() -> Result<(), Box<dyn std::error::Error>> {

  let config = Config::new();

  if config.env == "prod" {
      println!("Skipping seeding in prod environment.");
      return Ok(());
  }
  
  let data = setup(config).await;
  let existing_cars = data.get_cars().await?;

  if !existing_cars.is_empty() {
      println!("Data already exists in the database. Skipping seeding.");
      return Ok(());
  }

  let file_path = "./../seed/cars.json";
  let json_string = read_json_file(file_path)?;
  let cars: Vec<Car> = serde_json::from_str(&json_string)?;

  for car in cars {
      let car_type = match car.r#type.as_str() {
          "Sedan" => models::CarType::Sedan,
          "Hatchback" => models::CarType::Hatchback,
          _ => models::CarType::Other,
      };

      let car = models::Car {
          id: None,
          name: car.name,
          brand: car.brand,
          year: car.year,
          r#type: car_type,
      };

      data.create_car(car).await?;
  }

  println!("Seeding completed.");
  Ok(())
}