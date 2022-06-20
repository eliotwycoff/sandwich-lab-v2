const hre = require("hardhat");

// This example script should be run via the following command:
// `npx hardhat run scripts/example.js --network mainnet`
async function main() {
  const dataAggregator = await hre.ethers.getContractAt(
    "DataAggregator",
    //"0x9Dd7C0368684C10C4853d6d3383763B0AA0bE1D2") // Ethereum deployment address
    "0x6501582bD29642da03De44a61375D3C2cDf62075"); // Rinkeby deployment address

  const factory = new ethers.Contract("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f",
    ["function allPairs(uint256) public view returns (address)"], ethers.provider);

  for (let i = 183; i < 1000; i++) {
    console.log(`\nIndex: ${i}\n`);

    const pairAddress = await factory.allPairs(i);
    let [facAddr, t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
      .getMetadata(pairAddress);

    console.log(`Pair Address: ${pairAddress}`);
    console.log(`Base Token: ${t0name} (${t0sym}) at ${t0addr} (${t0d} decimals)`);
    console.log(`Quote Token: ${t1name} (${t1sym}) at ${t1addr} (${t1d} decimals)`);
  }



  /*console.log("\nFetching v2 data from USDC/WETH on Ethereum...");

  //let pairAddress = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"; // v2 USDC/WETH pair on Ethereum
  let pairAddress = "0x17a41f752110b036D8d6314804a4f8601cdccd0e";
  let [facAddr, t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
    .getPairMetadata(pairAddress);
  
  console.log(`Factory Address: ${facAddr}`);
  console.log(`Base Token: ${t0name} (${t0sym}) at ${t0addr} (${t0d} decimals)`);
  console.log(`Quote Token: ${t1name} (${t1sym}) at ${t1addr} (${t1d} decimals)`);

  console.log("\nFetching v3 data from FRAX/USDC on Ethereum...");

  //pairAddress = "0xc63b0708e2f7e69cb8a1df0e1389a98c35a76d52"; // v3 FRAX/USDC pair
  pairAddress = "0xF9bA5210F91D0474bd1e1DcDAeC4C58E359AaD85";
  [facAddr, t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
    .getPairMetadata(pairAddress);
  
  console.log(`Factory Address: ${facAddr}`);
  console.log(`Base Token: ${t0name} (${t0sym}) at ${t0addr} (${t0d} decimals)`);
  console.log(`Quote Token: ${t1name} (${t1sym}) at ${t1addr} (${t1d} decimals)`);*/
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
});