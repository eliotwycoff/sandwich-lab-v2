-- Your SQL goes here
CREATE TABLE tokens (
    token_id SERIAL PRIMARY KEY,
    token_name VARCHAR (64) NOT NULL,
    token_symbol VARCHAR (16) NOT NULL,
    decimals SMALLINT NOT NULL,
    blockchain_str_id VARCHAR (32) NOT NULL,
    token_address CHAR (42) NOT NULL,
    UNIQUE (blockchain_str_id, token_address)
);

CREATE TABLE pairs (
    pair_id SERIAL PRIMARY KEY,
    blockchain_str_id VARCHAR (32) NOT NULL,
    factory_address CHAR (42) NOT NULL,
    pair_address CHAR (42) NOT NULL,
    base_token_id INT NOT NULL REFERENCES tokens (token_id) ON DELETE CASCADE,
    quote_token_id INT NOT NULL REFERENCES tokens (token_id) ON DELETE CASCADE,
    latest_scanned_block BIGINT,
    earliest_scanned_block BIGINT,
    scanning_latest BOOLEAN NOT NULL DEFAULT 'f',
    scanning_previous BOOLEAN NOT NULL DEFAULT 'f',
    UNIQUE (blockchain_str_id, pair_address)
);

CREATE TABLE sandwiches (
    sandwich_id BIGSERIAL PRIMARY KEY,
    pair_id INT NOT NULL REFERENCES pairs (pair_id) ON DELETE CASCADE,
    block_number BIGINT NOT NULL
);

CREATE INDEX sandwich_pair_id ON sandwiches (pair_id);
CREATE INDEX sandwich_block_number ON sandwiches (block_number);

CREATE TABLE frontrun_transactions (
    frontrun_id BIGSERIAL PRIMARY KEY,
    tx_hash CHAR (66) NOT NULL,
    tx_index INT NOT NULL,
    base_in DOUBLE PRECISION NOT NULL,
    quote_in DOUBLE PRECISION NOT NULL,
    base_out DOUBLE PRECISION NOT NULL,
    quote_out DOUBLE PRECISION NOT NULL,
    gas DOUBLE PRECISION NOT NULL,
    sandwich_id BIGINT NOT NULL REFERENCES sandwiches (sandwich_id) ON DELETE CASCADE,
    UNIQUE (sandwich_id)
);

CREATE TABLE lunchmeat_transactions (
    lunchmeat_id BIGSERIAL PRIMARY KEY,
    tx_hash CHAR (66) NOT NULL,
    tx_index INT NOT NULL,
    base_in DOUBLE PRECISION NOT NULL,
    quote_in DOUBLE PRECISION NOT NULL,
    base_out DOUBLE PRECISION NOT NULL,
    quote_out DOUBLE PRECISION NOT NULL,
    gas DOUBLE PRECISION NOT NULL,
    sandwich_id BIGINT NOT NULL REFERENCES sandwiches (sandwich_id) ON DELETE CASCADE
);

CREATE INDEX lunchmeat_sandwich_id ON lunchmeat_transactions (sandwich_id);

CREATE TABLE backrun_transactions (
    backrun_id BIGSERIAL PRIMARY KEY,
    tx_hash CHAR (66) NOT NULL,
    tx_index INT NOT NULL,
    base_in DOUBLE PRECISION NOT NULL,
    quote_in DOUBLE PRECISION NOT NULL,
    base_out DOUBLE PRECISION NOT NULL,
    quote_out DOUBLE PRECISION NOT NULL,
    gas DOUBLE PRECISION NOT NULL,
    sandwich_id BIGINT NOT NULL REFERENCES sandwiches (sandwich_id) ON DELETE CASCADE,
    UNIQUE (sandwich_id)
);