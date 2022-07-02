use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::result::Error as DbError;
use diesel::dsl::max;
use diesel::{ insert_into, update/*, delete*/ };
use std::env;
use r2d2;
use super::models::{ 
    Token, 
    Pair, 
    Range,
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
    let pool = Pool::new(manager).expect("error creating db pool");

    // Run any pending migrations.
    embed_migrations!();
    embedded_migrations::run(&pool.get()
        .expect("error getting database connection"))
        .expect("error running pending migrations");

    // Return the database pool.
    pool
}

// Fetch the token with the given parameters,
// or return `Err(NotFound)`.
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
// or return `Err(NotFound)`.
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
// then insert it and return the new `token_id`.
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

    insert_into(tokens)
        .values(values)
        .returning(token_id)
        .get_result(db_connection)
}

// Fetch the pair with the given parameters,
// or return `Err(NotFound)`.
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

// Fetch the pair with the given `pair_id`,
// or return `Err(NotFound)`.
/*pub fn fetch_pair_by_id(
    db_connection: &DbConnection,
    pid: i32
) -> Result<Pair, DbError> {
    use crate::api::schema::pairs::dsl::*;

    pairs
        .find(pid)
        .first(db_connection)
}*/

// Take the parameters for a new pair;
// then insert it and return the new `pair_id`.
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

    insert_into(pairs)
        .values(values)
        .returning(pair_id)
        .get_result(db_connection)
}

// Given a `block_number` and a `pair_id`, find the range,
// if any, that contains that block.
pub fn find_encompassing_range(
    db_connection: &DbConnection,
    pid: i32,
    block_number: i64
) -> Result<Range, DbError> {
    use crate::api::schema::ranges::dsl::*;

    ranges
        .filter(pair_id.eq(pid))
        .filter(lower_bound.le(block_number))
        .filter(upper_bound.ge(block_number))
        .first(db_connection)
}

// Given a `block_number` and a `pair_id`, find the upper bound
// of the range, if any, that immediately precedes that block.
pub fn find_preceding_range_upper_bound(
    db_connection: &DbConnection,
    pid: i32,
    block_number: i64
) -> Result<Option<i64>, DbError> {
    use crate::api::schema::ranges::dsl::*;

    ranges
        .select(max(upper_bound))
        .filter(pair_id.eq(pid))
        .filter(upper_bound.lt(block_number))
        .first(db_connection)
}

// Insert a new range.
pub fn insert_range(
    db_connection: &DbConnection,
    pid: i32,
    lb: i64,
    ub: i64,
    complete: bool,
    failed: bool
) -> Result<Range, DbError> {
    use crate::api::schema::ranges::dsl::*;

    let values = (
        pair_id.eq(pid),
        lower_bound.eq(lb),
        upper_bound.eq(ub),
        scan_complete.eq(complete),
        scan_failed.eq(failed)
    );

    insert_into(ranges)
        .values(values)
        .get_result(db_connection)
}

// Update a range's scan-related metadata.
pub fn update_range_metadata(
    db_connection: &DbConnection,
    rid: i64,
    complete: bool,
    failed: bool
) -> Result<i64, DbError> {
    use crate::api::schema::ranges::dsl::*;

    update(ranges.filter(range_id.eq(rid)))
        .set((scan_complete.eq(complete), scan_failed.eq(failed)))
        .returning(range_id)
        .get_result(db_connection)
}

// Delete the range given by `rid`.
/*pub fn delete_range(
    db_connection: &DbConnection,
    rid: i64
) -> Result<usize, DbError> {
    use crate::api::schema::ranges::dsl::*;

    delete(ranges.filter(range_id.eq(rid))).execute(db_connection)
}*/

// Fetch all sandwiches for a given `pair_id` and `block_number` range
// or return `Err(NotFound)`.
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

// Fetch the sandwich with the given `sandwich_id`,
// or return `Err(NotFound)`.
/*pub fn fetch_sandwich_by_id(
    db_connection: &DbConnection,
    sid: i64
) -> Result<Sandwich, DbError> {
    use crate::api::schema::sandwiches::dsl::*;

    sandwiches
        .find(sid)
        .first(db_connection)
}*/

// Insert a new sandwich. 
pub fn insert_sandwich(
    db_connection: &DbConnection,
    pid: i32,
    block: i64
) -> Result<Sandwich, DbError> {
    use crate::api::schema::sandwiches::dsl::*;

    let values = (
        pair_id.eq(pid),
        block_number.eq(block)
    );

    insert_into(sandwiches)
        .values(values)
        .get_result(db_connection)
}

// Fetch the frontrun transaction for a given `sandwich_id`
// or return `Err(NotFound)`.
pub fn fetch_frontrun_transaction_by_sandwich_id(
    db_connection: &DbConnection,
    sid: i64
) -> Result<FrontrunTransaction, DbError> {
    use crate::api::schema::frontrun_transactions::dsl::*;

    frontrun_transactions
        .find(sid)
        .first(db_connection)
}

// Insert a new frontrun transaction.
pub fn insert_frontrun_transaction(
    db_connection: &DbConnection,
    hash: &str,
    idx: i32,
    t0_in: f64,
    t1_in: f64,
    t0_out: f64,
    t1_out: f64,
    gs: f64,
    sid: i64
) -> Result<FrontrunTransaction, DbError> {
    use crate::api::schema::frontrun_transactions::dsl::*;

    let values = (
        tx_hash.eq(hash),
        tx_index.eq(idx),
        base_in.eq(t0_in),
        quote_in.eq(t1_in),
        base_out.eq(t0_out),
        quote_out.eq(t1_out),
        gas.eq(gs),
        sandwich_id.eq(sid)
    );

    insert_into(frontrun_transactions)
        .values(values)
        .get_result(db_connection)
}

// Fetch all lunchmeat transations for a given `sandwich_id`
// or return `Err(NotFound)`.
pub fn fetch_lunchmeat_transactions_by_sandwich_id(
    db_connection: &DbConnection,
    sid: i64
) -> Result<Vec<LunchmeatTransaction>, DbError> {
    use crate::api::schema::lunchmeat_transactions::dsl::*;

    lunchmeat_transactions
        .filter(sandwich_id.eq(sid))
        .load::<LunchmeatTransaction>(db_connection)
}

// Insert a new lunchmeat transaction.
pub fn insert_lunchmeat_transaction(
    db_connection: &DbConnection,
    hash: &str,
    idx: i32,
    t0_in: f64,
    t1_in: f64,
    t0_out: f64,
    t1_out: f64,
    gs: f64,
    sid: i64
) -> Result<LunchmeatTransaction, DbError> {
    use crate::api::schema::lunchmeat_transactions::dsl::*;

    let values = (
        tx_hash.eq(hash),
        tx_index.eq(idx),
        base_in.eq(t0_in),
        quote_in.eq(t1_in),
        base_out.eq(t0_out),
        quote_out.eq(t1_out),
        gas.eq(gs),
        sandwich_id.eq(sid)
    );

    insert_into(lunchmeat_transactions)
        .values(values)
        .get_result(db_connection)
}

// Fetch the backrun transaction for a given `sandwich_id`
// or return `Err(NotFound)`.
pub fn fetch_backrun_transaction_by_sandwich_id(
    db_connection: &DbConnection,
    sid: i64
) -> Result<BackrunTransaction, DbError> {
    use crate::api::schema::backrun_transactions::dsl::*;

    backrun_transactions
        .find(sid)
        .first(db_connection)
}

// Insert a new backrun transaction.
pub fn insert_backrun_transaction(
    db_connection: &DbConnection,
    hash: &str,
    idx: i32,
    t0_in: f64,
    t1_in: f64,
    t0_out: f64,
    t1_out: f64,
    gs: f64,
    sid: i64
) -> Result<BackrunTransaction, DbError> {
    use crate::api::schema::backrun_transactions::dsl::*;

    let values = (
        tx_hash.eq(hash),
        tx_index.eq(idx),
        base_in.eq(t0_in),
        quote_in.eq(t1_in),
        base_out.eq(t0_out),
        quote_out.eq(t1_out),
        gas.eq(gs),
        sandwich_id.eq(sid)
    );

    insert_into(backrun_transactions)
        .values(values)
        .get_result(db_connection)
}