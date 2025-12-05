const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("ProjectContract", function () {
  let ProjectContract, project, admin, donor, primary, beneficiary, token, vendor, other;

  beforeEach(async function () {
    [admin, donor, primary, beneficiary, token, vendor, ngo, other] = await ethers.getSigners();

    // Deploy the ProjectContract
    ProjectContract = await ethers.getContractFactory("ProjectContract");
    project = await ProjectContract.deploy();
    await project.initialize("projectX", donor.address, primary.address, ngo.address, token.address);

  });

  describe("Initialization", function () {
    it("should initialize with the correct donor and primary wallets", async function () {
      expect(await project.donorWallet()).to.equal(donor.address);
      expect(await project.primaryWallet()).to.equal(primary.address);
    });

    it("should set the admin as the default admin", async function () {
      const isAdmin = await project.hasRole(await project.DEFAULT_ADMIN_ROLE(), ngo.address);
      expect(isAdmin).to.be.true;
    });
  });

  describe("Beneficiary Management", function () {
    it("should not allow non-admins to allocate allowance", async function () {
      await expect(
        project.connect(other).allocateAllowance(beneficiary.address, ethers.parseEther("5"))
      ).to.be.revertedWith("Caller is not a Project Manager");
    });
  });

  describe("Vendor Management", function () {
    it("should allow registering a vendor", async function () {
      await project.connect(ngo).registerVendor(vendor.address);

      const vendorData = await project.vendors(vendor.address);
      expect(vendorData.wallet).to.equal(vendor.address);
      expect(vendorData.isRegistered).to.be.true;
    });

    it("should allow deregistering a vendor", async function () {
      await project.connect(ngo).registerVendor(vendor.address);
      await project.connect(ngo).deregisterVendor(vendor.address);

      const vendorData = await project.vendors(vendor.address);
      expect(vendorData.isRegistered).to.be.false;
    });

    it("should not allow non-admins to register a vendor", async function () {
      await expect(project.connect(other).registerVendor(vendor.address)).to.be.revertedWith(
        "Caller is not a Project Manager"
      );
    });
  });

  // describe("Fund Claiming", function () {
  //   beforeEach(async function () {

  //     await project.connect(ngo).updateMaxSpendingLimit(ethers.parseEther("10"));
  //     await project.connect(ngo).allocateAllowance(beneficiary.address, ethers.parseEther("10"));
  //     await project.connect(ngo).registerVendor(vendor.address);

  //     // Send ETH to the contract to simulate funding
  //     await admin.sendTransaction({
  //       to: project.target,
  //       value: ethers.parseEther("20"),
  //     });
  //   });

  //   it("should allow a vendor to claim funds from a beneficiary's allowance", async function () {
  //     await project.connect(beneficiary).claimFunds(vendor.address, ethers.parseEther("5"));

  //     const beneficiaryData = await project.beneficiaries(beneficiary.address);
  //     expect(beneficiaryData.allowance).to.equal(ethers.parseEther("5"));

  //     const vendorBalance = await ethers.provider.getBalance(vendor.address);
  //     expect(vendorBalance).to.be.above(ethers.parseEther("5")); // Vendor receives the funds
  //   });

  //   it("should not allow claiming more than the allowance", async function () {
  //     await expect(
  //       project.connect(beneficiary).claimFunds(vendor.address, ethers.parseEther("15"))
  //     ).to.be.revertedWith("Insufficient allowance");
  //   });

  //   it("should not allow unregistered vendors to claim funds", async function () {
  //     await project.connect(ngo).deregisterVendor(vendor.address);

  //     await expect(
  //       project.connect(beneficiary).claimFunds(vendor.address, ethers.parseEther("5"))
  //     ).to.be.revertedWith("Vendor not registered");
  //   });

  //   // it("should not allow inactive beneficiaries to claim funds", async function () {
  //   //   await project.connect(ngo).allocateAllowance(beneficiary.address, ethers.parseEther("0"));

  //   //   await expect(
  //   //     project.connect(beneficiary).claimFunds(vendor.address, ethers.parseEther("5"))
  //   //   ).to.be.revertedWith("Beneficiary not active");
  //   // });
  // });

  describe("Spending Limits", function () {
    it("should enforce maximum spending limits", async function () {
      await project.connect(ngo).updateMaxSpendingLimit(ethers.parseEther("5"));

      await expect(
        project.connect(ngo).allocateAllowance(beneficiary.address, ethers.parseEther("10"))
      ).to.be.revertedWith("Amount exceeds spending limit");
    });
  });

  describe("ETH Management", function () {
    it("should allow receiving ETH", async function () {
      await admin.sendTransaction({
        to: project.target,
        value: ethers.parseEther("10"),
      });

      const balance = await ethers.provider.getBalance(project.target);
      expect(balance).to.equal(ethers.parseEther("10"));
    });
  });
});
