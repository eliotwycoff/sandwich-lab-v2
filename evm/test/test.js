const { expect } = require("chai");
const { ethers } = require("hardhat");

// These tests should be run via the following command:
// `npx hardhat test --network mainnet`
describe("DataAggregator", function () {
  it("should return the correct Uniswap v2 metadata from Ethereum", async function () {
    const dataAggregator = await hre.ethers.getContractAt(
      "DataAggregator",
      "0x9Dd7C0368684C10C4853d6d3383763B0AA0bE1D2") // Ethereum deployment address
  
    const pairAddress = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"; // v2 USDC/WETH pair
    const [t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
      .getTokenMetadata(pairAddress);

    expect(t0addr).to.equal("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    expect(t0name).to.equal("USD Coin");
    expect(t0sym).to.equal("USDC");
    expect(t0d).to.equal(6);
    expect(t1addr).to.equal("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    expect(t1name).to.equal("Wrapped Ether");
    expect(t1sym).to.equal("WETH");
    expect(t1d).to.equal(18);
  });

  it("should return the correct Uniswap v3 metadata from Ethereum", async function () {
    const dataAggregator = await hre.ethers.getContractAt(
      "DataAggregator",
      "0x9Dd7C0368684C10C4853d6d3383763B0AA0bE1D2") // Ethereum deployment address
  
    const pairAddress = "0xc63b0708e2f7e69cb8a1df0e1389a98c35a76d52"; // v3 FRAX/USDC pair
    const [t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
      .getTokenMetadata(pairAddress);

    expect(t0addr).to.equal("0x853d955aCEf822Db058eb8505911ED77F175b99e");
    expect(t0name).to.equal("Frax");
    expect(t0sym).to.equal("FRAX");
    expect(t0d).to.equal(18);
    expect(t1addr).to.equal("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    expect(t1name).to.equal("USD Coin");
    expect(t1sym).to.equal("USDC");
    expect(t1d).to.equal(6);
  });
});
