use ethers::prelude::{ EthEvent, LogMeta };
use ethers::types::{ Address, U64, U128, U256, I256, TxHash, Sign, Transaction, TransactionReceipt };
use ethers::utils::{ hex, format_units };
use ethers::core as ethers_core;
use ethers::contract as ethers_contract;
use super::super::models::Token;
use std::convert::From;

#[derive(Debug, Clone)]
pub struct Swap<'a> {
    pub swap: SwapCore,
    pub transaction: Option<Transaction>,
    pub receipt: Option<TransactionReceipt>,
    pub native_decimals: u8,
    pub base: &'a Token,
    pub quote: &'a Token
}

impl<'a> Swap<'a> {
    pub fn add_transaction_meta(&mut self, transaction: Transaction) {
        self.transaction = Some(transaction);
    }

    pub fn add_receipt_meta(&mut self, receipt: TransactionReceipt) {
        self.receipt = Some(receipt);
    }

    pub fn in0(&self) -> f64 {
        self.swap.in0(self.base.decimals as u8)
    }

    pub fn in1(&self) -> f64 {
        self.swap.in1(self.quote.decimals as u8)
    }

    pub fn out0(&self) -> f64 {
        self.swap.out0(self.base.decimals as u8)
    }

    pub fn out1(&self) -> f64 {
        self.swap.out1(self.quote.decimals as u8)
    }

    pub fn gas(&self) -> f64 {
        let transaction = match &self.transaction {
            Some(transaction) => transaction,
            None => return 0f64
        };

        let gas_price = match transaction.gas_price {
            Some(price) => price,
            None => return 0f64
        };

        let receipt = match &self.receipt {
            Some(receipt) => receipt,
            None => return 0f64
        };

        let gas_used = match receipt.gas_used {
            Some(amount) => amount,
            None => return 0f64
        };

        match gas_price.checked_mul(gas_used) {
            Some(value) => format_units(value, self.native_decimals as u32)
                .unwrap_or("0.0".to_string()).parse::<f64>().unwrap(),
            None => 0f64
        }
    }
}

// Wrap a `SwapCore` struct with token metadata.
pub fn to_wrapped<'a>(
    swap: SwapCore,
    native_decimals: u8,
    base: &'a Token,
    quote: &'a Token
) -> Swap<'a> {
    Swap {
        swap,
        transaction: None,
        receipt: None,
        native_decimals,
        base,
        quote
    }
}

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

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct SwapCore {
    block_number: U64,
    pub tx_hash: TxHash,
    tx_index: U64,
    in0: U256,
    in1: U256,
    out0: U256,
    out1: U256
}

impl SwapCore {
    pub fn block_number(&self) -> u64 {
        self.block_number.as_u64()
    }

    pub fn tx_hash(&self) -> String {
        hex::encode(self.tx_hash)
    }

    pub fn tx_index(&self) -> i32 {
        self.tx_index.as_u32().try_into().unwrap_or(i32::MAX)
    }

    pub fn in0(&self, decimals: u8) -> f64 {
        Self::u256_to_f64(self.in0, decimals)
    }

    pub fn in1(&self, decimals: u8) -> f64 {
        Self::u256_to_f64(self.in1, decimals)
    }

    pub fn out0(&self, decimals: u8) -> f64 {
        Self::u256_to_f64(self.out0, decimals)
    }

    pub fn out1(&self, decimals: u8) -> f64 {
        Self::u256_to_f64(self.out1, decimals)
    }

    fn u256_to_f64(u256: U256, decimals: u8) -> f64 {
        format_units(u256, decimals as u32).unwrap_or("0.0".to_string()).parse::<f64>().unwrap()
    }
}

impl From<(RawSwapV2, LogMeta)> for SwapCore {
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

impl From<(RawSwapV3, LogMeta)> for SwapCore {
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