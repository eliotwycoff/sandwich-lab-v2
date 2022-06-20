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
    rinkeby: {
      url: process.env.RINKEBY_URL,
      accounts: [process.env.PRIVATE_KEY]
    },
    mainnet: {
      url: process.env.ETHEREUM_URL,
      accounts: [process.env.PRIVATE_KEY]
    },
    moonriver: {
      url: process.env.MOONRIVER_URL,
      accounts: [process.env.PRIVATE_KEY]
    }
  }
};