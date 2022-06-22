table! {
    backrun_transactions (backrun_id) {
        backrun_id -> Int8,
        tx_hash -> Bpchar,
        tx_index -> Int4,
        base_in -> Float8,
        quote_in -> Float8,
        base_out -> Float8,
        quote_out -> Float8,
        gas -> Float8,
        sandwich_id -> Int8,
    }
}

table! {
    frontrun_transactions (frontrun_id) {
        frontrun_id -> Int8,
        tx_hash -> Bpchar,
        tx_index -> Int4,
        base_in -> Float8,
        quote_in -> Float8,
        base_out -> Float8,
        quote_out -> Float8,
        gas -> Float8,
        sandwich_id -> Int8,
    }
}

table! {
    lunchmeat_transactions (lunchmeat_id) {
        lunchmeat_id -> Int8,
        tx_hash -> Bpchar,
        tx_index -> Int4,
        base_in -> Float8,
        quote_in -> Float8,
        base_out -> Float8,
        quote_out -> Float8,
        gas -> Float8,
        sandwich_id -> Int8,
    }
}

table! {
    pairs (pair_id) {
        pair_id -> Int4,
        blockchain_str_id -> Varchar,
        factory_address -> Bpchar,
        pair_address -> Bpchar,
        base_token_id -> Int4,
        quote_token_id -> Int4,
        latest_scanned_block -> Nullable<Int8>,
        earliest_scanned_block -> Nullable<Int8>,
        scanning_latest -> Bool,
        scanning_previous -> Bool,
    }
}

table! {
    sandwiches (sandwich_id) {
        sandwich_id -> Int8,
        pair_id -> Int4,
        block_number -> Int8,
    }
}

table! {
    tokens (token_id) {
        token_id -> Int4,
        token_name -> Varchar,
        token_symbol -> Varchar,
        decimals -> Int2,
        blockchain_str_id -> Varchar,
        token_address -> Bpchar,
    }
}

joinable!(backrun_transactions -> sandwiches (sandwich_id));
joinable!(frontrun_transactions -> sandwiches (sandwich_id));
joinable!(lunchmeat_transactions -> sandwiches (sandwich_id));
joinable!(sandwiches -> pairs (pair_id));

allow_tables_to_appear_in_same_query!(
    backrun_transactions,
    frontrun_transactions,
    lunchmeat_transactions,
    pairs,
    sandwiches,
    tokens,
);
