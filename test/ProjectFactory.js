const { expect } = require("chai");

describe("ProjectFactory", function () {
    let projectImplementation;
    let projectFactory;
    let donor, primary, token, ngo;

    beforeEach(async function () {
        [admin, donor, primary,token, ngo] = await ethers.getSigners();

        // Deploy ProjectContract implementation
        const ProjectContract = await ethers.getContractFactory("ProjectContract");
        projectImplementation = await ProjectContract.deploy();

        // Deploy ProjectFactory
        const ProjectFactory = await ethers.getContractFactory("ProjectFactory");
        projectFactory = await ProjectFactory.deploy(projectImplementation.target);
    });

    it("should deploy a new project clone and verify via deployedProjects", async function () {
        // Create a new project
        const tx = await projectFactory.createProject(
            "Test Project",
            donor.address,
            primary.address,
            ngo.address,
            token.address
        );
        await tx.wait();

        // Get the address of the deployed project
        const deployedProjects = await projectFactory.getDeployedProjects();
        expect(deployedProjects).to.have.lengthOf(1);

        const projectAddress = deployedProjects[0];
        expect(projectAddress).to.properAddress;

        // Verify the state of the deployed project
        const projectClone = await ethers.getContractAt("ProjectContract", projectAddress);

        expect(await projectClone.projectName()).to.equal("Test Project");
        expect(await projectClone.donorWallet()).to.equal(donor.address);
        expect(await projectClone.primaryWallet()).to.equal(primary.address);
        expect(await projectClone.ngo()).to.equal(ngo.address);
    });
});
