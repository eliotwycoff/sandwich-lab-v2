const { expect } = require("chai");
const { ethers } = require("hardhat");

// These tests should be run via the following command:
// `npx hardhat test --network mainnet`
describe("DataAggregator", function () {
  it("should return the correct Uniswap v2 metadata from Ethereum", async function () {
    const dataAggregator = await hre.ethers.getContractAt(
      "DataAggregator",
      "0x34954249EF141B0E7ed365c5c3551b09fEE4E279") // Ethereum deployment address
  
    const pairAddress = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"; // v2 USDC/WETH pair
    const [facAddr, t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
      .getMetadata(pairAddress);

    expect(facAddr.toLowerCase()).to.equal(
      "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".toLowerCase());
    expect(t0addr.toLowerCase()).to.equal(
      "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".toLowerCase());
    expect(t0name).to.equal("USD Coin");
    expect(t0sym).to.equal("USDC");
    expect(t0d).to.equal(6);
    expect(t1addr.toLowerCase()).to.equal(
      "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".toLowerCase());
    expect(t1name).to.equal("Wrapped Ether");
    expect(t1sym).to.equal("WETH");
    expect(t1d).to.equal(18);
  });

  it("should return the correct Sushiswap metadata from Ethereum", async function () {
    const dataAggregator = await hre.ethers.getContractAt(
      "DataAggregator",
      "0x34954249EF141B0E7ed365c5c3551b09fEE4E279") // Ethereum deployment address
  
    const pairAddress = "0xA1d7b2d891e3A1f9ef4bBC5be20630C2FEB1c470"; // Sushi SNX/WETH pair
    const [facAddr, t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
      .getMetadata(pairAddress);

    expect(facAddr.toLowerCase()).to.equal(
      "0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac".toLowerCase());
    expect(t0addr.toLowerCase()).to.equal(
      "0xc011a73ee8576fb46f5e1c5751ca3b9fe0af2a6f".toLowerCase());
    expect(t0name).to.equal("Synthetix Network Token");
    expect(t0sym).to.equal("SNX");
    expect(t0d).to.equal(18);
    expect(t1addr.toLowerCase()).to.equal(
      "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".toLowerCase());
    expect(t1name).to.equal("Wrapped Ether");
    expect(t1sym).to.equal("WETH");
    expect(t1d).to.equal(18);
  });

  it("should return the correct Uniswap v3 metadata from Ethereum", async function () {
    const dataAggregator = await hre.ethers.getContractAt(
      "DataAggregator",
      "0x34954249EF141B0E7ed365c5c3551b09fEE4E279") // Ethereum deployment address
  
    const pairAddress = "0xc63b0708e2f7e69cb8a1df0e1389a98c35a76d52"; // v3 FRAX/USDC pair
    const [facAddr, t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
      .getMetadata(pairAddress);

    expect(facAddr.toLowerCase()).to.equal(
      "0x1F98431c8aD98523631AE4a59f267346ea31F984".toLowerCase());
    expect(t0addr.toLowerCase()).to.equal(
      "0x853d955aCEf822Db058eb8505911ED77F175b99e".toLowerCase());
    expect(t0name).to.equal("Frax");
    expect(t0sym).to.equal("FRAX");
    expect(t0d).to.equal(18);
    expect(t1addr.toLowerCase()).to.equal(
      "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".toLowerCase());
    expect(t1name).to.equal("USD Coin");
    expect(t1sym).to.equal("USDC");
    expect(t1d).to.equal(6);
  });

  it("should return the correct Uniswap v2 metadata from Ethereum, \
  even when one token (MKR) returns bytes32 instead of string", async function () {
    const dataAggregator = await hre.ethers.getContractAt(
      "DataAggregator",
      "0x34954249EF141B0E7ed365c5c3551b09fEE4E279") // Ethereum deployment address
  
    const pairAddress = "0xC2aDdA861F89bBB333c90c492cB837741916A225"; // v2 MKR/WETH pair
    const [facAddr, t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
      .getMetadata(pairAddress);

    expect(facAddr.toLowerCase()).to.equal(
      "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".toLowerCase());
    expect(t0addr.toLowerCase()).to.equal(
      "0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2".toLowerCase());
    expect(t0name).to.equal("Maker");
    expect(t0sym).to.equal("MKR");
    expect(t0d).to.equal(18);
    expect(t1addr.toLowerCase()).to.equal(
      "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".toLowerCase());
    expect(t1name).to.equal("Wrapped Ether");
    expect(t1sym).to.equal("WETH");
    expect(t1d).to.equal(18);
  });
}); 
