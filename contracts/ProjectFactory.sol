// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import "@openzeppelin/contracts/proxy/Clones.sol";
import "./ProjectContract.sol";

contract ProjectFactory {
    address public implementation;
    address[] public deployedProjects;

    event ProjectCreated(address indexed projectAddress, string projectName);

    constructor(address _implementation) {
        require(_implementation != address(0), "Implementation address cannot be zero");
        implementation = _implementation;
    }

    function createProject(
        string memory _projectName,
        address _donorWallet,
        address _primaryWallet,
        address _ngo,
        address _token
    ) external returns (address) {
        address clone = Clones.clone(implementation);

        ProjectContract(payable(clone)).initialize(
            _projectName,
            _donorWallet,
            _primaryWallet,
            _ngo,
            _token
        );

        deployedProjects.push(clone);

        emit ProjectCreated(clone, _projectName);
        return clone;
    }

    function getDeployedProjects() external view returns (address[] memory) {
        return deployedProjects;
    }
}
