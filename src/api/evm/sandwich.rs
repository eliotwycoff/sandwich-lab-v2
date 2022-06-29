use ethers::prelude::{ Provider, Middleware, Http };
use super::swap::Swap;
use std::fmt;
use tokio;

// This helper macro spawns a new tokio task to fetch
// a transaction for the given transaction hash,
// and it returns a handle to the future.
#[macro_export]
macro_rules! get_transaction_handle {
    ($provider_url:expr, $tx_hash:expr) => {
        tokio::spawn(async move {
            match Provider::<Http>::try_from($provider_url) {
                Ok(provider) => {
                    match provider.get_transaction($tx_hash).await {
                        Ok(Some(transaction)) => Ok(transaction),
                        Ok(None) => Err(SandwichError::NoTransaction),
                        Err(_) => Err(SandwichError::ProviderError)
                    }
                },
                Err(_) => Err(SandwichError::ParseError)
            }
        })
    }
}

// This helper macro spawns a new tokio task to fetch
// a transaction receipt for the given transaction hash,
// and it returns a handle to the future.
#[macro_export]
macro_rules! get_receipt_handle {
    ($provider_url:expr, $tx_hash:expr) => {
        tokio::spawn(async move {
            match Provider::<Http>::try_from($provider_url) {
                Ok(provider) => {
                    match provider.get_transaction_receipt($tx_hash).await {
                        Ok(Some(receipt)) => Ok(receipt),
                        Ok(None) => Err(SandwichError::NoReceipt),
                        Err(_) => Err(SandwichError::ProviderError)
                    }
                },
                Err(_) => Err(SandwichError::ParseError)
            }
        })
    }
}

pub struct Sandwich<'a> {
    pub frontrun: Swap<'a>,
    pub lunchmeat: Vec<Swap<'a>>,
    pub backrun: Swap<'a>
}

impl<'a> Sandwich<'a> {
    // Add transaction metadata (e.g. gas information)
    // to each swap in this sandwich.
    pub async fn add_tx_meta(
        &mut self,
        provider_url: &str
    ) -> Result<(), SandwichError> {
        // Create vecs to hold all the transaction and receipt handles.
        let mut tx_handles = Vec::with_capacity(self.lunchmeat.len() + 2);
        let mut receipt_handles = Vec::with_capacity(self.lunchmeat.len() + 2);

        // Get the tx and receipt handles for the frontrun transaction.
        let provider_url_copy = provider_url.to_string();
        let tx_hash = self.frontrun.swap.tx_hash.clone();
        tx_handles.push(get_transaction_handle!(provider_url_copy, tx_hash));

        let provider_url_copy = provider_url.to_string();
        let tx_hash = self.frontrun.swap.tx_hash.clone();
        receipt_handles.push(get_receipt_handle!(provider_url_copy, tx_hash));

        // Get the tx and receipt handles for all the lunchmeat transactions.
        for i in 0..self.lunchmeat.len() {
            let provider_url_copy = provider_url.to_string();
            let tx_hash = self.lunchmeat[i].swap.tx_hash.clone();
            tx_handles.push(get_transaction_handle!(provider_url_copy, tx_hash));

            let provider_url_copy = provider_url.to_string();
            let tx_hash = self.lunchmeat[i].swap.tx_hash.clone();
            receipt_handles.push(get_receipt_handle!(provider_url_copy, tx_hash));
        }

        // Get the tx and receipt handle for the backrun transaction.
        let provider_url_copy = provider_url.to_string();
        let tx_hash = self.backrun.swap.tx_hash.clone();
        tx_handles.push(get_transaction_handle!(provider_url_copy, tx_hash));

        let provider_url_copy = provider_url.to_string();
        let tx_hash = self.backrun.swap.tx_hash.clone();
        receipt_handles.push(get_receipt_handle!(provider_url_copy, tx_hash));

        // Create a vec to hold all the transactions and receipts.
        let mut transactions = Vec::with_capacity(tx_handles.len());
        let mut receipts = Vec::with_capacity(receipt_handles.len());

        for tx_handle in tx_handles {
            transactions.push(tx_handle.await??);
        }

        for receipt_handle in receipt_handles {
            receipts.push(receipt_handle.await??);
        }

        // Clone the transactions back into the Swap structs.
        for i in 0..transactions.len() {
            if i == 0 {
                self.frontrun.add_transaction_meta(transactions[i].clone());
            } else if i < transactions.len() - 1 {
                self.lunchmeat[i-1].add_transaction_meta(transactions[i].clone());
            } else {
                self.backrun.add_transaction_meta(transactions[i].clone());
            }
        }

        // Clone the receipts back into the Swap structs.
        for i in 0..receipts.len() {
            if i == 0 {
                self.frontrun.add_receipt_meta(receipts[i].clone());
            } else if i < receipts.len() - 1 {
                self.lunchmeat[i-1].add_receipt_meta(receipts[i].clone());
            } else {
                self.backrun.add_receipt_meta(receipts[i].clone());
            }
        }

        // Let the calling function know that everything worked.
        Ok(())
    }
}

// Pull sandwich data out from the given bundle of swaps.
pub async fn parse_sandwiches<'a>(
    bundle: &'a Vec<Swap<'a>>,
    provider_url: &str
) -> Result<Vec<Sandwich<'a>>, SandwichError> {
    let mut sandwiches = vec![]; // we don't know how big this vec will be
    let num_swaps = bundle.len();

    let mut i = 0;

    while i < num_swaps - 2 { // stop looping if not enough swaps are left to create a sandwich
        let mut j = i + 2; // a swap starting at index `i` needs at least two more swaps
        while j < num_swaps {
            let frontrun = &bundle[i];
            let backrun = &bundle[j];

            if is_match(frontrun, backrun) { // if the swaps match, create the sandwich
                let mut lunchmeat = vec![];

                for k in i+1..j {
                    lunchmeat.push(bundle[k].clone());
                }

                let mut sandwich = Sandwich {
                    frontrun: frontrun.clone(),
                    lunchmeat,
                    backrun: backrun.clone()
                };

                sandwich.add_tx_meta(provider_url).await?;

                sandwiches.push(sandwich);

                i = j + 1;
                j = i + 2;
            } else { // update the indices
                j += 1;
                i = if j >= num_swaps { i + 1 } else { i };
            }
        }
    }

    Ok(sandwiches)
}

// Given two swaps, determine if they match as a frontrun and backrun pair.
fn is_match(a: &Swap, b: &Swap) -> bool {
    if a.base.token_id != b.base.token_id {
        return false;
    }

    if a.quote.token_id != b.quote.token_id {
        return false;
    }

    let tol = 1.005;

    let base_ratio = a.swap.in0(a.base.decimals as u8) / b.swap.out0(b.base.decimals as u8);
    let quote_ratio = a.swap.in1(a.quote.decimals as u8) / b.swap.out1(b.quote.decimals as u8);

    if 1.0/tol < base_ratio && base_ratio < tol {
        return true;
    }

    if 1.0/tol < quote_ratio && quote_ratio < tol {
        return true;
    }

    false
}

pub enum SandwichError {
    NoTransaction,
    NoReceipt,
    ProviderError,
    ParseError,
    JoinError
}

impl SandwichError {
    fn message(&self) -> &str {
        match self {
            Self::NoTransaction => "no transaction was returned from the provider",
            Self::NoReceipt => "no receipt was returned from the provider",
            Self::ProviderError => "the provider returned with an error",
            Self::ParseError => "could not parse the provider url",
            Self::JoinError => "join error"
        }
    }
}

impl fmt::Display for SandwichError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl fmt::Debug for SandwichError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for SandwichError {}

impl From<tokio::task::JoinError> for SandwichError {
    fn from(_: tokio::task::JoinError) -> Self {
        Self::JoinError
    }
}