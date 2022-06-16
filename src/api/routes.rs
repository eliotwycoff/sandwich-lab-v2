use actix_web::{ get, web, HttpResponse, Responder };
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PairParams {
    blockchain: String,
    exchange: String,
    pair_address: String,
    after: Option<u64>,
    before: Option<u64>
}

#[derive(Debug, Deserialize)]
struct SandwichParams {
    sandwich_id: u64
}

#[get("/pair")]
async fn fetch_pair(info: web::Query<PairParams>) -> impl Responder {
    // under construction
    HttpResponse::Ok()
}

#[get("/sandwich")]
async fn fetch_sandwich(info: web::Query<SandwichParams>) -> impl Responder {
    // under construction
    HttpResponse::Ok()
}

#[get("/sandwiches")]
async fn fetch_sandwiches(info: web::Query<PairParams>) -> impl Responder {
    // under construction
    HttpResponse::Ok()
}

// Package up all the api routes into a ServiceConfig
// that can be registered on startup in main.rs.
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
        .service(fetch_pair)
        .service(fetch_sandwich)
        .service(fetch_sandwiches));
}