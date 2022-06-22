use ethers::prelude::{ Provider, Http, Middleware, Contract };
use ethers::abi::AbiParser;
use ethers::types::Address;
use ethers::utils::hex;
use std::sync::Arc;
use std::convert::From;

type RpcError = Box<dyn std::error::Error + Send + Sync>;
type Metadata = (Address, Address, String, String, u8, Address, String, String, u8);

#[derive(Debug, Clone)]
pub struct PairMetadata {
    pub factory_address: String,
    pub base_address: String,
    pub quote_address: String,
    pub base_name: String,
    pub quote_name: String,
    pub base_symbol: String,
    pub quote_symbol: String,
    pub base_decimals: u8,
    pub quote_decimals: u8
}

impl From<Metadata> for PairMetadata {
    fn from(metadata: Metadata) -> Self {
        Self {
            factory_address: hex::encode(metadata.0).to_lowercase(),
            base_address: hex::encode(metadata.1).to_lowercase(),
            quote_address: hex::encode(metadata.5).to_lowercase(),
            base_name: metadata.2,
            quote_name: metadata.6,
            base_symbol: metadata.3,
            quote_symbol: metadata.7,
            base_decimals: metadata.4,
            quote_decimals: metadata.8
        }
    }
}

// Fetches the latest block number from the given provider.
pub async fn fetch_latest_block_number(
    provider_url: &str
) -> Result<u64, RpcError> {
    let provider = Provider::<Http>::try_from(provider_url)?;
    Ok(provider.get_block_number().await?.as_u64())
}

// Fetches the pair (and base and quote token) metadata
// from the blockchain via the DataAggregator contract.
pub async fn fetch_pair_metadata(
    provider_url: &str, 
    pair_address: &str,
    data_aggregator_address: &str
) -> Result<PairMetadata, RpcError> {

    let data_aggregator_abi = AbiParser::default().parse_str(r#"[
        function getMetadata(address) external view returns (address, address, string, string, uint8, address, string, string, uint8)
    ]"#).unwrap();

    let provider = Provider::<Http>::try_from(provider_url)?;
    let client = Arc::new(provider.clone());
    let contract = Contract::<Provider<Http>>::new(
        data_aggregator_address.parse::<Address>()?, 
        data_aggregator_abi.clone(), 
        Arc::clone(&client));

    let metadata = contract
        .method::<_, Metadata>("getMetadata", pair_address.parse::<Address>()?)?
        .call()
        .await?;

    Ok(PairMetadata::from(metadata))
}