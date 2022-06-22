#[macro_use] 
pub mod utils;
mod pair;
mod sandwiches;

// Package up all the api routes into a ServiceConfig
// that can be registered on startup in main.rs.
pub fn routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/api")
        .service(pair::fetch_pair)
        .service(sandwiches::fetch_sandwiches));
}