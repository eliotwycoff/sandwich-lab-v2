use diesel::*;
use serde::{ Serialize, Deserialize };
use crate::api::schema::{ 
    tokens, 
    pairs, 
    ranges,
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
    pub blockchain_str_id: String,
    pub token_address: String
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "pairs"]
pub struct Pair {
    pub pair_id: i32,
    pub blockchain_str_id: String,
    pub factory_address: String,
    pub pair_address: String,
    pub base_token_id: i32,
    pub quote_token_id: i32
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "ranges"]
pub struct Range {
    pub range_id: i64,
    pub pair_id: i32,
    pub lower_bound: i64,
    pub upper_bound: i64,
    pub scan_complete: bool,
    pub scan_failed: bool
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "sandwiches"]
pub struct Sandwich {
    pub sandwich_id: i64,
    pair_id: i32,
    pub block_number: i64
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "frontrun_transactions"]
pub struct FrontrunTransaction {
    frontrun_id: i64,
    pub tx_hash: String,
    pub tx_index: i32,
    pub base_in: f64,
    pub quote_in: f64,
    pub base_out: f64,
    pub quote_out: f64,
    pub gas: f64,
    sandwich_id: i64
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "lunchmeat_transactions"]
pub struct LunchmeatTransaction {
    lunchmeat_id: i64,
    pub tx_hash: String,
    pub tx_index: i32,
    pub base_in: f64,
    pub quote_in: f64,
    pub base_out: f64,
    pub quote_out: f64,
    pub gas: f64,
    sandwich_id: i64
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "backrun_transactions"]
pub struct BackrunTransaction {
    backrun_id: i64,
    pub tx_hash: String,
    pub tx_index: i32,
    pub base_in: f64,
    pub quote_in: f64,
    pub base_out: f64,
    pub quote_out: f64,
    pub gas: f64,
    sandwich_id: i64
}