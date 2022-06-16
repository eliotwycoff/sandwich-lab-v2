#[macro_use] extern crate diesel;

use actix_web::{ middleware, App, HttpServer };
use actix_files::Files;
use dotenv::dotenv;
pub use state::{ AppState, init_app_state };

mod rpc;
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
            .service(Files::new("/static/css", "./static/css").show_files_listing())
            .configure(app::routes)
            .configure(api::routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}