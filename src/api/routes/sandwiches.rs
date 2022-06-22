use actix_web::{ get, web };
use serde::{ Serialize, Deserialize };
use diesel::NotFound;
use crate::state::AppState;
use super::super::evm;
use super::super::db;

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

    // Check if the search range for this scan starts between
    // the earliest scanned block and the latest scanned block:
    //
    //          `earliest`                       `latest`
    //          |  `after`                       |     
    //          |  |                  |          |           
    // ---------+--+==================+----------+--------->
    //          |  |                  |          |           
    //              \----------------/|                      
    //                 `max_blocks`   `before`  
    //
    // OR
    //
    //                `earliest`                 `latest`
    //       `after`  |                          |     
    //             |  |               |          |           
    // ------------+--+===============+----------+--------->
    //             |  |               |          |           
    //              \----------------/|                      
    //                 `max_blocks`   `before`  
    //
    // Legend: the scan range is denoted by equal signs ======

    if let Some(latest) = pair.latest_scanned_block {
        if let Some(earliest) = pair.earliest_scanned_block {
            let earliest = earliest as u64;
            let latest = latest as u64;

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

                        // Convert the vector of db LunchmeatTransaction structures 
                        // into a vector of TransactionData structures.
                        let mut lunchmeats = Vec::new();

                        for db_lunchmeat in &db_lunchmeats {
                            lunchmeats.push(TransactionData {
                                hash: db_lunchmeat.tx_hash.clone(),
                                index: db_lunchmeat.tx_index as usize,
                                base_in: db_lunchmeat.base_in,
                                quote_in: db_lunchmeat.quote_in,
                                base_out: db_lunchmeat.base_out,
                                quote_out: db_lunchmeat.quote_out,
                                gas: db_lunchmeat.gas
                            });
                        }
                        
                        sandwiches.push(SandwichData {
                            block_number: db_sandwich.block_number as u64,
                            frontrun: TransactionData {
                                hash: db_frontrun.tx_hash,
                                index: db_frontrun.tx_index as usize,
                                base_in: db_frontrun.base_in,
                                quote_in: db_frontrun.quote_in,
                                base_out: db_frontrun.base_out,
                                quote_out: db_frontrun.quote_out,
                                gas: db_frontrun.gas
                            },
                            lunchmeat: lunchmeats,
                            backrun: TransactionData {
                                hash: db_backrun.tx_hash,
                                index: db_backrun.tx_index as usize,
                                base_in: db_backrun.base_in,
                                quote_in: db_backrun.quote_in,
                                base_out: db_backrun.base_out,
                                quote_out: db_backrun.quote_out,
                                gas: db_backrun.gas
                            }
                        });
                    }

                    Ok(sandwiches)
                }).await;

                // Unpack the database result. 
                let sandwiches = match thread_unwrap!(sandwiches_thread_result, SandwichesResponse) {
                    Ok(sandwich_vector) => sandwich_vector,
                    Err(_) => return response_error!("sandwiches read error", SandwichesResponse)
                };

                // Create and return the SandwichesResponse.
                return web::Json(SandwichesResponse {
                    sandwiches: Some(sandwiches),
                    metadata: Some(ScannerMetadata {
                        latest_fetched_block: max_le_block as u64,
                        earliest_fetched_block: min_ge_block as u64,
                        latest_scanned_block: latest,
                        earliest_scanned_block: earliest,
                        scanning_latest: pair.scanning_latest,
                        scanning_previous: pair.scanning_previous
                    }),
                    error_message: "".to_string()
                });
            }
        }
    } 

    // No scanning has taken place on this pair before,
    // so begin the scan at the `before` block.


    // under construction
    response_error!("under construction", SandwichesResponse)
}