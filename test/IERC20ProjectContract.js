const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Modified ProjectContract with IERC20", function () {
  let ERC20Token, erc20Token, ProjectContract, project, owner, donor, primary, ngo, beneficiary, vendor;

  beforeEach(async function () {
    // Accounts
    [owner, donor, primary, ngo, beneficiary, vendor] = await ethers.getSigners();

    // Deploy ERC20 token
    ERC20Token = await ethers.getContractFactory("ERC20Token");
    erc20Token = await ERC20Token.deploy("Test Token", "TTK");

    // Mint tokens to primary wallet
    await erc20Token.mint(primary.address, ethers.parseEther("500"));

    // Deploy ProjectContract
    ProjectContract = await ethers.getContractFactory("ProjectContract");
    project = await ProjectContract.deploy();

    // Initialize ProjectContract
    await project.connect(ngo).initialize(
      "Test Project",
      donor.address,
      primary.address,
      ngo.address,
      erc20Token.target
    );
  });

  describe("ERC20 Token Interactions", function () {
    it("should allow the primary wallet to approve and deposit tokens into the contract", async function () {
      // Approve tokens
      await erc20Token.connect(primary).approve(project.target, ethers.parseEther("100"));

      // Deposit tokens into the contract
      await project.connect(primary).depositTokens(ethers.parseEther("100"));

      // Check contract balance
      const contractBalance = await erc20Token.balanceOf(project.target);
      expect(contractBalance).to.equal(ethers.parseEther("100"));
    });

    it("should allocate an allowance to a beneficiary", async function () {
      // Set max spending limit
      await project.connect(ngo).updateMaxSpendingLimit(ethers.parseEther("200"));

      // Allocate allowance
      await project.connect(ngo).allocateAllowance(beneficiary.address, ethers.parseEther("50"));

      const beneficiaryData = await project.beneficiaries(beneficiary.address);
      expect(beneficiaryData.allowance).to.equal(ethers.parseEther("50"));
    });

    it("should allow a beneficiary to claim funds and transfer tokens to a vendor", async function () {
      // Approve and deposit tokens
      await erc20Token.connect(primary).approve(project.target, ethers.parseEther("100"));
      await project.connect(primary).depositTokens(ethers.parseEther("100"));

      // Allocate allowance
      await project.connect(ngo).updateMaxSpendingLimit(ethers.parseEther("200"));
      await project.connect(ngo).allocateAllowance(beneficiary.address, ethers.parseEther("50"));

      // Register vendor
        await project.connect(ngo).registerVendor(vendor.address);
      // Beneficiary claims funds
      await erc20Token.connect(primary).approve(vendor.address, ethers.parseEther("50"));
      await project.connect(beneficiary).claimFunds(vendor.address, ethers.parseEther("50"));

      // Check vendor's token balance
      const vendorBalance = await erc20Token.balanceOf(vendor.address);
      expect(vendorBalance).to.equal(ethers.parseEther("50"));

      // Check beneficiary's allowance
      const beneficiaryData = await project.beneficiaries(beneficiary.address);
      expect(beneficiaryData.allowance).to.equal(ethers.parseEther("0"));
    });

    it("should not allow a beneficiary to claim more than their allowance", async function () {
      // Allocate allowance
      await project.connect(ngo).updateMaxSpendingLimit(ethers.parseEther("200"));
      await project.connect(ngo).allocateAllowance(beneficiary.address, ethers.parseEther("50"));
      // Register vendor
      await project.connect(ngo).registerVendor(vendor.address);
      // Try to claim more than the allocated allowance
      await expect(
        project.connect(beneficiary).claimFunds(vendor.address, ethers.parseEther("60"))
      ).to.be.revertedWith("Insufficient allowance");
    });

    it("should not allow allocating an allowance exceeding the max spending limit", async function () {
      // Set max spending limit
      await project.connect(ngo).updateMaxSpendingLimit(ethers.parseEther("40"));

      // Try to allocate more than the max spending limit
      await expect(
        project.connect(ngo).allocateAllowance(beneficiary.address, ethers.parseEther("50"))
      ).to.be.revertedWith("Amount exceeds spending limit");
    });

  });
});
