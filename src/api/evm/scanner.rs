use ethers::prelude::{ Provider, Http, Contract, LogMeta };
use ethers::abi::AbiParser;
use ethers::types::{ Address };
use super::super::{ db, models };
use super::swap::{ RawSwapV2, RawSwapV3, SwapCore, Swap, to_wrapped };
use super::sandwich::parse_sandwiches;
use crate::state::Exchange;
use tokio::runtime::Runtime;
use std::thread;
use std::error::Error;
use std::collections::HashMap;

// This helper macro tries the given expression
// or logs the given range (by id) as failed.
#[macro_export]
macro_rules! try_or_log_error {
    ($expression:expr, $db_conn:expr, $range_id:expr) => {
        match $expression {
            Ok(value) => value,
            Err(_) => {
                db::update_range_metadata(&$db_conn, $range_id, false, true).unwrap();
                return
            }
        }
    }
}

// This helper macro gets the lower bound 
// for a Swap search range request.
#[macro_export]
macro_rules! get_lower_bound {
    ($upper:expr, $size:expr, $min_lower:expr) => {
        if $upper > $size {
            if $upper - $size > $min_lower { 
                $upper - $size
            } else {
                $min_lower
            }
        } else {
            $min_lower
        }
    }
}

// This struct holds search parameters.
#[derive(Debug, Clone)]
pub struct Params {
    pub blocks_per_chunk: u64,
    pub max_blocks_per_chunk: u64,
    pub target_swaps_per_chunk: u64,
    pub max_blocks_per_request: u64
}

// Start scanning for sandwiches on the given pair
// over the given range of blocks, inside a new thread.
pub fn start_scan_job(
    db_connection: db::DbConnection,
    provider_url: String,
    pair: models::Pair,
    exchange: Exchange,
    native_decimals: u8,
    range: models::Range,
    params: Params
) -> bool {
    match thread::Builder::new().spawn(move || {
        // Now that we're in a brand-new thread, create a new tokio runtime
        // so that we can make async blockchain provider calls via ethers.
        let runtime = try_or_log_error!(Runtime::new(), db_connection, range.range_id);
        let range_id = range.range_id;

        try_or_log_error!(runtime.block_on(async { 
            run_scan_loop(
                &db_connection, 
                provider_url, 
                pair, 
                exchange, 
                native_decimals,
                range, 
                params).await }), db_connection, range_id);
    }) {
        Ok(_) => true, // the thread was successfully created
        Err(_) => false // failed to create the thread
    }
}

async fn run_scan_loop(
    db_connection: &db::DbConnection,
    provider_url: String,
    pair: models::Pair,
    exchange: Exchange,
    native_decimals: u8,
    range: models::Range,
    params: Params
) -> Result<(), Box<dyn Error>> {
    let pair_abi = AbiParser::default().parse_str("")?;
    let provider = Provider::<Http>::try_from(provider_url.clone())?;
    let address = pair.pair_address.parse::<Address>()?;
    let contract = Contract::new(address, pair_abi.clone(), provider.clone());
    let base = db::fetch_token_by_id(&db_connection, pair.base_token_id)?;
    let quote = db::fetch_token_by_id(&db_connection, pair.quote_token_id)?;

    let mut blocks_per_chunk = params.blocks_per_chunk;
    let mut upper = range.upper_bound as u64;
    let mut lower = get_lower_bound!(upper, blocks_per_chunk, range.lower_bound as u64);

    while upper >= range.lower_bound as u64 {
        println!("\nLower: {lower}\nUpper: {upper}\nLower Bound: {}", range.lower_bound);
        let swaps = match exchange {
            Exchange::V2 { name: _ } => {
                let raw_swaps: Vec<(RawSwapV2, LogMeta)> = contract.event()
                   .from_block::<u64>(lower).to_block::<u64>(upper).query_with_meta().await?;

                raw_swaps.into_iter()
                    .map(|raw_swap| to_wrapped(
                        SwapCore::from(raw_swap), native_decimals, &base, &quote))
                    .collect::<Vec<Swap>>()
            },
            Exchange::V3 { name: _ } => {
                let raw_swaps: Vec<(RawSwapV3, LogMeta)> = contract.event()
                    .from_block::<u64>(lower).to_block::<u64>(upper).query_with_meta().await?;

                raw_swaps.into_iter()
                    .map(|raw_swap| to_wrapped(
                        SwapCore::from(raw_swap), native_decimals, &base, &quote))
                    .collect::<Vec<Swap>>()
            }
        };

        let total_swaps = swaps.len();
        println!(" -- Fetched {total_swaps} swaps!");

        // Group swaps by block, and filter out blocks with less than three swaps.
        let mut swaps_per_block: HashMap<u64, u64> = HashMap::new();
        let mut swaps_by_block: HashMap<u64, Vec<Swap>> = HashMap::new();

        for swap in swaps.into_iter() {
            *swaps_per_block.entry(swap.swap.block_number()).or_insert(0) += 1;
            swaps_by_block.entry(swap.swap.block_number()).or_insert(Vec::new()).push(swap);
        }

        for block in swaps_per_block.keys() {
            if *swaps_per_block.get(block).unwrap() < 3 {
                swaps_by_block.remove(block);
            }
        }

        println!(" -- A total of {} blocks have 3 or more swaps.", swaps_by_block.len());

        // Look for and save any sandwich trades in blocks with at least three swaps. 
        for block in swaps_by_block.keys() {
            let mut bundle = swaps_by_block.get(block).unwrap().to_vec();
            bundle.sort_by_key(|s| s.swap.tx_index());

            // Pull sandwich data from the bundle of swaps,
            // and save these sandwiches to the database.
            for sandwich in parse_sandwiches(&bundle, &provider_url).await? {
                let db_sandwich = db::insert_sandwich(
                    db_connection,
                    pair.pair_id,
                    i64::try_from(*block)?)?;

                // Insert the frontrun transaction.
                db::insert_frontrun_transaction(
                    db_connection,
                    &format!("0x{}", sandwich.frontrun.swap.tx_hash()),
                    sandwich.frontrun.swap.tx_index(),
                    sandwich.frontrun.in0(),
                    sandwich.frontrun.in1(),
                    sandwich.frontrun.out0(),
                    sandwich.frontrun.out1(),
                    sandwich.frontrun.gas(),
                    db_sandwich.sandwich_id)?;

                // Insert the lunchmeat transaction(s).
                for i in 0..sandwich.lunchmeat.len() {
                    db::insert_lunchmeat_transaction(
                        db_connection,
                        &format!("0x{}", sandwich.lunchmeat[i].swap.tx_hash()),
                        sandwich.lunchmeat[i].swap.tx_index(),
                        sandwich.lunchmeat[i].in0(),
                        sandwich.lunchmeat[i].in1(),
                        sandwich.lunchmeat[i].out0(),
                        sandwich.lunchmeat[i].out1(),
                        sandwich.lunchmeat[i].gas(),
                        db_sandwich.sandwich_id)?;
                }

                // Insert the backrun transaction.
                db::insert_backrun_transaction(
                    db_connection,
                    &format!("0x{}", sandwich.backrun.swap.tx_hash()),
                    sandwich.backrun.swap.tx_index(),
                    sandwich.backrun.in0(),
                    sandwich.backrun.in1(),
                    sandwich.backrun.out0(),
                    sandwich.backrun.out1(),
                    sandwich.backrun.gas(),
                    db_sandwich.sandwich_id)?;
            }
        }

        // Update the block search range.
        let swap_density = total_swaps as f64 / (upper - lower + 1) as f64;
        blocks_per_chunk = (params.target_swaps_per_chunk as f64 / swap_density).floor() as u64;

        if blocks_per_chunk > params.max_blocks_per_chunk {
            blocks_per_chunk = params.max_blocks_per_chunk
        }

        upper = lower - 1;
        lower = get_lower_bound!(upper, blocks_per_chunk, range.lower_bound as u64);
    }

    // Update and mark this range as complete.
    db::update_range_metadata(&db_connection, range.range_id, true, false).unwrap();

    // Return without error.
    Ok(())
}