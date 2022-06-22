require("@nomiclabs/hardhat-waffle");
require("dotenv").config();

module.exports = {
  solidity: {
    compilers: [
      {
        version: "0.8.4"
      }
    ]
  },
  networks: {
    avalanche: {
      url: process.env.AVALANCHE_URL,
      accounts: [process.env.PRIVATE_KEY]
    },
    ethereum: {
      url: process.env.ETHEREUM_URL,
      accounts: [process.env.PRIVATE_KEY]
    },
    goerli: {
      url: process.env.GOERLI_URL,
      accounts: [process.env.PRIVATE_KEY]
    },
    moonriver: {
      url: process.env.MOONRIVER_URL,
      accounts: [process.env.PRIVATE_KEY]
    },
    moonbeam: {
      url: process.env.MOONBEAM_URL,
      accounts: [process.env.PRIVATE_KEY]
    },
    polygon: {
      url: process.env.POLYGON_URL,
      accounts: [process.env.PRIVATE_KEY]
    },
    rinkeby: {
      url: process.env.RINKEBY_URL,
      accounts: [process.env.PRIVATE_KEY]
    }
  }
};