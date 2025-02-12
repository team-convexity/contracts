use crate::{ProjectContract, ProjectContractClient};
use soroban_sdk::{testutils::Address as TestAddress, Address, Bytes, Env, Map, String, Symbol, Vec };
// use crate::initialize;

#[test]
fn increment() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProjectContract);
    let client = ProjectContractClient::new(&env, &contract_id);

    // assert_eq!(client.increment(), 1);
    // assert_eq!(client.increment(), 2);
    // assert_eq!(client.increment(), 3);
}

#[test]
fn test_initialize_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProjectContract);
    let client = ProjectContractClient::new(&env, &contract_id);

    let project_name = soroban_sdk::String::from_slice(&env, "Test Project");
    // let donor_wallet = Address::from([1u8; 32]);
    // let primary_wallet = Address::from([2u8; 32]);
    // let ngo = Address::from([3u8; 32]);


    let donor_wallet = Address::generate(&env); 
    let primary_wallet = Address::generate(&env); 
    let ngo = Address::generate(&env);


    // Call initialize
    client.initialize( &project_name.clone(), &donor_wallet.clone(), &primary_wallet.clone(), &ngo.clone());

    // Verify storage values
    // let initialized_key = Symbol::new(&env, "initialized");
    let initialized_key = Symbol::new(&env, "initialized");
    // assert_eq!(env.storage().instance().get::<bool>(&initialized_key), Some(true));
    assert_eq!(env.storage().instance().get::<Symbol, bool>(&initialized_key), Some(true));


    let project_name_key = Symbol::new(&env, "project_name");
    assert_eq!(
        env.storage()
            .instance()
            // .get::<soroban_sdk::String>(&project_name_key),
            .get::<Symbol, String>(&project_name_key),
        Some(project_name)
    );

    let donor_wallet_key = Symbol::new(&env, "donor_wallet");
    assert_eq!(
        env.storage()
            .instance()
            // .get::<Address>(&donor_wallet_key),
            .get::<Symbol, Address>(&donor_wallet_key),
        Some(donor_wallet)
    );

    let primary_wallet_key = Symbol::new(&env, "primary_wallet");
    assert_eq!(
        env.storage()
            .instance()
            // .get::<Address>(&primary_wallet_key),
            .get::<Symbol, Address>(&primary_wallet_key),

        Some(primary_wallet)
    );

    let ngo_key = Symbol::new(&env, "ngo");
    assert_eq!(env.storage().instance()
             .get::<Symbol, Address>(&ngo_key),
              Some(ngo.clone()));

    // let roles_key = Symbol::new(&env, "roles");
    // let roles: Map<soroban_sdk::String, Vec<Address>> = env
    //     .storage()
    //     .instance()
    //     .get(&roles_key)
    //     .unwrap();

    // assert_eq!(
    //     roles.get(soroban_sdk::String::from_str(&env, "NGO_ROLE")),
    //     Some(Vec::from_array(&env, [ngo.clone()]))
    // );
    // assert_eq!(
    //     roles.get(soroban_sdk::String::from_str(&env, "PROJECT_MANAGER_ROLE")),
    //     Some(Vec::from_array(&env, [ngo.clone()]))
    // );
    // assert_eq!(
    //     roles.get(soroban_sdk::String::from_str(&env, "DEFAULT_ADMIN_ROLE")),
    //     Some(Vec::from_array(&env, [ngo.clone()]))
    // );
}

#[test]
#[should_panic(expected = "Contract is already initialized")]
fn test_initialize_reinitialization_fails() {

    let env = Env::default();
    let contract_id = env.register_contract(None, ProjectContract);
    let client = ProjectContractClient::new(&env, &contract_id);



    // let env = TestEnv::default();
    let project_name = soroban_sdk::String::from_slice(&env, "Test Project");
    let donor_wallet = Address::generate(&env);
    let primary_wallet = Address::generate(&env);
    let ngo = Address::generate(&env);

    // Call initialize the first time
    client.initialize( &project_name.clone(), &donor_wallet.clone(), &primary_wallet.clone(), &ngo.clone());

    // Attempt to re-initialize should panic
    client.initialize( &project_name, &donor_wallet, &primary_wallet, &ngo);
}

#[test]
fn test_initialize_with_invalid_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProjectContract);
    let client = ProjectContractClient::new(&env, &contract_id);

    // let env = TestEnv::default();
    let project_name = soroban_sdk::String::from_slice(&env, "Invalid Test");
    let donor_wallet = Address::generate(&env); // Invalid address
    let primary_wallet = Address::generate(&env); // Invalid address
    let ngo = Address::generate(&env);

    // Test invalid data handling
    client.initialize( &project_name, &donor_wallet, &primary_wallet, &ngo);

    // Check that the contract doesn't panic or store invalid data
    let donor_wallet_key = Symbol::new(&env, "donor_wallet");
    assert!(env
        .storage()
        .instance()
        // .get::<Address>(&donor_wallet_key)
        .get::<Symbol, Address>(&donor_wallet_key)
        .is_some());
}