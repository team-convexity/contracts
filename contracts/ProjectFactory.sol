// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./ProjectContract.sol";

contract ProjectFactory is AccessControl {
    bytes32 public constant NGO_ROLE = keccak256("NGO_ROLE");

    // Store deployed project contracts
    address[] public deployedProjects;

    event ProjectCreated(address indexed projectAddress, string name, address ngo);

    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(NGO_ROLE, msg.sender);
    }

    // Deploy a new project contract
    function createProject(string memory name, address donorWallet, address primaryWallet) external onlyRole(NGO_ROLE) {
        ProjectContract newProject = new ProjectContract(name, donorWallet, primaryWallet, msg.sender);
        deployedProjects.push(address(newProject));
        emit ProjectCreated(address(newProject), name, msg.sender);
    }

    // Get all deployed projects
    function getDeployedProjects() external view returns (address[] memory) {
        return deployedProjects;
    }
}
