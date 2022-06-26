// This helper macro creates a JSON response formatted as an error.
#[macro_export]
macro_rules! response_error {
    ($message:expr, $response:ident) => {
        web::Json($response::as_error($message.to_string()))
    }
}

// This helper macro gets a database connection from the database pool,
// returning the appropriate error if a connection cannot be established.
#[macro_export]
macro_rules! get_db_connection {
    ($data:expr, $response:ident) => {
        match $data.db_pool.get() {
            Ok(db_connection) => db_connection,
            Err(_) => return response_error!("cannot connect to database", $response)
        }
    }
}

// This helper macro tries to cast the passed value as an i64 
// and returns the appropriate error if an overflow occurs.
#[macro_export]
macro_rules! into_i64 {
    ($val:expr, $response:ident) => {
        match i64::try_from($val) {
            Ok(int) => int,
            Err(_) => return response_error!("numeric overflow", $response)
        }
    }
}

// This helper macro tries to unpack a web::block thread result
// and returns the appropriate error if the result is an error.
#[macro_export]
macro_rules! thread_unwrap {
    ($thread_result:expr, $response:ident) => {
        match $thread_result {
            Ok(result) => result,
            Err(_) => return response_error!("thread error", $response)
        }
    }
}

// This helper macro implements the From trait for the various
// `Transaction` db structs into `sandwiches::TransactionData`
#[macro_export]
macro_rules! implement_transaction_data_from {
    ($from:ident) => {
        impl std::convert::From<&$from> for super::sandwiches::TransactionData {
            fn from(tx: &$from) -> Self {
                Self {
                    hash: tx.tx_hash.clone(),
                    index: tx.tx_index as usize,
                    base_in: tx.base_in,
                    quote_in: tx.quote_in,
                    base_out: tx.base_out,
                    quote_out: tx.quote_out,
                    gas: tx.gas
                }
            }
        }
    }
}