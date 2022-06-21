use actix_web::{ get, web, HttpRequest, HttpResponse, Responder };
use serde::Deserialize;
use crate::state::AppState;
use crate::templates;

// Serve the home page.
#[get("/")]
async fn index(
    data: web::Data<AppState>,
    req: HttpRequest
) -> impl Responder {
    let app_name = data.app_name.lock().unwrap();
    let inspect_url = req.url_for_static("inspect_pair").unwrap().to_string();

    let mut blockchains = Vec::new();

    for (str_id, blockchain) in data.blockchains.iter() {
        blockchains.push(templates::index::Blockchain {
            name: &blockchain.name,
            str_id: str_id
        });
    }

    blockchains.sort(); // sort the blockchains alphabetically by name

    HttpResponse::Ok().body(templates::index::render(&app_name, &inspect_url, blockchains))
}

// Define request params for inspect_pair, below.
#[derive(Debug, Deserialize)]
struct ScanRequest {
    blockchain: String,
    pair: String
}

// Serve a page for a given pair, if it exists.
#[get("/inspect")]
async fn inspect_pair(
    data: web::Data<AppState>,
    req: HttpRequest,
    info: web::Query<ScanRequest>
) -> impl Responder {
    let app_name = data.app_name.lock().unwrap();
    let home_url = req.url_for_static("index").unwrap().to_string();

    let blockchain_str_id = info.blockchain.to_lowercase();
    let blockchain = match data.blockchains.get(&blockchain_str_id) {
        Some(blockchain) => blockchain,
        None => {
            let message = "Did you specify a valid blockchain?".to_string();
            return render_not_found(&message, &home_url); 
        }
    };

    let pair_address = info.pair.to_lowercase();
    let api_url = req.url_for_static("fetch_pair").unwrap().to_string();

    HttpResponse::Ok().body(templates::pair::render(
        &app_name,
        &blockchain.name,
        &blockchain_str_id,
        &pair_address,
        &api_url,
        &home_url))
} 

// Serve a 404 for non-existant resources.
async fn not_found(
    req: HttpRequest
) -> impl Responder {
    let home_url = req.url_for_static("index").unwrap().to_string();
    let message = "This is not the content you're looking for.".to_string();
    render_not_found(&message, &home_url)
}

// helper function so that 404s can be rendered in other routes
fn render_not_found(
    message: &str, 
    home_url: &str
) -> HttpResponse {
    HttpResponse::NotFound().body(templates::not_found::render(
        "Sandwich Lab", message, home_url))
}

// Package up all the app routes into a ServiceConfig
// that can be registered on startup in main.rs.
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
        .service(index)
        .service(inspect_pair)
        .default_service(web::route().to(not_found)));
}