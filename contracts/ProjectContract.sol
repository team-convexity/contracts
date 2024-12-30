// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract ProjectContract is AccessControl, ReentrancyGuard {
    bytes32 public constant PROJECT_MANAGER_ROLE = keccak256("PROJECT_MANAGER_ROLE");
    bytes32 public constant FIELD_AGENT_ROLE = keccak256("FIELD_AGENT_ROLE");

    struct Beneficiary {
        address wallet;
        uint256 allowance;
        bool isActive;
    }

    struct Vendor {
        address wallet;
        bool isRegistered;
    }

    string public projectName;
    address public donorWallet;
    address public primaryWallet;
    uint256 public maxSpendingLimit;

    mapping(address => Beneficiary) public beneficiaries;
    mapping(address => Vendor) public vendors;

    event AllowanceAllocated(address indexed beneficiary, uint256 amount);
    event FundsClaimed(address indexed vendor, address indexed beneficiary, uint256 amount);
    event VendorRegistered(address indexed vendor);
    event VendorDeregistered(address indexed vendor);

    constructor(string memory _projectName, address _donorWallet, address _primaryWallet, address _ngo) {
        require(_donorWallet != address(0), "Donor wallet cannot be zero address");
        require(_primaryWallet != address(0), "Primary wallet cannot be zero address");

        projectName = _projectName;
        donorWallet = _donorWallet;
        primaryWallet = _primaryWallet;

        _grantRole(DEFAULT_ADMIN_ROLE, _ngo);
        _grantRole(PROJECT_MANAGER_ROLE, _ngo);
    }

    modifier onlyManager() {
        require(hasRole(PROJECT_MANAGER_ROLE, msg.sender), "Caller is not a Project Manager");
        _;
    }

    // Allocate allowance to a beneficiary
    function allocateAllowance(address _beneficiary, uint256 _amount) external onlyManager {
        require(_amount <= maxSpendingLimit, "Amount exceeds spending limit");

        beneficiaries[_beneficiary] = Beneficiary({
            wallet: _beneficiary,
            allowance: _amount,
            isActive: true
        });

        emit AllowanceAllocated(_beneficiary, _amount);
    }

    // Vendor claims funds
    function claimFunds(address _vendor, uint256 _amount) external nonReentrant {
        Vendor storage vendor = vendors[_vendor];
        Beneficiary storage beneficiary = beneficiaries[msg.sender];

        require(vendor.isRegistered, "Vendor not registered");
        require(beneficiary.isActive, "Beneficiary not active");
        require(beneficiary.allowance >= _amount, "Insufficient allowance");

        beneficiary.allowance -= _amount;

        (bool success, ) = _vendor.call{value: _amount}("");
        require(success, "Transfer failed");

        emit FundsClaimed(_vendor, msg.sender, _amount);
    }

    // Register a vendor
    function registerVendor(address _vendor) external onlyManager {
        require(_vendor != address(0), "Vendor cannot be zero address");

        vendors[_vendor] = Vendor({
            wallet: _vendor,
            isRegistered: true
        });

        emit VendorRegistered(_vendor);
    }

    // Deregister a vendor
    function deregisterVendor(address _vendor) external onlyManager {
        require(vendors[_vendor].isRegistered, "Vendor not registered");

        vendors[_vendor].isRegistered = false;

        emit VendorDeregistered(_vendor);
    }

    // Update maximum spending limit
    function updateMaxSpendingLimit(uint256 _newLimit) external onlyManager {
        require(_newLimit > 0, "Spending limit must be greater than zero");
        maxSpendingLimit = _newLimit;
    }

    // Allow contract to receive ETH
    receive() external payable {}
}
