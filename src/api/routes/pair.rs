use actix_web::{ get, web };
use serde::{ Serialize, Deserialize };
use diesel::NotFound;
use crate::state::AppState;
use super::super::{ evm, db };

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
    exchange_name: String
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
            &db_connection, &blockchain_id, &pair_address
        ) {
            Ok(pair) => pair,
            Err(e) => return Err(e)
        };

        let base = match db::fetch_token_by_id(
            &db_connection, pair.base_token_id
        ) {
            Ok(token) => token,
            Err(e) => return Err(e)
        };
        
        let quote = match db::fetch_token_by_id(
            &db_connection, pair.quote_token_id
        ) {
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
                exchange_name: exchange_name
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
            &blockchain.data_aggregator_address
        ).await {
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
                exchange_name: exchange_name
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
                &base_address
            ) {
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
                &quote_address
            ) {
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