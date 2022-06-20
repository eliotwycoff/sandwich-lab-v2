const hre = require("hardhat");

// This example script should be run via the following command:
// `npx hardhat run scripts/example.js --network mainnet`
async function main() {
  // Ethereum Mainnet Deployment Addresses //
  // DataUtils: 0x2b7F44482BFaeAd1c86dA63aB458E15A2663836A
  // DataAggregator: 0x34954249EF141B0E7ed365c5c3551b09fEE4E279

  const dataAggregator = await hre.ethers.getContractAt(
    "DataAggregator",
    "0x34954249EF141B0E7ed365c5c3551b09fEE4E279") // Ethereum deployment address
    //"0x435817298F34e230261Ca431BAEA3d2377d98b2a"); // Rinkeby deployment address

  const factoryAddress = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";
  const factory = new ethers.Contract(factoryAddress,
    ["function allPairs(uint256) public view returns (address)"], ethers.provider);

  for (let i = 0; i < 10; i++) {
    console.log(`\nIndex: ${i}\n`);

    const pairAddress = await factory.allPairs(i);
    let [facAddr, t0addr, t0name, t0sym, t0d, t1addr, t1name, t1sym, t1d] = await dataAggregator
      .getMetadata(pairAddress);

    console.log(`Pair Address: ${pairAddress}`);
    console.log(`Factory Address: ${facAddr}`);
    console.log(`Base Token: ${t0name} (${t0sym}) at ${t0addr} (${t0d} decimals)`);
    console.log(`Quote Token: ${t1name} (${t1sym}) at ${t1addr} (${t1d} decimals)`);
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
});