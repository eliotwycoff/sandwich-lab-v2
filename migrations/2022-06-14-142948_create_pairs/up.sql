-- Your SQL goes here
CREATE TABLE tokens (
    token_id SERIAL PRIMARY KEY,
    token_name VARCHAR (64) NOT NULL,
    token_symbol VARCHAR (16) NOT NULL,
    decimals SMALLINT NOT NULL CHECK (decimals >= 0),
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
    UNIQUE (blockchain_str_id, pair_address)
);

CREATE TABLE ranges (
    range_id BIGSERIAL PRIMARY KEY,
    pair_id INT NOT NULL REFERENCES pairs (pair_id) ON DELETE CASCADE,
    lower_bound BIGINT NOT NULL CHECK (lower_bound >= 0),
    upper_bound BIGINT NOT NULL CHECK (upper_bound >= 0),
    scan_complete BOOLEAN NOT NULL DEFAULT 'f',
    scan_failed BOOLEAN NOT NULL DEFAULT 'f'
);

CREATE INDEX range_pair_id ON ranges (pair_id);
CREATE INDEX range_lower_bound ON ranges (lower_bound);
CREATE INDEX range_upper_bound ON ranges (upper_bound);
CREATE INDEX range_complete ON ranges (scan_complete);

CREATE TABLE sandwiches (
    sandwich_id BIGSERIAL PRIMARY KEY,
    pair_id INT NOT NULL REFERENCES pairs (pair_id) ON DELETE CASCADE,
    block_number BIGINT NOT NULL CHECK (block_number >= 0)
);

CREATE INDEX sandwich_pair_id ON sandwiches (pair_id);
CREATE INDEX sandwich_block_number ON sandwiches (block_number);

CREATE TABLE frontrun_transactions (
    frontrun_id BIGSERIAL PRIMARY KEY,
    tx_hash CHAR (66) NOT NULL,
    tx_index INT NOT NULL CHECK (tx_index >= 0),
    base_in DOUBLE PRECISION NOT NULL CHECK (base_in >= 0),
    quote_in DOUBLE PRECISION NOT NULL CHECK (quote_in >= 0),
    base_out DOUBLE PRECISION NOT NULL CHECK (base_out >= 0),
    quote_out DOUBLE PRECISION NOT NULL CHECK (quote_out >= 0),
    gas DOUBLE PRECISION NOT NULL CHECK (gas >= 0),
    sandwich_id BIGINT NOT NULL REFERENCES sandwiches (sandwich_id) ON DELETE CASCADE,
    UNIQUE (sandwich_id)
);

CREATE TABLE lunchmeat_transactions (
    lunchmeat_id BIGSERIAL PRIMARY KEY,
    tx_hash CHAR (66) NOT NULL,
    tx_index INT NOT NULL CHECK (tx_index >= 0),
    base_in DOUBLE PRECISION NOT NULL CHECK (base_in >= 0),
    quote_in DOUBLE PRECISION NOT NULL CHECK (quote_in >= 0),
    base_out DOUBLE PRECISION NOT NULL CHECK (base_out >= 0),
    quote_out DOUBLE PRECISION NOT NULL CHECK (quote_out >= 0),
    gas DOUBLE PRECISION NOT NULL CHECK (gas >= 0),
    sandwich_id BIGINT NOT NULL REFERENCES sandwiches (sandwich_id) ON DELETE CASCADE
);

CREATE INDEX lunchmeat_sandwich_id ON lunchmeat_transactions (sandwich_id);

CREATE TABLE backrun_transactions (
    backrun_id BIGSERIAL PRIMARY KEY,
    tx_hash CHAR (66) NOT NULL,
    tx_index INT NOT NULL CHECK (tx_index >= 0),
    base_in DOUBLE PRECISION NOT NULL CHECK (base_in >= 0),
    quote_in DOUBLE PRECISION NOT NULL CHECK (quote_in >= 0),
    base_out DOUBLE PRECISION NOT NULL CHECK (base_out >= 0),
    quote_out DOUBLE PRECISION NOT NULL CHECK (quote_out >= 0),
    gas DOUBLE PRECISION NOT NULL CHECK (gas >= 0),
    sandwich_id BIGINT NOT NULL REFERENCES sandwiches (sandwich_id) ON DELETE CASCADE,
    UNIQUE (sandwich_id)
);