use actix_web::{ get, web, HttpResponse, Responder };
use serde::{ Serialize, Deserialize };
use diesel::NotFound;
use crate::state::AppState;
use crate::evm;
use super::{ db };

// This helper macro creates a JSON response formatted as an error.
#[macro_export]
macro_rules! response_error {
    ($message:expr, $response:ident) => {
        web::Json($response::as_error($message.to_string()))
    }
}

// This helper macro gets a database connection from the database pool,
// returning the appropriate error if a connection cannot be established.
#[macro_export]
macro_rules! get_db_connection {
    ($data:expr, $response:ident) => {
        match $data.db_pool.get() {
            Ok(db_connection) => db_connection,
            Err(_) => return response_error!("cannot connect to database", $response)
        }
    };
}

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
    // Get a database connection, and return an error
    // if a connection cannot be established.
    let db_connection = get_db_connection!(data, PairResponse);

    // Standardize the incoming data.
    let blockchain_id = info.blockchain.to_lowercase();
    let exchange_id = info.exchange.to_lowercase();
    let pair_address = info.pair_address.to_lowercase();

    // First get the blockchain state data, or return an error.
    let blockchain = match data.blockchains.get(&blockchain_id) {
        Some(blockchain) => blockchain,
        None => return response_error!("blockchain not supported", PairResponse)
    };

    // Get the exchange state data, or return an error.
    let exchange = match blockchain.exchanges.get(&exchange_id) {
        Some(exchange) => exchange,
        None => return response_error!("exchange not supported", PairResponse)
    };

    // Spawn a new, non-blocking thread to fetch
    // the pair, base and quote from the database.
    let metadata_thread_result = web::block(move || {
        let pair = match db::fetch_pair_by_params(
            &db_connection, &blockchain_id, &pair_address) {
            Ok(pair) => pair,
            Err(e) => return Err(e)
        };

        let base = match db::fetch_token_by_id(
            &db_connection, pair.base_token_id) {
            Ok(token) => token,
            Err(e) => return Err(e)
        };
        
        let quote = match db::fetch_token_by_id(
            &db_connection, pair.quote_token_id) {
            Ok(token) => token,
            Err(e) => return Err(e)
        };

        Ok((pair, base, quote))
    }).await;

    // Unpack the thread result.
    let metadata_db_result = match metadata_thread_result {
        Ok(result) => result,
        Err(_) => return response_error!("thread error", PairResponse)
    };

    // Unpack the database result.
    if let Ok((pair, base, quote)) = metadata_db_result {
        // The pair, base and quote were successfully fetched,
        // so process the data and return it to the user as JSON.

        // Get the base token's decimal precision as a u8.
        let base_decimals = match u8::try_from(base.decimals) {
            Ok(d) => d,
            _ => return response_error!("base decimal error", PairResponse)
        };

        // Get the quote token's decimal precision as a u8.
        let quote_decimals = match u8::try_from(quote.decimals) {
            Ok(d) => d,
            _ => return response_error!("quote decimal error", PairResponse)
        };

        // Construct and return the PairResponse JSON object.
        web::Json(PairResponse {
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
            error_message: "".to_string()})
    } else {
        // There was an error fetching the pair, base and quote,
        // or at least one could not be found in the database,
        // so try to get this information from the blockchain.
        let pair_address = info.pair_address.to_lowercase();
        let blockchain_name = blockchain.name.clone();
        let exchange_name = exchange.name().to_string();

        // Asynchronously get the pair metadata from the blockchain.
        let metadata = match evm::fetch_pair_metadata(
            &blockchain.provider_url,
            &pair_address,
            &blockchain.data_aggregator_address).await {
            
            Ok(metadata) => metadata,
            _ => return response_error!("provider error", PairResponse)
        };

        // Standardize the token addresses.
        let base_address = format!("0x{}", metadata.base_address.to_lowercase());
        let quote_address = format!("0x{}", metadata.quote_address.to_lowercase());

        // Clone the metadata for use inside the following closure.
        let metadata_clone = metadata.clone();

        // Create the PairResponse object.
        let pair_response = web::Json(PairResponse {
            pair: Some(PairMetadata {
                address: pair_address.clone(),
                latest_scanned_block: None,
                earliest_scanned_block: None,
                scanning_latest: false,
                scanning_previous: false
            }),
            base: Some(TokenMetadata {
                address: base_address.clone(),
                name: metadata.base_name,
                symbol: metadata.base_symbol,
                decimals: metadata.base_decimals
            }),
            quote: Some(TokenMetadata {
                address: quote_address.clone(),
                name: metadata.quote_name,
                symbol: metadata.quote_symbol,
                decimals: metadata.quote_decimals
            }),
            error_message: "".to_string()
        });

        // TODO
        // The following code commits the new pair and token data
        // to the database before returning the pair_response, but
        // we can do the database operations in a scheduled job,
        // allowing us to return the pair_response early.
        // ****
        
        // Get a new database connection, and return an error
        // if a connection cannot be established.
        let db_connection = get_db_connection!(data, PairResponse);

        // Spawn a new, non-blocking thread to save
        // the pair, base and quote to the database.
        let insert_thread_result = web::block(move || {
            // Commit the pair's base token to the database, if necessary.
            let base_id = match db::fetch_token_by_params(
                &db_connection,
                &blockchain_name,
                &base_address) {

                Ok(base_token) => base_token.token_id,
                Err(NotFound) => {
                    match db::insert_token(
                        &db_connection,
                        &metadata_clone.base_name,
                        &metadata_clone.base_symbol,
                        metadata_clone.base_decimals as i16,
                        &blockchain_name,
                        &base_address) {

                        Ok(id) => id,
                        Err(e) => return Err(e)
                    }
                },
                Err(e) => return Err(e)
            };

            // Commit the pair's quote token to the database, if necessary.
            let quote_id = match db::fetch_token_by_params(
                &db_connection,
                &blockchain_name,
                &quote_address) {
                
                Ok(quote_token) => quote_token.token_id,
                Err(NotFound) => {
                    match db::insert_token(
                        &db_connection,
                        &metadata_clone.quote_name,
                        &metadata_clone.quote_symbol,
                        metadata_clone.quote_decimals as i16,
                        &blockchain_name,
                        &quote_address) {
                        
                        Ok(id) => id,
                        Err(e) => return Err(e)
                    }
                },
                Err(e) => return Err(e)
            };

            // Commit the pair to the database, or return an error.
            match db::insert_pair(
                &db_connection,
                &blockchain_name,
                &exchange_name,
                &pair_address,
                base_id,
                quote_id) {
                
                Err(e) => return Err(e),
                _ => return Ok(())
            };
        }).await;

        // Unpack the thread result.
        let db_result = match insert_thread_result {
            Ok(result) => result,
            _ => return response_error!("thread error", PairResponse)
        };

        // Unpack the database result.
        if let Ok(_) = db_result {
            pair_response
        } else {
            response_error!("database write error", PairResponse)
        }
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