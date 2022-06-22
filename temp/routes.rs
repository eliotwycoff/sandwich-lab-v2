use actix_web::{ get, web, HttpResponse, Responder };
use serde::{ Serialize, Deserialize };
use diesel::NotFound;
use crate::state::AppState;
use super::evm;
use super::db;

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
    }
}

// This helper macro tries to cast the passed value as an i64 
// and returns the appropriate error if an overflow occurs.
#[macro_export]
macro_rules! into_i64 {
    ($val:expr, $response:ident) => {
        match i64::try_from($val) {
            Ok(int) => int,
            Err(_) => return response_error!("numeric overflow", $response)
        }
    }
}

// This helper macro tries to unpack a web::block thread result
// and returns the appropriate error if the result is an error.
#[macro_export]
macro_rules! thread_unwrap {
    ($thread_result:expr, $response:ident) => {
        match $thread_result {
            Ok(result) => result,
            Err(_) => return response_error!("thread error", $response)
        }
    }
}

#[derive(Debug, Deserialize)]
struct PairRequest {
    blockchain: String,
    pair_address: String
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
    exchange_name: String,
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
async fn fetch_pair(
    data: web::Data<AppState>, 
    info: web::Query<PairRequest>
) -> web::Json<PairResponse> {
    // Get a database connection, and return an error
    // if a connection cannot be established.
    let db_connection = get_db_connection!(data, PairResponse);

    // Standardize the incoming data.
    let blockchain_id = info.blockchain.to_lowercase();
    let pair_address = info.pair_address.to_lowercase();

    // First get the blockchain state data, or return an error.
    let blockchain = match data.blockchains.get(&blockchain_id) {
        Some(blockchain) => blockchain,
        None => return response_error!("blockchain not supported", PairResponse)
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

    // Unpack the database result.
    if let Ok((pair, base, quote)) = thread_unwrap!(metadata_thread_result, PairResponse) {
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

        // Get the exchange name.
        let exchange_name = match blockchain.exchanges.get(&pair.factory_address) {
            Some(exchange) => exchange.name().to_string(),
            None => return response_error!("exchange not found", PairResponse)
        };

        // Construct and return the PairResponse JSON object.
        web::Json(PairResponse {
            pair: Some(PairMetadata {
                address: pair.pair_address,
                exchange_name: exchange_name,
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

        // Asynchronously get the pair metadata from the blockchain.
        let metadata = match evm::fetch_pair_metadata(
            &blockchain.provider_url,
            &pair_address,
            &blockchain.data_aggregator_address).await {
            
            Ok(metadata) => metadata,
            _ => return response_error!("provider error", PairResponse)
        };

        // Standardize the addresses.
        let factory_address = format!("0x{}", metadata.factory_address.to_lowercase());
        let base_address = format!("0x{}", metadata.base_address.to_lowercase());
        let quote_address = format!("0x{}", metadata.quote_address.to_lowercase());

        // Check for exchange state data, or return an error.
        let exchange_name = match blockchain.exchanges.get(&factory_address) {
            Some(exchange) => exchange.name().to_string(),
            None => return response_error!("exchange not supported", PairResponse)
        };

        // Clone the metadata for use inside the following closure.
        let metadata_clone = metadata.clone();

        // Create the PairResponse object.
        let pair_response = web::Json(PairResponse {
            pair: Some(PairMetadata {
                address: pair_address.clone(),
                exchange_name: exchange_name,
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
        let blockchain_id = info.blockchain.to_lowercase();

        // Spawn a new, non-blocking thread to save
        // the pair, base and quote to the database.
        let insert_thread_result = web::block(move || {
            // Commit the pair's base token to the database, if necessary.
            let base_id = match db::fetch_token_by_params(
                &db_connection,
                &blockchain_id,
                &base_address) {

                Ok(base_token) => base_token.token_id,
                Err(NotFound) => {
                    match db::insert_token(
                        &db_connection,
                        &metadata_clone.base_name,
                        &metadata_clone.base_symbol,
                        metadata_clone.base_decimals as i16,
                        &blockchain_id,
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
                &blockchain_id,
                &quote_address) {
                
                Ok(quote_token) => quote_token.token_id,
                Err(NotFound) => {
                    match db::insert_token(
                        &db_connection,
                        &metadata_clone.quote_name,
                        &metadata_clone.quote_symbol,
                        metadata_clone.quote_decimals as i16,
                        &blockchain_id,
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
                &blockchain_id,
                &factory_address,
                &pair_address,
                base_id,
                quote_id) {
                
                Err(e) => return Err(e),
                _ => return Ok(())
            };
        }).await;

        // Unpack the database result.
        if let Ok(_) = thread_unwrap!(insert_thread_result, PairResponse) {
            pair_response
        } else {
            response_error!("database write error", PairResponse)
        }
    }
}

#[derive(Debug, Deserialize)]
struct SandwichesRequest {
    blockchain: String,
    pair_address: String,
    before: Option<u64>
}

#[derive(Debug, Serialize)]
struct SandwichesResponse {
    sandwiches: Option<Vec<SandwichData>>,
    metadata: Option<ScannerMetadata>,
    error_message: String
}

impl SandwichesResponse {
    fn as_error(msg: String) -> Self {
        Self {
            sandwiches: None,
            metadata: None,
            error_message: msg
        }
    }
}

#[derive(Debug, Serialize)]
struct SandwichData {
    block_number: u64,
    frontrun: TransactionData,
    lunchmeat: Vec<TransactionData>,
    backrun: TransactionData
}

#[derive(Debug, Serialize)]
struct TransactionData {
    hash: String,
    index: usize,
    base_in: f64,
    quote_in: f64,
    base_out: f64,
    quote_out: f64,
    gas: f64
}

#[derive(Debug, Serialize)]
struct ScannerMetadata {
    latest_fetched_block: u64,
    earliest_fetched_block: u64,
    latest_scanned_block: u64,
    earliest_scanned_block: u64,
    scanning_latest: bool,
    scanning_previous: bool
}

#[get("/sandwiches")]
async fn fetch_sandwiches(
    data: web::Data<AppState>, 
    info: web::Query<SandwichesRequest>
) -> web::Json<SandwichesResponse> {
    // Standardize the incoming data.
    let blockchain_id = info.blockchain.to_lowercase();
    let pair_address = info.pair_address.to_lowercase();

    // First get the blockchain state data, or return an error.
    let blockchain = match data.blockchains.get(&blockchain_id) {
        Some(blockchain) => blockchain,
        None => return response_error!("blockchain not supported", SandwichesResponse)
    };

    // Determine from which block to begin the reverse-chronological scan.
    let before = match info.before {
        Some(block_number) => block_number,
        None => {
            match evm::fetch_latest_block_number(&blockchain.provider_url).await {
                Ok(block_number) => block_number,
                _ => return response_error!("provider error", SandwichesResponse)
            }
        }
    };

    // Get a database connection, and return an error
    // if a connection cannot be established.
    let db_connection = get_db_connection!(data, SandwichesResponse);

    // Spawn a new, non-blocking thread to fetch
    // the pair from the database, if it exists.
    let pair_thread_result = web::block(move || {
        db::fetch_pair_by_params(
            &db_connection,
            &blockchain_id,
            &pair_address)
    }).await;

    // Unpack the database result.
    let pair = match thread_unwrap!(pair_thread_result, SandwichesResponse) {
        Ok(pair) => pair,
        Err(NotFound) => return response_error!("pair does not exist", SandwichesResponse),
        Err(_) => return response_error!("database error", SandwichesResponse)
    };

    // Set up helper variables.
    let max_blocks = blockchain.scanner_params.max_blocks_per_request;
    let pair_id = pair.pair_id;

    // Determine at which block the search must end.
    let after = if before > max_blocks {
        before - max_blocks
    } else {
        0
    };             

    // Check if the search range for this scan begins
    // within the block range that's already been scanned.
    if let Some(latest) = pair.latest_scanned_block {
        if let Some(earliest) = pair.earliest_scanned_block {
            let earliest = earliest as u64;
            let latest = latest as u64;

            //          `earliest`                       `latest`
            //          |  `after`                       |     
            //          |  |                  |          |           
            // ---------+--+==================+----------+--------->
            //          |  |                  |          |           
            //              \----------------/|                      
            //                 `max_blocks`    `before`  

            if earliest < before && before <= latest {
                // We have scanned this region of blocks already.
                // So return up to `max_blocks` worth of sandwiches.
                let min_ge_block = if after > earliest { 
                    into_i64!(after, SandwichesResponse)
                } else { 
                    into_i64!(earliest, SandwichesResponse)
                };

                let max_le_block = into_i64!(before, SandwichesResponse);

                // Get a database connection, and return an error
                // if a connection cannot be established.
                let db_connection = get_db_connection!(data, SandwichesResponse);

                // Spawn a new, non-blocking thread to fetch
                // the sandwiches from the database, if they exist.
                let sandwiches_thread_result = web::block(move || {
                    // Get the sandwiches in the database's Sandwich model form.
                    let db_sandwiches = match db::fetch_all_sandwiches_by_params(
                        &db_connection,
                        pair_id,
                        Some(min_ge_block),
                        Some(max_le_block)) {

                        Ok(sandwich_vector) => sandwich_vector,
                        Err(e) => return Err(e)
                    };

                    // Create a vector of sandwiches in SandwichData form.
                    let sandwiches: Vec<SandwichData> = Vec::new();

                    for db_sandwich in &db_sandwiches {
                        // Get the frontrun transaction for this sandwich.
                        
                    }

                    // under construction
                    Ok(())
                }).await;

                // Unpack the database result. 
                let sandwiches = match thread_unwrap!(sandwiches_thread_result, SandwichesResponse) {
                    Ok(sandwich_vector) => sandwich_vector,
                    Err(_) => return response_error!("sandwiches read error", SandwichesResponse)
                };

                
            }
        }
    } 

    // No scanning has taken place on this pair before,
    // so begin the scan at the `before` block.


    // under construction
    response_error!("under construction", SandwichesResponse)
}

// Package up all the api routes into a ServiceConfig
// that can be registered on startup in main.rs.
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
        .service(fetch_pair)
        .service(fetch_sandwiches));
}