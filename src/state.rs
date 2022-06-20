use actix_web::web::Data;
use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use crate::api::db;

// A thread-safe instance of this AppState is used
// to hold global variables and database connections.
pub struct AppState {
    pub app_name: Mutex<String>,
    pub db_pool: db::Pool,
    pub blockchains: HashMap<String, Blockchain> // blockchain name (key) -> blockchain struct (value)
}

// Each blockchain that this application interacts with
// will have helper data stored in a Blockchain instance.
pub struct Blockchain {
    pub name: String,
    pub provider_url: String,
    pub data_aggregator_address: String,
    pub exchanges: HashMap<String, Exchange> // factory address (key) -> exchange enum (value)
}

// Each exchange that this application interacts with
// will have helper data stored in an Exchange instance.
pub enum Exchange {
    V2 { name: String },
    V3 { name: String }
}

impl Exchange {
    pub fn name(&self) -> &str {
        match self {
            Exchange::V2 { name } => name,
            Exchange::V3 { name } => name
        }
    }
}

// This function should be called on server startup
// to initialize the application's global, shared state.
pub fn init_app_state() -> Data<AppState> {
    Data::new(AppState {
        app_name: Mutex::new(String::from("Sandwich Lab")),
        db_pool: db::init_db_pool(),
        blockchains: init_blockchains() // this function is defined below
    })
}

// For now, hardcode which blockchains and exchanges are supported.
// (Consider pushing this data into the database in the future.)
fn init_blockchains() -> HashMap<String, Blockchain> {
    HashMap::from([ 
        (
            "ethereum".to_string(), 
            Blockchain { 
                name: "Ethereum".to_string(), 
                provider_url: env::var("ETHEREUM_URL")
                    .expect("error reading provider url"),
                data_aggregator_address: env::var("ETHEREUM_DATA_AGGREGATOR")
                    .expect("error reading data aggregator address"),
                exchanges: HashMap::from([
                    (
                        "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".to_lowercase(),
                        Exchange::V2 {
                            name: "Uniswap V2".to_string()
                        }
                    ),
                    (
                        "0x1F98431c8aD98523631AE4a59f267346ea31F984".to_lowercase(),
                        Exchange::V3 {
                            name: "Uniswap V3".to_string()
                        }
                    ),
                    (
                        "0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac".to_lowercase(),
                        Exchange::V2 {
                            name: "SushiSwap V2".to_string()
                        }
                    )
                ])
            },
        ),
        (
            "moonriver".to_string(),
            Blockchain {
                name: "Moonriver".to_string(),
                provider_url: env::var("MOONRIVER_URL")
                    .expect("error reading provider url"),
                data_aggregator_address: env::var("MOONRIVER_DATA_AGGREGATOR")
                    .expect("error reading data aggregator address"),
                exchanges: HashMap::from([
                    (
                        "0x049581aEB6Fe262727f290165C29BDAB065a1B68".to_lowercase(),
                        Exchange::V2 {
                            name: "Solarbeam".to_string()
                        }
                    )
                ])
            }
        )
    ])
}