// This setup uses Hardhat Ignition to manage smart contract deployments.
// Learn more about it at https://hardhat.org/ignition

const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");



module.exports = buildModule("ProjectFactoryModule", (m) => {

  const _implementation = m.getParameter("_implementation","0xd829E817C24Cf483042dad21dF89C58d8d9aeFD3");  
  const projectFactory = m.contract("ProjectFactory", [_implementation]);

  return { projectFactory };
});

