const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("ProjectFactory", function () {
  let ProjectFactory, factory, ProjectContract, projectTemplate;
  let admin, ngo, projectManager, other;

  beforeEach(async function () {
    [admin, ngo, projectManager, other] = await ethers.getSigners();

    // Deploy the ProjectContract (template for deployment)
    ProjectContract = await ethers.getContractFactory("ProjectContract");
    projectTemplate = await ProjectContract.deploy("projectX",admin.address,projectManager.address, ngo.address);


    // Deploy the ProjectFactory
    ProjectFactory = await ethers.getContractFactory("ProjectFactory");
    factory = await ProjectFactory.deploy();

    console.log("Factory Address:", await factory.target);

    // Grant NGO_ROLE to the NGO address
    await factory.connect(admin).grantRole(await factory.NGO_ROLE(), ngo.address);
  });

  describe("Deployment", function () {
    it("should deploy the factory contract", async function () {
      expect(await factory.target).to.properAddress;
    });

    it("should set the admin as the default admin", async function () {
      const isAdmin = await factory.hasRole(await factory.DEFAULT_ADMIN_ROLE(), admin.address);
      expect(isAdmin).to.be.true;
    });
  });

  describe("Project Creation", function () {
    it("should allow NGOs to create projects", async function () {
      const tx = await factory.connect(ngo).createProject(
        "Education for All",
        "0x000000000000000000000000000000000000dEaD", // donor wallet (dummy)
        "0x000000000000000000000000000000000000dEaD" // primary wallet (dummy)
      );

      // Verify deployed project
      const deployedProjects = await factory.getDeployedProjects();
      expect(deployedProjects.length).to.equal(1);
    });

    it("should not allow non-NGOs to create projects", async function () {
      await expect(
        factory.connect(other).createProject(
          "Health Initiative",
          "0x000000000000000000000000000000000000dEaD",
          "0x000000000000000000000000000000000000dEaD"
        )
      ).to.be.reverted;
    });
  });

  describe("Access Control", function () {
    it("should allow admin to grant NGO_ROLE", async function () {
      await factory.connect(admin).grantRole(await factory.NGO_ROLE(), projectManager.address);
      const isNGO = await factory.hasRole(await factory.NGO_ROLE(), projectManager.address);
      expect(isNGO).to.be.true;
    });

    it("should not allow non-admins to grant NGO_ROLE", async function () {
      await expect(
        factory.connect(ngo).grantRole(await factory.NGO_ROLE(), other.address)
      ).to.be.reverted;
    });
  });

  describe("Deployed Projects", function () {
    it("should return all deployed projects", async function () {
      // Deploy multiple projects
      await factory.connect(ngo).createProject(
        "Project A",
        "0x000000000000000000000000000000000000dEaD",
        "0x000000000000000000000000000000000000dEaD"
      );

      await factory.connect(ngo).createProject(
        "Project B",
        "0x000000000000000000000000000000000000dEaD",
        "0x000000000000000000000000000000000000dEaD"
      );

      const deployedProjects = await factory.getDeployedProjects();
      expect(deployedProjects.length).to.equal(2);

      // Verify project addresses
      for (const address of deployedProjects) {
        expect(address).to.properAddress;
      }
    });
  });
});
