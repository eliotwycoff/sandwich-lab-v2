const hre = require("hardhat");

async function main() {
  const DataUtils = await hre.ethers.getContractFactory("DataUtils");
  const dataUtils = await DataUtils.deploy();
  await dataUtils.deployed();

  console.log(`DataUtils deployed to ${dataUtils.address}`);

  const DataAggregator = await hre.ethers.getContractFactory("DataAggregator");
  const dataAggregator = await DataAggregator.deploy(dataUtils.address);
  await dataAggregator.deployed();

  console.log(`DataAggregator deployed to ${dataAggregator.address}`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
});
