use actix_web::{get, web, HttpResponse,Result, HttpRequest, post, delete};
use crate::db::db::MongoDbClient;
use crate::dto::car_dto::CarDto;
use serde::{Serialize};

#[derive(Serialize)]
pub struct Response {
    pub message: String,
}


#[get("/cars")]
pub async fn get_cars(data: web::Data<MongoDbClient>) -> Result<HttpResponse> {
    let cars = data.get_cars().await.unwrap();
    Ok(HttpResponse::Ok().json(cars.iter().map(CarDto::from).collect::<Vec<_>>()))
}

#[get("/cars/{id}")]
pub async fn get_car(req: HttpRequest, data: web::Data<MongoDbClient>) -> Result<HttpResponse> {
    let car_id = req.match_info().get("id").unwrap();
    let car = data.get_car(car_id).await;
    match car {
        Ok(car) => Ok(HttpResponse::Ok().json(CarDto::from(&car))),
        Err(_) => Ok(HttpResponse::NotFound().json(Response {
            message: "Resource not found".to_string(),
        }))
    }
}

#[post("/cars")]
pub async fn create_car(car_dto: web::Json<CarDto>, data: web::Data<MongoDbClient>) -> Result<HttpResponse> {
    let result = data.create_car(car_dto.into_inner().into()).await;
    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(CarDto::from(&result.unwrap()))),
        Err(_) => Ok(HttpResponse::InternalServerError().json(Response {
            message: "Error creating car".to_string(),
        }))
    }
}

#[delete("/cars/{id}")]
pub async fn delete_car(req: HttpRequest, data: web::Data<MongoDbClient>) -> Result<HttpResponse> {
    let car_id = req.match_info().get("id").unwrap();
    let result = data.delete_car(car_id).await;
    match result {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().json(Response {
            message: "Error deleting car".to_string(),
        }))
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(
      web::scope("/api")
          .service(get_cars)
          .service(create_car)
          .service(get_car)
          .service(delete_car)
  );
}