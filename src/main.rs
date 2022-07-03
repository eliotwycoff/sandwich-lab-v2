#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

use actix_web::{ middleware, App, HttpServer };
use actix_files::Files;
use dotenv::dotenv;
use std::env;
pub use state::{ AppState, init_app_state };

mod api;
mod app;
mod state;
mod templates;

// Start the main actix-web runtime.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Pull in all the necessary environment variables.
    dotenv().ok();

    // Initialize the global AppState instance.
    let app_state = init_app_state();

    // Register routes and start running the server.
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::NormalizePath::trim())
            .service(Files::new("/static", "./static").show_files_listing())
            .configure(api::routes)
            .configure(app::routes)
    })
    .bind(env::var("SOCKET").expect("SOCKET environment variable not found"))?
    .run()
    .await
}