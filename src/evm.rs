use ethers::prelude::{ Provider, Http, Middleware, Contract, Multicall, LogMeta };
use ethers::abi::{ Abi, AbiParser, Detokenize };
use ethers::types::{ Address };
use ethers::utils::{ hex };
use std::sync::{ Arc };
use std::convert::From;

type RpcError = Box<dyn std::error::Error + Send + Sync>;
type Metadata = (Address, String, String, u8, Address, String, String, u8);

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

impl From<Metadata> for PairMetadata {
    fn from(metadata: Metadata) -> Self {
        Self {
            base_address: hex::encode(metadata.0),
            quote_address: hex::encode(metadata.4),
            base_name: metadata.1,
            quote_name: metadata.5,
            base_symbol: metadata.2,
            quote_symbol: metadata.6,
            base_decimals: metadata.3,
            quote_decimals: metadata.7
        }
    }
}

pub async fn fetch_pair_metadata(
    provider_url: &str, 
    pair_address: &str,
    data_aggregator_address: &str
) -> Result<PairMetadata, RpcError> {

    let data_aggregator_abi = AbiParser::default().parse_str(r#"[
        function getTokenMetadata(address) external view returns (address, string, string, uint8, address, string, string, uint8)
    ]"#).unwrap();

    let provider = Provider::<Http>::try_from(provider_url)?;
    let client = Arc::new(provider.clone());
    let contract = Contract::<Provider<Http>>::new(
        data_aggregator_address.parse::<Address>()?, 
        data_aggregator_abi.clone(), 
        Arc::clone(&client));

    let metadata = contract
        .method::<_, Metadata>("getTokenMetadata", pair_address.parse::<Address>()?)?
        .call()
        .await?;

    Ok(PairMetadata::from(metadata))
}