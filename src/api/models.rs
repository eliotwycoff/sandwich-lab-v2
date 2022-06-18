//#[macro_use]
use diesel::*;
//use diesel::sql_types::*;
use serde::{ Serialize, Deserialize };
use crate::api::schema::{ 
    tokens, 
    pairs, 
    sandwiches, 
    frontrun_transactions, 
    lunchmeat_transactions, 
    backrun_transactions };

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "tokens"]
pub struct Token {
    pub token_id: i32,
    pub token_name: String,
    pub token_symbol: String,
    pub decimals: i16,
    pub blockchain_name: String,
    pub token_address: String
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "pairs"]
pub struct Pair {
    pub pair_id: i32,
    pub blockchain_name: String,
    pub exchange_name: String,
    pub pair_address: String,
    pub base_token_id: i32,
    pub quote_token_id: i32,
    pub latest_scanned_block: Option<i64>,
    pub earliest_scanned_block: Option<i64>,
    pub scanning_latest: bool,
    pub scanning_previous: bool
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "sandwiches"]
pub struct Sandwich {
    sandwich_id: i64,
    pair_id: i32,
    block_number: i64
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "frontrun_transactions"]
pub struct FrontrunTransaction {
    frontrun_id: i64,
    tx_hash: String,
    tx_index: i32,
    sandwich_id: i64
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "lunchmeat_transactions"]
pub struct LunchmeatTransaction {
    lunchmeat_id: i64,
    tx_hash: String,
    tx_index: i32,
    sandwich_id: i64
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "backrun_transactions"]
pub struct BackrunTransaction {
    backrun_id: i64,
    tx_hash: String,
    tx_index: i32,
    sandwich_id: i64
}