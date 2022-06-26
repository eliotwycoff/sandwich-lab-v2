use actix_web::{ get, web };
use serde::{ Serialize, Deserialize };
use diesel::NotFound;
use diesel::result::Error as DbError;
use crate::state::AppState;
use super::super::{ evm, db };
use super::super::models::{ Range, FrontrunTransaction, LunchmeatTransaction, BackrunTransaction };
use std::convert::From;

#[derive(Debug, Deserialize)]
struct SandwichesRequest {
    blockchain: String,
    pair_address: String,
    before: Option<u64>
}

#[derive(Debug, Serialize)]
struct SandwichesResponse {
    sandwiches: Option<Vec<SandwichData>>,
    token_metadata: Option<TokenMetadata>,
    fetch_metadata: Option<FetchMetadata>,
    scan_metadata: Option<ScanMetadata>,
    error_message: String
}

impl SandwichesResponse {
    fn as_error(msg: String) -> Self {
        Self {
            sandwiches: None,
            token_metadata: None,
            fetch_metadata: None,
            scan_metadata: None,
            error_message: msg
        }
    }
}

#[derive(Debug, Serialize)]
struct TokenMetadata {
    base_symbol: String,
    quote_symbol: String,
    native_symbol: String
}

#[derive(Debug, Serialize)]
struct FetchMetadata {
    lower_bound: u64,
    upper_bound: u64
}

#[derive(Debug, Serialize)]
struct ScanMetadata {
    lower_bound: u64,
    upper_bound: u64,
    complete: bool,
    failed: bool
}

impl From<Range> for ScanMetadata {
    fn from(range: Range) -> Self {
        Self {
            lower_bound: range.lower_bound as u64,
            upper_bound: range.upper_bound as u64,
            complete: range.scan_complete,
            failed: range.scan_failed
        }
    }
}

#[derive(Debug, Serialize)]
struct SandwichData {
    block_number: i64,
    frontrun: TransactionData,
    lunchmeat: Vec<TransactionData>,
    backrun: TransactionData
}

#[derive(Debug, Serialize)]
pub struct TransactionData {
    hash: String,
    index: usize,
    base_in: f64,
    quote_in: f64,
    base_out: f64,
    quote_out: f64,
    gas: f64
}

// Implement the From trait to convert from the following
// database structs into the TransactionData struct.
implement_transaction_data_from!(FrontrunTransaction);
implement_transaction_data_from!(LunchmeatTransaction);
implement_transaction_data_from!(BackrunTransaction);

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

    // Get a database connection, and return an error
    // if a connection cannot be established.
    let db_connection = get_db_connection!(data, SandwichesResponse);

    // Spawn a new, non-blocking thread to fetch
    // the pair from the database, if it exists.
    let pair_thread_result = web::block(move || {
        db::fetch_pair_by_params(&db_connection, &blockchain_id, &pair_address)
    }).await;

    // Unpack the database result.
    let pair = match thread_unwrap!(pair_thread_result, SandwichesResponse) {
        Ok(pair) => pair,
        Err(NotFound) => return response_error!("pair does not exist", SandwichesResponse),
        Err(_) => return response_error!("database error", SandwichesResponse)
    };

    // Also get the exchange state data, or return an error.
    let exchange = match blockchain.exchanges.get(&pair.factory_address) {
        Some(exchange) => exchange.clone(),
        None => return response_error!("exchange not supported", SandwichesResponse)
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
    let before_block = into_i64!(before, SandwichesResponse);
    let pair_id = pair.pair_id;

    // Spawn a new, non-blocking thread to fetch
    // the encompassing range, if it exists.
    let in_range_thread_result = web::block(move || {
        db::find_encompassing_range(&db_connection, pair_id, before_block)
    }).await;

    // Get the range encompassing the `before` block, or if it doesn't exist,
    // create an appropriately sized range, and begin scanning.
    let in_range = match thread_unwrap!(in_range_thread_result, SandwichesResponse) {
        Ok(range) => range,
        Err(NotFound) => {
            // We need to create our own range; first, create helper variables.
            let db_connection = get_db_connection!(data, SandwichesResponse);
            let pair_id = pair.pair_id;
            let before_block = into_i64!(before, SandwichesResponse);
            let max_blocks = into_i64!(
                blockchain.scanner_params.max_blocks_per_request, SandwichesResponse);

            // Spawn a new, non-blocking thread to fetch the upper bound
            // of the preceding range, if it exists. Then calculate 
            // the lower bound of the new range, i.e. `after`,
            // and insert the new range into the database.
            let provider_url = blockchain.provider_url.clone();
            let pair_clone = pair.clone();
            let native_decimals = blockchain.native_token.decimals.clone();
            let params = blockchain.scanner_params.clone();

            let new_range_thread_result = web::block(move || {
                let ub = match db::find_preceding_range_upper_bound(
                    &db_connection,
                    pair_id,
                    before_block
                ) {
                    Ok(Some(number)) => number,
                    Ok(None) | Err(NotFound) => 0,
                    Err(e) => return Err(e)
                };

                // Determine `after_block`, the lower bound of the new range.
                let after_block = if before_block - max_blocks >= ub { 
                    before_block - max_blocks + 1
                } else {
                    ub + 1
                };

                // Insert the new range, [`after`, `before`]. 
                let new_range = match db::insert_range(
                    &db_connection,
                    pair_id,
                    after_block,
                    before_block,
                    false, // scan not complete
                    false // scan not failed
                ) {
                    Ok(range) => range,
                    Err(e) => return Err(e)
                };

                let range = new_range.clone();

                // Start the scan as a background job.
                if evm::scanner::start_scan_job(
                    db_connection,
                    provider_url,
                    pair_clone,
                    exchange,
                    native_decimals,
                    range, 
                    params
                ) {

                }
                Ok(new_range)
            }).await;
            
            match thread_unwrap!(new_range_thread_result, SandwichesResponse) {
                Ok(range) => {
                    // Let the user know that the scan was successfully started.
                    return web::Json(SandwichesResponse {
                        sandwiches: None,
                        token_metadata: None,
                        fetch_metadata: None,
                        scan_metadata: Some(ScanMetadata::from(range)),
                        error_message: "".to_string()
                    });
                },
                Err(_) => return response_error!("create scan range database error", SandwichesResponse)
            }
        },
        Err(_) => return response_error!("range search database error", SandwichesResponse)
    };   
    
    // If the scan of the range we're in previously failed,
    // return the range metadata to the user and let them know.
    if in_range.scan_failed {
        return web::Json(SandwichesResponse {
            sandwiches: None,
            token_metadata: None,
            fetch_metadata: None,
            scan_metadata: Some(ScanMetadata::from(in_range)),
            error_message: "scan failed".to_string()
        });
    }

    // Return up to `max_blocks` worth of sandwich data 
    // from the range, regardless of whether it has completed
    // or not; also return fetch metadata.
    let db_connection = get_db_connection!(data, SandwichesResponse);
    let pair_id = pair.pair_id;
    let before_block = into_i64!(before, SandwichesResponse);
    let max_blocks = into_i64!(
        blockchain.scanner_params.max_blocks_per_request, SandwichesResponse);

    let after_block = if before_block - max_blocks >= in_range.lower_bound {
        before_block - max_blocks
    } else {
        in_range.lower_bound
    };

    // Spawn a new, non-blocking thread to fetch up to `max_blocks` 
    // worth of sandwiches from the range, as well as token metadata.
    let base_id = pair.base_token_id;
    let quote_id = pair.quote_token_id;
    let thread_result = web::block(move || {
        (fetch_db_sandwich_data(&db_connection, pair_id, after_block, before_block),
        db::fetch_token_by_id(&db_connection, base_id),
        db::fetch_token_by_id(&db_connection, quote_id))
    }).await;

    // Get the sandwich vector by unpacking the thread result.
    let (sandwiches, base, quote) = match thread_unwrap!(thread_result, SandwichesResponse) {
        (Ok(sandwiches), Ok(base), Ok(quote)) => (sandwiches, base, quote),
        _ => return response_error!("fetch sandwiches database error", SandwichesResponse)
    };

    // Create the token metadata.
    let token_metadata = TokenMetadata {
        base_symbol: base.token_symbol,
        quote_symbol: quote.token_symbol,
        native_symbol: blockchain.native_token.symbol.clone()
    };

    // Create the fetch metadata, i.e. log the `lower_bound`
    // and the `upper_bound` of the scan (if the scan completed)
    // or of the fetched sandwiches (if the scan continues).
    let fetch_metadata = FetchMetadata {
        lower_bound: if in_range.scan_complete { after_block as u64 } else { 
            match sandwiches.iter().min_by_key(|s| s.block_number) {
                Some(sandwich) => sandwich.block_number as u64,
                None => before_block as u64
            }
        },
        upper_bound: before_block as u64
    };

    // Create and return the SandwichesResponse.
    let total = sandwiches.len();

    web::Json(SandwichesResponse {
        sandwiches: if total > 0 { Some(sandwiches) } else { None },
        token_metadata: if total > 0 { Some(token_metadata) } else { None },
        fetch_metadata: Some(fetch_metadata),
        scan_metadata: Some(ScanMetadata::from(in_range)),
        error_message: "".to_string()
    })
}

// Fetch low-level, database model instances of sandwiches
// and transactions, and compile them all into a vector
// of SandwichData structures, or return the appropriate error.
fn fetch_db_sandwich_data(
    db_connection: &db::DbConnection,
    pair_id: i32,
    min_ge_block: i64,
    max_le_block: i64
) -> Result<Vec<SandwichData>, DbError> {
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
    let mut sandwiches = Vec::new();

    for db_sandwich in &db_sandwiches {
        // Get the frontrun transaction for this sandwich.
        let db_frontrun = match db::fetch_frontrun_transaction_by_sandwich_id(
            &db_connection,
            db_sandwich.sandwich_id) {

            Ok(frontrun) => frontrun,
            Err(e) => return Err(e)
        };

        // Get all lunchmeat transactions for this sandwich.
        let db_lunchmeats = match db::fetch_lunchmeat_transactions_by_sandwich_id(
            &db_connection,
            db_sandwich.sandwich_id) {

            Ok(lunchmeat_vector) => lunchmeat_vector,
            Err(e) => return Err(e)
        };

        // Get the backrun transaction for this sandwich.
        let db_backrun = match db::fetch_backrun_transaction_by_sandwich_id(
            &db_connection,
            db_sandwich.sandwich_id) {

            Ok(backrun) => backrun,
            Err(e) => return Err(e)
        };
        
        sandwiches.push(SandwichData {
            block_number: db_sandwich.block_number,
            frontrun: TransactionData::from(&db_frontrun),
            lunchmeat: db_lunchmeats.iter()
                .map(|db_tx| TransactionData::from(db_tx))
                .collect::<Vec<TransactionData>>(),
            backrun: TransactionData::from(&db_backrun)
        });
    }
    
    Ok(sandwiches)
}