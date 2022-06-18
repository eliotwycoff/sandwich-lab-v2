use actix_web::{ get, web, HttpResponse, Responder };
use serde::{ Serialize, Deserialize };
use crate::state::AppState;
use crate::evm;
use super::{ db, models };

#[derive(Debug, Deserialize)]
struct PairRequest {
    blockchain: String,
    exchange: String,
    pair_address: String,
    after: Option<u64>,
    before: Option<u64>
}

#[derive(Debug, Deserialize)]
struct SandwichRequest {
    sandwich_id: u64
}

#[derive(Debug, Serialize)]
struct PairResponse {
    pair: Option<PairMetadata>,
    base: Option<TokenMetadata>,
    quote: Option<TokenMetadata>,
    error_message: String
}

impl PairResponse {
    fn as_error(msg: String) -> Self {
        Self {
            pair: None,
            base: None,
            quote: None,
            error_message: msg
        }
    }
}

#[derive(Debug, Serialize)]
struct PairMetadata {
    address: String,
    latest_scanned_block: Option<i64>,
    earliest_scanned_block: Option<i64>,
    scanning_latest: bool,
    scanning_previous: bool
}

#[derive(Debug, Serialize)]
struct TokenMetadata {
    address: String,
    name: String,
    symbol: String,
    decimals: u8
}

#[get("/pair")]
async fn fetch_pair(data: web::Data<AppState>, info: web::Query<PairRequest>) -> web::Json<PairResponse> {
    // Get the database connection, and return an error
    // if the connection cannot be established.
    let db_connection = match data.db_pool.get() {
        Ok(db_connection) => db_connection,
        Err(_) => return web::Json(
            PairResponse::as_error("cannot connect to database".to_string()))
    };

    // Standardize the incoming data.
    let blockchain_name = info.blockchain.to_lowercase();
    let pair_address = info.pair_address.to_lowercase();

    // Try to get the pair from the database.
    let db_pair_result = db::fetch_pair_by_params(
        &db_connection, 
        &blockchain_name, 
        &pair_address);

    // If the result is an error, return the error.
    let db_pair_option = match db_pair_result {
        Ok(option) => option,
        Err(_) => return web::Json(
            PairResponse::as_error("database access error".to_string()))
    };

    // Since the result isn't an error, use the database data
    // or check the blockchain for information on the pair.
    if let Some(pair) = db_pair_option {
        // Get the base token, or return an error.
        let base = match db::fetch_token_by_id(&db_connection, pair.base_token_id) {
            Ok(Some(base)) => base,
            _ => return web::Json(
                PairResponse::as_error("base does not exist".to_string()))
        };

        // Get the base token's decimal precision as a u8.
        let base_decimals = match u8::try_from(base.decimals) {
            Ok(d) => d,
            Err(_) => return web::Json(
                PairResponse::as_error("base decimal error".to_string()))
        };

        // Get the quote token, or return an error.
        let quote = match db::fetch_token_by_id(&db_connection, pair.quote_token_id) {
            Ok(Some(quote)) => quote,
            _ => return web::Json(
                PairResponse::as_error("quote does not exist".to_string()))
        };

        // Get the quote token's decimal precision as a u8.
        let quote_decimals = match u8::try_from(quote.decimals) {
            Ok(d) => d,
            Err(_) => return web::Json(
                PairResponse::as_error("base decimal error".to_string()))
        };

        // Construct and return the PairResponse JSON object.
        return web::Json(PairResponse {
            pair: Some(PairMetadata {
                address: pair.pair_address,
                latest_scanned_block: pair.latest_scanned_block,
                earliest_scanned_block: pair.earliest_scanned_block,
                scanning_latest: pair.scanning_latest,
                scanning_previous: pair.scanning_previous
            }),
            base: Some(TokenMetadata {
                address: base.token_address,
                name: base.token_name,
                symbol: base.token_symbol,
                decimals: base_decimals
            }),
            quote: Some(TokenMetadata {
                address: quote.token_address,
                name: quote.token_name,
                symbol: quote.token_symbol,
                decimals: quote_decimals
            }),
            error_message: "".to_string()
        });
    } else {
        // The pair is not in the database, so to get its metadata,
        // we need to make an RPC call to the DataAggregator contract.
        let exchange_name = info.exchange.to_lowercase();

        // First get the blockchain state data, or return an error.
        let blockchain = match data.blockchains.get(&blockchain_name) {
            Some(blockchain) => blockchain,
            None => return web::Json(
                PairResponse::as_error("blockchain not supported".to_string()))
        };

        // Get the exchange state data, or return an error.
        let exchange = match blockchain.exchanges.get(&exchange_name) {
            Some(exchange) => exchange,
            None => return web::Json(
                PairResponse::as_error("exchange not supported".to_string()))
        };

        // Try to get the pair metadata from the blockchain.
        let metadata = match evm::fetch_pair_metadata(
            &blockchain.provider_url,
            &pair_address,
            &blockchain.data_aggregator_address).await {
            
            Ok(metadata) => metadata,
            Err(_) => return web::Json(
                PairResponse::as_error("provider error".to_string()))
        };

        // Standardize the token addresses.
        let base_address = format!("0x{}", metadata.base_address.to_lowercase());
        let quote_address = format!("0x{}", metadata.quote_address.to_lowercase());

        // Commit the pair's base token to the database, if necessary.
        let base_id = match db::fetch_token_by_params(
            &db_connection,
            &blockchain.name,
            &base_address) {

            Ok(Some(base_token)) => base_token.token_id,
            Ok(None) => {
                match db::insert_token(
                    &db_connection,
                    &metadata.base_name,
                    &metadata.base_symbol,
                    metadata.base_decimals as i16,
                    &blockchain.name,
                    &base_address) {

                    Ok(id) => id,
                    Err(_) => return web::Json(
                        PairResponse::as_error("base insert error".to_string()))
                }
            },
            Err(_) => return web::Json(
                PairResponse::as_error("base fetch error".to_string()))
        };

        // Commit the pair's quote token to the database, if necessary.
        let quote_id = match db::fetch_token_by_params(
            &db_connection,
            &blockchain.name,
            &quote_address) {
            
            Ok(Some(quote_token)) => quote_token.token_id,
            Ok(None) => {
                match db::insert_token(
                    &db_connection,
                    &metadata.quote_name,
                    &metadata.quote_symbol,
                    metadata.quote_decimals as i16,
                    &blockchain.name,
                    &quote_address) {
                    
                    Ok(id) => id,
                    Err(_) => return web::Json(
                        PairResponse::as_error("quote insert error".to_string()))
                }
            },
            Err(_) => return web::Json(
                PairResponse::as_error("quote fetch error".to_string()))
        };

        // Commit the pair to the database, or return an error.
        match db::insert_pair(
            &db_connection,
            &blockchain.name,
            exchange.name(),
            &pair_address,
            base_id,
            quote_id) {
            
            Err(_) => return web::Json(
                PairResponse::as_error("pair insert error".to_string())),
            _ => {}
        };

        // Construct and return the PairResponse JSON object.
        return web::Json(PairResponse {
            pair: Some(PairMetadata {
                address: pair_address,
                latest_scanned_block: None,
                earliest_scanned_block: None,
                scanning_latest: false,
                scanning_previous: false
            }),
            base: Some(TokenMetadata {
                address: metadata.base_address,
                name: metadata.base_name,
                symbol: metadata.base_symbol,
                decimals: metadata.base_decimals
            }),
            quote: Some(TokenMetadata {
                address: metadata.quote_address,
                name: metadata.quote_name,
                symbol: metadata.quote_symbol,
                decimals: metadata.quote_decimals
            }),
            error_message: "".to_string()
        });
    }
}

#[get("/sandwich")]
async fn fetch_sandwich(info: web::Query<SandwichRequest>) -> impl Responder {
    // under construction
    HttpResponse::Ok()
}

#[get("/sandwiches")]
async fn fetch_sandwiches(info: web::Query<PairRequest>) -> impl Responder {
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