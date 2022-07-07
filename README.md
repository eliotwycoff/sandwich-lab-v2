# Sandwich Lab (V2)
Sandwich Lab is a web app built in Rust for exploring historical sandwich trades. It was built in two weeks as a final project for the Chainshot Ethereum Developer Bootcamp.

## Setup
To run this app locally, you will need to create a .env file in the main directory with a link to an RPC provider on Ethereum (Alchemy recommended), e.g.
`ETHEREUM_URL={your_provider_url}`

You will also need the following environment variable:
`ETHEREUM_DATA_AGGREGATOR=0x34954249EF141B0E7ed365c5c3551b09fEE4E279`

This is the Ethereum deployment address of Sandwich Lab's DataAggregator smart contract.

You will also need PostgreSQL installed and an environment variable to the database, e.g.
`DATABASE_URL=postgres://{user}:{password}@localhost/{database_name}`

Finally, you will need a socket address for the app to bind to, e.g.:
`SOCKET=127.0.0.1:8000`

Once your .env is setup, you just need to run `cargo run`.

## Live Demo
Alternatively, you can just play with the live demo currently hosted at http://sandwich-lab.io:8000/. Please note that I have not installed an SSL certificate, so ignore any warnings about the site being unsafe.

## Sample LP Pair Addresses
Here are some Ethereum pair addresses on Uniswap (V2 and V3) that should turn up a good number of sandwich trades:

WBTC-WETH: 0xcbcdf9626bc03e24f779434178a73a0b4bad62ed
APE-WETH: 0xac4b3dacb91461209ae9d41ec517c2b9cb1b7daf
FXS-FRAX: 0xe1573b9d29e2183b1af0e743dc2754979a40d237
UNI-WETH: 0xd3d2e2692501a5c9ca623199d38826e513033a17
ELON-WETH: 0x7b73644935b8e68019ac6356c40661e1bc315860

You can also pull pair addresses from the analytics pages of Uniswap (V2 and V3) and Sushiswap. Note that pairs with little trading activity are unlikely to have any recent sandwich trades.
