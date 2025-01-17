const { ethers } = require("ethers");

(async () => {
  const provider = new ethers.JsonRpcProvider("https://sepolia.infura.io/v3/07f0dfde071243bdbc4c3a53562536cf");
  const privateKey = "";// replace with privateKey
  const wallet = new ethers.Wallet(privateKey, provider);

  const projectFactoryAddress = "0x7d349a9a1DDF9bcadaa25F7c909037429824e5B3";
  const projectImplementationAddress = "0xd829E817C24Cf483042dad21dF89C58d8d9aeFD3"; 

  // ABI of the ProjectFactory contract
  const factoryABI = [
    "event ProjectCreated(address indexed projectAddress, string projectName)",
    "function createProject(string _projectName, address _donorWallet, address _primaryWallet, address _ngo) external returns (address)",
    "function getDeployedProjects() external view returns (address[])",
  ];

  // Connect to the factory contract
  const factoryContract = new ethers.Contract(projectFactoryAddress, factoryABI, wallet);

  // Parameters for creating a new project
  const projectName = "MyProject";
  const donorWallet = "0x57C79794d22595C60f0034287D019882Cf63c046"; 
  const primaryWallet = "0x06a8E8f19682749Fd01807943168020F52E0DA58";
  const ngoWallet = "0x5f98Bf2254BF20F70f1ca7722abBa28359591deD";

  try {
    // Create a new project clone
    console.log("Creating a new project...");
    const txResponse = await factoryContract.createProject(projectName, donorWallet, primaryWallet, ngoWallet);
    console.log("Transaction submitted:", txResponse.hash);

    // Wait for the transaction to be confirmed
    const receipt = await txResponse.wait();
    console.log("Transaction confirmed:", receipt);

    // Parse the ProjectCreated event from the receipt
    const event = receipt.logs.map((log) => {
      try {
        return factoryContract.interface.parseLog(log);
      } catch (err) {
        return null;
      }
    }).filter(Boolean)[0]; // Get the first valid event

    if (event) {
      const { projectAddress, projectName } = event.args;
      console.log(`New project created: Address=${projectAddress}, Name=${projectName}`);
    } else {
      console.log("No ProjectCreated event found in the transaction logs.");
    }

    // Fetch and display all deployed projects
    console.log("Fetching deployed projects...");
    const deployedProjects = await factoryContract.getDeployedProjects();
    console.log("Deployed Projects:", deployedProjects);
  } catch (error) {
    console.error("Error during project creation:", error);
  }
})();
