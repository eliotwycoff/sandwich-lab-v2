use actix_web::{ get, web, HttpRequest, HttpResponse, Responder };
use crate::state::AppState;
use crate::templates;

// Serve the home page.
#[get("/")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let app_name = data.app_name.lock().unwrap();
    let mut exchanges = Vec::new();

    // temporary code -- requires changing
    for blockchain in data.blockchains.values() {
        for exchange in blockchain.exchanges.values() {
            exchanges.push(templates::index::Exchange {
                name: exchange.name(),
                blockchain: &blockchain.name
            });
        }
    }

    HttpResponse::Ok().body(templates::index::render(&app_name, exchanges))
}

// Serve a page for a given pair, if it exists.
#[get("/{blockchain_name}/{exchange_name}/{pair_address}")]
async fn pair_profile(
    data: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(String, String, String)>) -> impl Responder {

    let app_name = data.app_name.lock().unwrap();
    let home_url = req.url_for_static("index").unwrap().to_string();
    let (blockchain_name, exchange_name, pair_address) = path.into_inner();

    let blockchain_id_str = blockchain_name.to_lowercase();
    let blockchain = match data.blockchains.get(&blockchain_id_str) {
        Some(blockchain) => blockchain,
        None => {
            let message = "Did you specify a valid blockchain?".to_string();
            return render_not_found(&message, &home_url); 
        }
    };

    let exchange_id_str = exchange_name.to_lowercase();
    let exchange = match blockchain.exchanges.get(&exchange_id_str) {
        Some(exchange) => exchange,
        None => {
            let message = "Did you specify a valid exchange?".to_string();
            return render_not_found(&message, &home_url);
        }
    };

    let pair_address = pair_address.to_lowercase();
    let api_url = req.url_for_static("fetch_pair").unwrap().to_string();

    HttpResponse::Ok().body(templates::pair::render(
        &app_name,
        &blockchain.name,
        &blockchain_id_str,
        exchange.name(),
        &exchange_id_str,
        &pair_address,
        &api_url))
} 

// Serve a 404 for non-existant resources.
async fn not_found(req: HttpRequest) -> impl Responder {
    let home_url = req.url_for_static("index").unwrap().to_string();
    let message = "This is not the content you're looking for.".to_string();
    render_not_found(&message, &home_url)
}

// helper function so that 404s can be rendered in other routes
fn render_not_found(message: &str, home_url: &str) -> HttpResponse {
    HttpResponse::NotFound().body(templates::not_found::render(
        "Sandwich Lab", message, home_url))
}

// Package up all the app routes into a ServiceConfig
// that can be registered on startup in main.rs.
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
        .service(index)
        .service(pair_profile)
        .default_service(web::route().to(not_found)));
}