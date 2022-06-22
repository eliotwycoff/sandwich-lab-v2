use ethers::prelude::{ EthEvent, LogMeta };
use ethers::types::{ Address, U64, U128, U256, I256, TxHash, Sign };
use ethers::utils::{ hex, format_units };
use ethers::core as ethers_core;
use ethers::contract as ethers_contract;
use std::convert::From;

#[derive(Clone, Debug, EthEvent)]
#[ethevent(name = "Swap", abi = "Swap(address,uint,uint,uint,uint,address)")]
pub struct RawSwapV2 {
    #[ethevent(indexed)]
    pub sender: Address,
    #[ethevent(name = "amount0In")]
    pub in0: U256,
    #[ethevent(name = "amount1In")]
    pub in1: U256,
    #[ethevent(name = "amount0Out")]
    pub out0: U256,
    #[ethevent(name = "amount1Out")]
    pub out1: U256,
    #[ethevent(indexed, name = "to")]
    pub recipient: Address
}

#[derive(Clone, Debug, EthEvent)]
#[ethevent(name = "Swap", abi = "Swap(address,address,int256,int256,uint160,uint128,int24)")]
pub struct RawSwapV3 {
    #[ethevent(indexed)]
    pub sender: Address,
    #[ethevent(indexed)]
    pub recipient: Address,
    pub amount0: I256,
    pub amount1: I256,
    #[ethevent(name = "sqrtPriceX96")]
    pub sqrt_price: U256,
    pub liquidity: U128,
    pub tick: I256
}

pub struct Swap {
    block_number: U64,
    pub tx_hash: TxHash,
    tx_index: U64,
    in0: U256,
    in1: U256,
    out0: U256,
    out1: U256
}

impl From<(RawSwapV2, LogMeta)> for Swap {
    fn from((swap, meta): (RawSwapV2, LogMeta)) -> Self {
        Self {
            block_number: meta.block_number,
            tx_hash: meta.transaction_hash,
            tx_index: meta.transaction_index,
            in0: swap.in0,
            in1: swap.in1,
            out0: swap.out0,
            out1: swap.out1
        }
    }
}

impl From<(RawSwapV3, LogMeta)> for Swap {
    fn from((swap, meta): (RawSwapV3, LogMeta)) -> Self {
        let (in0, out0) = match swap.amount0.sign() {
            Sign::Positive => (swap.amount0.into_raw(), U256::zero()),
            Sign::Negative => (U256::zero(), swap.amount0.saturating_neg().into_raw())
        };

        let (in1, out1) = match swap.amount1.sign() {
            Sign::Positive => (swap.amount1.into_raw(), U256::zero()),
            Sign::Negative => (U256::zero(), swap.amount1.saturating_neg().into_raw())
        };

        Self {
            block_number: meta.block_number,
            tx_hash: meta.transaction_hash,
            tx_index: meta.transaction_index,
            in0: in0,
            in1: in1,
            out0: out0,
            out1: out1
        }
    }
}