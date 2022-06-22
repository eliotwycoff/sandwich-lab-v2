use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::result::Error as DbError;
use std::env;
use r2d2;
use super::models::{ 
    Token, 
    Pair, 
    Sandwich, 
    FrontrunTransaction, 
    LunchmeatTransaction, 
    BackrunTransaction };

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

// Initialize the database pool for use
// throughout the entire application.
pub fn init_db_pool() -> Pool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable not found");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    Pool::new(manager).expect("error creating db pool")
}

// Fetch the token with the given parameters,
// or return Err(NotFound).
pub fn fetch_token_by_params(
    db_connection: &DbConnection,
    blockchain_id: &str,
    token_addr: &str
) -> Result<Token, DbError> {
    use crate::api::schema::tokens::dsl::*;

    tokens
        .filter(token_address.eq(token_addr.to_lowercase()))
        .filter(blockchain_str_id.eq(blockchain_id.to_lowercase()))
        .first(db_connection)
}

// Fetch the token with the given token_id,
// or return Err(NotFound).
pub fn fetch_token_by_id(
    db_connection: &DbConnection,
    tid: i32
) -> Result<Token, DbError> {
    use crate::api::schema::tokens::dsl::*;

    tokens
        .find(tid)
        .first(db_connection)
}

// Take the parameters for a new token;
// then insert it and return the new token_id.
pub fn insert_token(
    db_connection: &DbConnection,
    token_nm: &str,
    token_sym: &str,
    token_dec: i16,
    blockchain_id: &str,
    token_addr: &str
) -> Result<i32, DbError> {
    use crate::api::schema::tokens::dsl::*;

    let values = (
        token_name.eq(token_nm),
        token_symbol.eq(token_sym),
        decimals.eq(token_dec),
        blockchain_str_id.eq(blockchain_id.to_lowercase()),
        token_address.eq(token_addr.to_lowercase())
    );

    Ok(diesel::insert_into(tokens)
        .values(values)
        .returning(token_id)
        .get_result(db_connection)?)
}

// Fetch the pair with the given parameters,
// or return Err(NotFound).
pub fn fetch_pair_by_params(
    db_connection: &DbConnection, 
    blockchain_id: &str, 
    pair_addr: &str
) -> Result<Pair, DbError> {
    use crate::api::schema::pairs::dsl::*;

    pairs
        .filter(pair_address.eq(pair_addr.to_lowercase()))
        .filter(blockchain_str_id.eq(blockchain_id.to_lowercase()))
        .first(db_connection) // returns Ok(record) if found else Err(NotFound)
}

// Fetch the pair with the given pair_id,
// or return Err(NotFound).
pub fn fetch_pair_by_id(
    db_connection: &DbConnection,
    pid: i32
) -> Result<Pair, DbError> {
    use crate::api::schema::pairs::dsl::*;

    pairs
        .find(pid)
        .first(db_connection)
}

// Take the parameters for a new pair;
// then insert it and return the new pair_id.
pub fn insert_pair(
    db_connection: &DbConnection,
    blockchain_id: &str,
    factory_addr: &str,
    pair_addr: &str,
    base_id: i32,
    quote_id: i32
) -> Result<i32, DbError> {
    use crate::api::schema::pairs::dsl::*;

    let values = (
        blockchain_str_id.eq(blockchain_id.to_lowercase()),
        factory_address.eq(factory_addr.to_lowercase()),
        pair_address.eq(pair_addr.to_lowercase()),
        base_token_id.eq(base_id),
        quote_token_id.eq(quote_id)
    );

    Ok(diesel::insert_into(pairs)
        .values(values)
        .returning(pair_id)
        .get_result(db_connection)?)
}

// Fetch all sandwiches for a given pair_id and block_number range
// or return Err(NotFound).
pub fn fetch_all_sandwiches_by_params(
    db_connection: &DbConnection,
    pid: i32,
    min_ge_block: Option<i64>,
    max_le_block: Option<i64>
) -> Result<Vec<Sandwich>, DbError> {
    use crate::api::schema::sandwiches::dsl::*;

    let base_query = sandwiches.filter(pair_id.eq(pid));

    if let Some(min_block) = min_ge_block {
        if let Some(max_block) = max_le_block {
            base_query
                .filter(block_number.between(min_block, max_block))
                .load::<Sandwich>(db_connection)
        } else {
            base_query
                .filter(block_number.ge(min_block))
                .load::<Sandwich>(db_connection)
        }
    } else {
        if let Some(max_block) = max_le_block {
            base_query
                .filter(block_number.le(max_block))
                .load::<Sandwich>(db_connection)
        } else {
            base_query
                .load::<Sandwich>(db_connection)
        }
    }
}

// Fetch the sandwich with the given sandwich_id,
// or return Err(NotFound).
pub fn fetch_sandwich_by_id(
    db_connection: &DbConnection,
    sid: i64
) -> Result<Sandwich, DbError> {
    use crate::api::schema::sandwiches::dsl::*;

    sandwiches
        .find(sid)
        .first(db_connection)
}

// Fetch the frontrun transaction for a given sandwich_id
// or return Err(NotFound).
pub fn fetch_frontrun_transaction_by_sandwich_id(
    db_connection: &DbConnection,
    sid: i64
) -> Result<FrontrunTransaction, DbError> {
    use crate::api::schema::frontrun_transactions::dsl::*;

    frontrun_transactions
        .find(sid)
        .first(db_connection)
}

// Fetch all lunchmeat transations for a given sandwich_id
// or return Err(NotFound).
pub fn fetch_lunchmeat_transactions_by_sandwich_id(
    db_connection: &DbConnection,
    sid: i64
) -> Result<Vec<LunchmeatTransaction>, DbError> {
    use crate::api::schema::lunchmeat_transactions::dsl::*;

    lunchmeat_transactions
        .filter(sandwich_id.eq(sid))
        .load::<LunchmeatTransaction>(db_connection)
}

// Fetch the backrun transaction for a given sandwich_id
// or return Err(NotFound).
pub fn fetch_backrun_transaction_by_sandwich_id(
    db_connection: &DbConnection,
    sid: i64
) -> Result<BackrunTransaction, DbError> {
    use crate::api::schema::backrun_transactions::dsl::*;

    backrun_transactions
        .find(sid)
        .first(db_connection)
}