mod models;
mod db;
mod dto;
mod config;
mod controllers;
mod utils;
mod seeder;

use crate::config::config::Config;
use crate::db::db::MongoDbClient;
use crate::seeder::car_seeder as seed;
use serde::{Serialize};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use actix_web::web::Data;

#[derive(Serialize)]
pub struct Response {
    pub message: String,
}


#[get("/health")]
async fn healthcheck() -> impl Responder {
    let response = Response {
        message: "Everything is working fine".to_string(),
    };
    HttpResponse::Ok().json(response)
}

#[get("/")]
async fn index() -> impl Responder {
    let response = Response {
        message: "Backend Up and Running".to_string(),
    };
    HttpResponse::Ok().json(response)
}


async fn not_found() -> Result<HttpResponse> {
    let response = Response {
        message: "Resource not found".to_string(),
    };
    Ok(HttpResponse::NotFound().json(response))
}

async fn setup(config: Config) -> Data<MongoDbClient> {
    let cars_db = MongoDbClient::new(config).await;
    let app_data = web::Data::new(cars_db);
    app_data
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::new();
    let data = setup(config).await;
    
    seed::seed().await.unwrap();
    HttpServer::new(move ||
        App::new()
            .app_data(data.clone())
            .configure(controllers::car_controller::config)
            .service(healthcheck)
            .default_service(web::route().to(not_found))
            .wrap(actix_web::middleware::Logger::default())
            
    )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}


#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::http::StatusCode;
  use actix_web::test;
  use actix_web::test::TestRequest;
  use testcontainers::{clients};
  use testcontainers::core::{ WaitFor};
  use testcontainers::images::generic::GenericImage;
  use crate::models::car_model::{Car,CarType};
  use crate::dto::car_dto::CarDto;
  use crate::controllers::car_controller::{get_cars, create_car, get_car, delete_car};

    #[actix_web::test]
    async fn test_index() {
        let app = test::init_service(App::new().service(index)).await;
        let req = TestRequest::default().to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(StatusCode::OK, resp.status());
    }

    #[actix_web::test]
    async fn test_healthcheck() {
        let app = test::init_service(App::new().service(healthcheck)).await;
        let req = TestRequest::default().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(StatusCode::OK, resp.status());
    }

    #[actix_web::test]
    async fn test_get_cars() {
        let docker = clients::Cli::default();
        let msg = WaitFor::message_on_stdout("server is ready");
        let generic = GenericImage::new("mongo", "6.0.7").with_wait_for(msg.clone())
            .with_env_var("MONGO_INITDB_DATABASE", "cars_info")
            .with_env_var("MONGO_INITDB_ROOT_USERNAME", "root")
            .with_env_var("MONGO_INITDB_ROOT_PASSWORD", "root")
            .with_exposed_port(27017);

        let node = docker.run(generic);
        let port = node.get_host_port_ipv4(27017);
        println!("Port: {}", port);

        let mut config = Config::new();
        config.mongodb_uri = format!("mongodb://root:root@localhost:{}", port);
        let data = setup(config).await;
        let app = test::init_service(App::new().app_data(data.clone()).service(get_cars).service(create_car)).await;
        let req = TestRequest::default().uri("/cars").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(StatusCode::OK, resp.status());
        let result: Vec<Car> = test::read_body_json(resp).await;
        assert_eq!(result.len(), 0);

        let post = create_one_test_car();
        let resp = test::call_service(&app, post.to_request()).await;
        assert_eq!(StatusCode::OK, resp.status());
        let result: Car = test::read_body_json(resp).await; 
        let mut config = Config::new();
        config.mongodb_uri = format!("mongodb://root:root@localhost:{}", port);
        assert_eq!(result.name, "Test");

        let req = TestRequest::default().uri("/cars").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(StatusCode::OK, resp.status());
        let result: Vec<Car> = test::read_body_json(resp).await;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Test");
    }

    #[actix_web::test]
    async fn test_get_car() {
        let docker = clients::Cli::default();
        let msg = WaitFor::message_on_stdout("server is ready");
        let generic = GenericImage::new("mongo", "6.0.7").with_wait_for(msg.clone())
            .with_env_var("MONGO_INITDB_DATABASE", "cars_info")
            .with_env_var("MONGO_INITDB_ROOT_USERNAME", "root")
            .with_env_var("MONGO_INITDB_ROOT_PASSWORD", "root");

        let node = docker.run(generic);
        let port = node.get_host_port_ipv4(27017);

        let mut config = Config::new();
        config.mongodb_uri = format!("mongodb://root:root@localhost:{}", port);
        let data = setup(config).await;
        let app = test::init_service(App::new().app_data(data.clone()).service(get_cars).service(create_car).service(get_car)).await;



        let create_car_req = create_one_test_car();
        let resp = test::call_service(&app, create_car_req.to_request()).await;
        assert_eq!(StatusCode::OK, resp.status());
        let new_car: CarDto = test::read_body_json(resp).await;
        assert_eq!(new_car.name, "Test");

        let get_car_req = TestRequest::get().uri(format!("/cars/{}", new_car.id.unwrap()).as_str()).to_request();
        let resp = test::call_service(&app, get_car_req).await;
        assert_eq!(StatusCode::OK, resp.status());
        let result: CarDto = test::read_body_json(resp).await;
        assert_eq!(result.name, new_car.name);
    }

    #[actix_web::test]
    async fn test_delete_car() {
        let docker = clients::Cli::default();
        let msg = WaitFor::message_on_stdout("server is ready");
        let generic = GenericImage::new("mongo", "6.0.7").with_wait_for(msg.clone())
            .with_env_var("MONGO_INITDB_DATABASE", "cars_info")
            .with_env_var("MONGO_INITDB_ROOT_USERNAME", "root")
            .with_env_var("MONGO_INITDB_ROOT_PASSWORD", "root");

        let node = docker.run(generic);
        let port = node.get_host_port_ipv4(27017);

        let mut config = Config::new();
        config.mongodb_uri = format!("mongodb://root:root@localhost:{}", port);
        let data = setup(config).await;
        let app = test::init_service(App::new().app_data(data.clone())
            .service(get_cars).service(create_car).service(get_car)
            .service(delete_car)).await;

        let create_car_req = create_one_test_car();
        let resp = test::call_service(&app, create_car_req.to_request()).await;
        assert_eq!(StatusCode::OK, resp.status());
        let new_car: CarDto = test::read_body_json(resp).await;
        assert_eq!(new_car.name, "Test");

        let new_car_id = new_car.id.unwrap();
        let get_car_req = TestRequest::get().uri(format!("/cars/{}", new_car_id).as_str()).to_request();
        let resp = test::call_service(&app, get_car_req).await;
        assert_eq!(StatusCode::OK, resp.status());
        let result: CarDto = test::read_body_json(resp).await;
        assert_eq!(result.name, new_car.name);

        let delete_car_req = TestRequest::delete().uri(format!("/cars/{}", new_car_id).as_str()).to_request();
        let resp = test::call_service(&app, delete_car_req).await;
        assert_eq!(StatusCode::OK, resp.status());

        let get_car_req = TestRequest::get().uri(format!("/cars/{}", new_car_id).as_str()).to_request();
        let resp = test::call_service(&app, get_car_req).await;
        assert_eq!(StatusCode::NOT_FOUND, resp.status());
    }

    fn create_one_test_car() -> TestRequest {
        let post = TestRequest::post().uri("/cars").set_json(&CarDto {
            id: None,
            name: "Test".to_string(),
            brand: "Test".to_string(),
            year: 2021,
            r#type: CarType::Other,
        });
        post
    }
}