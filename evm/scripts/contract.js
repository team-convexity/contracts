
const { ethers } = require("hardhat");

async function main() {
    const [deployer] = await ethers.getSigners();
    console.log("Deploying contracts with the account:", deployer.address);

    // Deploy the base implementation of ProjectContract
    const ProjectContract = await ethers.getContractFactory("ProjectContract");
    const projectImplementation = await ProjectContract.deploy();

    console.log("ProjectContract Implementation Address:", projectImplementation.target);

    // Deploy the factory contract
    const ProjectFactory = await ethers.getContractFactory("ProjectFactory");
    const projectFactory = await ProjectFactory.deploy(projectImplementation.target);

    console.log("ProjectFactory Address:", projectFactory.target);
}


main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
