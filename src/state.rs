use actix_web::web::Data;
use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use crate::api::db;
use crate::api::evm::scanner::Params;

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
    pub exchanges: HashMap<String, Exchange>, // factory address (key) -> exchange enum (value)
    pub scanner_params: Params,
    pub native_token: NativeToken
}

pub struct NativeToken {
    pub name: String,
    pub symbol: String,
    pub decimals: u8
}

// Each exchange that this application interacts with
// will have helper data stored in an Exchange instance.
#[derive(Debug, Clone)]
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
            "arbitrum".to_string(),
            Blockchain {
                name: "Arbitrum".to_string(),
                provider_url: env::var("ARBITRUM_URL")
                    .expect("error reading provider url"),
                data_aggregator_address: env::var("ARBITRUM_DATA_AGGREGATOR")
                    .expect("error reading data aggregator address"),
                exchanges: HashMap::from([
                    (
                        "0x1F98431c8aD98523631AE4a59f267346ea31F984".to_lowercase(),
                        Exchange::V3 {
                            name: "Uniswap V3".to_string()
                        }
                    )
                ]),
                scanner_params: Params {
                    blocks_per_chunk: 1_000,
                    max_blocks_per_chunk: 10_000,
                    target_swaps_per_chunk: 300,
                    max_blocks_per_request: 100_000
                },
                native_token: NativeToken {
                    name: "Ethereum".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18
                }
            }
        ),
        (
            "avalanche".to_string(),
            Blockchain {
                name: "Avalanche".to_string(),
                provider_url: env::var("AVALANCHE_URL")
                    .expect("error reading provider url"),
                data_aggregator_address: env::var("AVALANCHE_DATA_AGGREGATOR")
                    .expect("error reading data aggregator address"),
                exchanges: HashMap::from([
                    (
                        "0x9Ad6C38BE94206cA50bb0d90783181662f0Cfa10".to_lowercase(),
                        Exchange::V2 {
                            name: "Trader Joe".to_string()
                        }
                    ),
                    (
                        "0xefa94DE7a4656D787667C749f7E1223D71E9FD88".to_lowercase(),
                        Exchange::V2 {
                            name: "Pangolin".to_string()
                        }
                    ),
                    (
                        "0xc35DADB65012eC5796536bD9864eD8773aBc74C4".to_lowercase(),
                        Exchange::V2 {
                            name: "SushiSwap V2".to_string()
                        }
                    )
                ]),
                scanner_params: Params {
                    blocks_per_chunk: 1_000,
                    max_blocks_per_chunk: 10_000,
                    target_swaps_per_chunk: 300,
                    max_blocks_per_request: 100_000
                },
                native_token: NativeToken {
                    name: "Avalanche".to_string(),
                    symbol: "AVAX".to_string(),
                    decimals: 18
                }
            }
        ),
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
                ]),
                scanner_params: Params {
                    blocks_per_chunk: 1_000,
                    max_blocks_per_chunk: 10_000,
                    target_swaps_per_chunk: 300,
                    max_blocks_per_request: 100_000
                },
                native_token: NativeToken {
                    name: "Ethereum".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18
                }
            }
        ),
        (
            "goerli".to_string(),
            Blockchain {
                name: "Goerli".to_string(),
                provider_url: env::var("GOERLI_URL")
                    .expect("error reading provider url"),
                data_aggregator_address: env::var("GOERLI_DATA_AGGREGATOR")
                    .expect("error reading data aggregator address"),
                exchanges: HashMap::from([
                    (
                        "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".to_lowercase(),
                        Exchange::V2 {
                            name: "Uniswap V2".to_string()
                        }
                    )
                ]),
                scanner_params: Params {
                    blocks_per_chunk: 1_000,
                    max_blocks_per_chunk: 10_000,
                    target_swaps_per_chunk: 300,
                    max_blocks_per_request: 100_000
                },
                native_token: NativeToken {
                    name: "Ethereum".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18
                }
            }
        ),
        (
            "moonbeam".to_string(),
            Blockchain {
                name: "Moonbeam".to_string(),
                provider_url: env::var("MOONBEAM_URL")
                    .expect("error reading provider url"),
                data_aggregator_address: env::var("MOONBEAM_DATA_AGGREGATOR")
                    .expect("error reading data aggregator address"),
                exchanges: HashMap::from([
                    (
                        "0x19B85ae92947E0725d5265fFB3389e7E4F191FDa".to_lowercase(),
                        Exchange::V2 {
                            name: "Solarflare".to_string()
                        }
                    ),
                    (
                        "0x68A384D826D3678f78BB9FB1533c7E9577dACc0E".to_lowercase(),
                        Exchange::V2 {
                            name: "StellaSwap".to_string()
                        }
                    ),
                    (
                        "0xc35DADB65012eC5796536bD9864eD8773aBc74C4".to_lowercase(),
                        Exchange::V2 {
                            name: "SushiSwap V2".to_string()
                        }
                    ),
                    (
                        "0x985BcA32293A7A496300a48081947321177a86FD".to_lowercase(),
                        Exchange::V2 {
                            name: "BeamSwap".to_string()
                        }
                    )
                ]),
                scanner_params: Params {
                    blocks_per_chunk: 100,
                    max_blocks_per_chunk: 1_000,
                    target_swaps_per_chunk: 100,
                    max_blocks_per_request: 10_000
                },
                native_token: NativeToken {
                    name: "Glimmer".to_string(),
                    symbol: "GLMR".to_string(),
                    decimals: 18
                }
            }
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
                    ),
                    (
                        "0x017603C8f29F7f6394737628a93c57ffBA1b7256".to_lowercase(),
                        Exchange::V2 {
                            name: "Huckleberry Finance".to_string()
                        }
                    ),
                    (
                        "0xc35DADB65012eC5796536bD9864eD8773aBc74C4".to_lowercase(),
                        Exchange::V2 {
                            name: "SushiSwap V2".to_string()
                        }
                    )
                ]),
                scanner_params: Params {
                    blocks_per_chunk: 100,
                    max_blocks_per_chunk: 1_000,
                    target_swaps_per_chunk: 100,
                    max_blocks_per_request: 10_000
                },
                native_token: NativeToken {
                    name: "Moonriver".to_string(),
                    symbol: "MOVR".to_string(),
                    decimals: 18
                }
            }
        ),
        (
            "optimism".to_string(),
            Blockchain {
                name: "Optimism".to_string(),
                provider_url: env::var("OPTIMISM_URL")
                    .expect("error reading provider url"),
                data_aggregator_address: env::var("OPTIMISM_DATA_AGGREGATOR")
                    .expect("error readign data aggregator address"),
                exchanges: HashMap::from([
                    (
                        "0x1F98431c8aD98523631AE4a59f267346ea31F984".to_lowercase(),
                        Exchange::V3 {
                            name: "Uniswap V3".to_string()
                        }
                    )
                ]),
                scanner_params: Params {
                    blocks_per_chunk: 1_000,
                    max_blocks_per_chunk: 10_000,
                    target_swaps_per_chunk: 300,
                    max_blocks_per_request: 100_000
                },
                native_token: NativeToken {
                    name: "Ethereum".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18
                }
            }
        ),
        (
            "polygon".to_string(),
            Blockchain {
                name: "Polygon".to_string(),
                provider_url: env::var("POLYGON_URL")
                    .expect("error reading provider url"),
                data_aggregator_address: env::var("POLYGON_DATA_AGGREGATOR")
                    .expect("error reading data aggregator address"),
                exchanges: HashMap::from([
                    (
                        "0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32".to_lowercase(),
                        Exchange::V2 {
                            name: "QuickSwap".to_string()
                        }
                    ),
                    (
                        "0xc35DADB65012eC5796536bD9864eD8773aBc74C4".to_lowercase(),
                        Exchange::V2 {
                            name: "SushiSwap V2".to_string()
                        }
                    ),
                    (
                        "0x1F98431c8aD98523631AE4a59f267346ea31F984".to_lowercase(),
                        Exchange::V3 {
                            name: "Uniswap V3".to_string()
                        }
                    )
                ]),
                scanner_params: Params {
                    blocks_per_chunk: 1_000,
                    max_blocks_per_chunk: 10_000,
                    target_swaps_per_chunk: 300,
                    max_blocks_per_request: 100_000
                },
                native_token: NativeToken {
                    name: "Matic".to_string(),
                    symbol: "MATIC".to_string(),
                    decimals: 18
                }
            }
        )
    ])
}