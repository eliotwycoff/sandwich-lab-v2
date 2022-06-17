use actix_web::{ get, web, HttpRequest, HttpResponse, Responder };
use crate::state::AppState;
use crate::templates;

// Serve the home page.
#[get("/")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let app_name = data.app_name.lock().unwrap();
    let mut exchanges = Vec::new();

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

    match data.blockchains.get(&blockchain_name.to_lowercase()) {
        Some(blockchain) => {
            match blockchain.exchanges.get(&exchange_name.to_lowercase()) {
                Some(exchange) => {
                    // temporary code -- should put this in the api and fetch dynamically
                    match crate::evm::fetch_pair_metadata(
                        &blockchain.provider_url,
                        &pair_address,
                        &blockchain.data_aggregator_address).await {

                        Ok(metadata) => {
                            HttpResponse::Ok().body(templates::pair::render(
                                &app_name,
                                &blockchain.name,
                                exchange.name(),
                                &pair_address,
                                &metadata.base_name,
                                &metadata.base_symbol,
                                &metadata.base_address,
                                metadata.base_decimals,
                                &metadata.quote_name,
                                &metadata.quote_symbol,
                                &metadata.quote_address,
                                metadata.quote_decimals)) 
                        },
                        Err(e) => {
                            println!("{e:?}");
                            render_not_found(&home_url)
                        }
                    }
                },
                None => render_not_found(&home_url)
            }
        },
        None => render_not_found(&home_url)
    }
} 

// Serve a 404 for non-existant resources.
async fn not_found(req: HttpRequest) -> impl Responder {
    let home_url = req.url_for_static("index").unwrap().to_string();
    render_not_found(&home_url)
}

// helper function so that 404s can be rendered in other routes
fn render_not_found(home_url: &str) -> HttpResponse {
    HttpResponse::NotFound().body(templates::not_found::render("Sandwich Lab", &home_url))
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