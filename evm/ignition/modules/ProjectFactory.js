// This setup uses Hardhat Ignition to manage smart contract deployments.
// Learn more about it at https://hardhat.org/ignition

const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");



module.exports = buildModule("ProjectFactoryModule", (m) => {

  const _implementation = m.getParameter("_implementation","0x88ffA5BBEf62ffF9C7561404808f2b8D01dAa979");  
  const projectFactory = m.contract("ProjectFactory", [_implementation]);

  return { projectFactory };
});

