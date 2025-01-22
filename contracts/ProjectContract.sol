
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";


contract ProjectContract is AccessControl, ReentrancyGuard {
    bytes32 public constant PROJECT_MANAGER_ROLE = keccak256("PROJECT_MANAGER_ROLE");
    bytes32 public constant FIELD_AGENT_ROLE = keccak256("FIELD_AGENT_ROLE");
    bytes32 public constant NGO_ROLE = keccak256("NGO_ROLE");

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
    address public ngo;

    IERC20 public token; // ERC20 Token instance

    bool private initialized;
    uint256 public maxSpendingLimit;

    mapping(address => Beneficiary) public beneficiaries;
    mapping(address => Vendor) public vendors;

    event TokensDeposited(address indexed from, uint256 amount);
    event AllowanceAllocated(address indexed beneficiary, uint256 amount);
    event FundsClaimed(address indexed vendor, address indexed beneficiary, uint256 amount);
    event VendorRegistered(address indexed vendor);
    event VendorDeregistered(address indexed vendor);

    modifier onlyManager() {
        require(hasRole(PROJECT_MANAGER_ROLE, msg.sender), "Caller is not a Project Manager");
        _;
    }

     modifier initializer() {
        require(!initialized, "Contract is already initialized");
        initialized = true;
        _;
    }

    function initialize(
        string memory _projectName,
        address _donorWallet,
        address _primaryWallet,
        address _ngo,
        address _token
    ) external initializer {
        require(_donorWallet != address(0), "Donor wallet cannot be zero address");
        require(_primaryWallet != address(0), "Primary wallet cannot be zero address");
        require(_token != address(0), "Token address cannot be zero");

        projectName = _projectName;
        donorWallet = _donorWallet;
        primaryWallet = _primaryWallet;
        ngo = _ngo;
        token = IERC20(_token);
        
        

        _grantRole(NGO_ROLE, _ngo);
        _grantRole(DEFAULT_ADMIN_ROLE, _ngo);
        _grantRole(PROJECT_MANAGER_ROLE, _ngo);
    }

    // Allocate allowance to a beneficiary
    function allocateAllowance(address _beneficiary, uint256 _amount) external onlyManager {
        require(_amount <= maxSpendingLimit, "Amount exceeds spending limit");

        require(token.balanceOf(primaryWallet) >= _amount, "Insufficient funds in primary wallet");

        require(_amount > 0, "Amount must be greater than zero");


        beneficiaries[_beneficiary] = Beneficiary({
            wallet: _beneficiary,
            allowance: _amount,
            isActive: true
        });

        // Transfer tokens from the sender to the contract
        bool success = token.transferFrom(msg.sender, address(this), _amount);
        require(success, "Token transfer failed");


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

          // Transfer tokens from the contract to the vendor
        bool success = token.transfer(_vendor, _amount);
        require(success, "Token transfer failed");

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
        maxSpendingLimit += _newLimit;

    }

    // Allow contract to receive ETH
    receive() external payable {}
}
