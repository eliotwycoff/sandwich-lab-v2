const hre = require("hardhat");

// This example script should be run via the following command:
// `npx hardhat run scripts/example.js --network mainnet`
async function main() {
  const dataAggregator = await hre.ethers.getContractAt(
    "DataAggregator",
    "0x9Dd7C0368684C10C4853d6d3383763B0AA0bE1D2") // Ethereum deployment address

  console.log("\nFetching v2 data from USDC/WETH on Ethereum...");

  let pairAddress = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"; // v2 USDC/WETH pair
  let [t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
    .getTokenMetadata(pairAddress);
  
  console.log(`Base Token: ${t0name} (${t0sym}) at ${t0addr} (${t0d} decimals)`);
  console.log(`Quote Token: ${t1name} (${t1sym}) at ${t1addr} (${t1d} decimals)`);

  console.log("\nFetching v3 data from FRAX/USDC on Ethereum...");

  pairAddress = "0xc63b0708e2f7e69cb8a1df0e1389a98c35a76d52"; // v3 FRAX/USDC pair
  [t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
    .getTokenMetadata(pairAddress);
  
  console.log(`Base Token: ${t0name} (${t0sym}) at ${t0addr} (${t0d} decimals)`);
  console.log(`Quote Token: ${t1name} (${t1sym}) at ${t1addr} (${t1d} decimals)`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
});