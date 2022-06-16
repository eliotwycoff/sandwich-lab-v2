use ethers::prelude::{ Provider, Http, Middleware, Contract, Multicall, LogMeta };
use ethers::abi::{ Abi, AbiParser };
use ethers::types::{ Address };
use ethers::utils::{ hex };
use std::sync::{ Arc };

type RpcError = Box<dyn std::error::Error + Send + Sync>;

pub struct PairMetadata {
    pub base_address: String,
    pub quote_address: String,
    pub base_name: String,
    pub quote_name: String,
    pub base_symbol: String,
    pub quote_symbol: String,
    pub base_decimals: u8,
    pub quote_decimals: u8
}

pub async fn fetch_pair_metadata(
    provider_url: &str, 
    pair_address: &str) -> Result<Option<PairMetadata>, RpcError> {

    let pair_abi = AbiParser::default().parse_str(r#"[
        function token0() public view returns (address)
        function token1() public view returns (address)
    ]"#).unwrap();

    let token_abi = AbiParser::default().parse_str(r#"[
        function name() public view returns (string)
        function symbol() public view returns (string)
        function decimals() public view returns (uint8)
    ]"#).unwrap();

    let provider = Provider::<Http>::try_from(provider_url)?;
    let address = pair_address.parse::<Address>()?;
    let client = Arc::new(provider.clone());
    let contract = Contract::<Provider<Http>>::new(address, pair_abi.clone(), Arc::clone(&client));

    unimplemented!()
}