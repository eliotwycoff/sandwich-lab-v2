const hre = require("hardhat");

async function main() {
  const DataAggregator = await hre.ethers.getContractFactory("DataAggregator");
  const dataAggregator = await DataAggregator.deploy();

  await dataAggregator.deployed();

  console.log("DataAggregator deployed to:", dataAggregator.address);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
});
