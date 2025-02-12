#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, log, symbol_short, Address, Env, Map, String, Symbol, Vec, IntoVal};
// use soroban_sdk::IntoContract;


#[contract]
pub struct ProjectContract;


#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct Beneficiary {
    pub wallet: Address,
    pub allowance: u64,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Vendor {
    wallet: Address,
    is_registered: bool,
}


#[contractimpl]
impl ProjectContract {
    pub fn initialize(
        env: Env,
        project_name: String,
        donor_wallet: Address,
        primary_wallet: Address,
        ngo: Address,
    ) {
        let initialized_key = Symbol::new(&env, "initialized");
    
        // Check if already initialized
        let initialized: bool = env
            .storage()
            .instance()
            .get(&initialized_key)
            .unwrap_or(false);
    
                if initialized {
                    panic!("Contract is already initialized");
                } else {
                    // Mark as initialized
                    env.storage().instance().set(&initialized_key, &true);
                    // log!("Contract initialized successfully.")
                }
            
                // // Validate inputs
                // if donor_wallet == Address::from([0u8; 32]) || primary_wallet == Address::from([0u8; 32]) {
                //     panic!("Donor wallet or primary wallet is not valid.");
                // }
            
                // Store data in contract storage
        let project_name_key = Symbol::new(&env, "project_name");
        env.storage().instance().set(&project_name_key, &project_name);
    
        let donor_wallet_key = Symbol::new(&env, "donor_wallet");
        env.storage().instance().set(&donor_wallet_key, &donor_wallet);
    
        let primary_wallet_key = Symbol::new(&env, "primary_wallet");
        env.storage().instance().set(&primary_wallet_key, &primary_wallet);
    
        let ngo_key = Symbol::new(&env, "ngo");
        env.storage().instance().set(&ngo_key, &ngo);


//         // Initialize roles
// let mut roles: Map<soroban_sdk::String, Vec<Address>> = Map::new(&env);
// // Set the NGO_ROLE
// roles.set( soroban_sdk::String::from_slice(&env, "NGO_ROLE"), Vec::from_array(&env, [ngo.clone()]),
// );

// // Set the PROJECT_MANAGER_ROLE
// roles.set( soroban_sdk::String::from_slice(&env, "PROJECT_MANAGER_ROLE"), Vec::from_array(&env, [ngo.clone()]),
// );

// // Set the DEFAULT_ADMIN_ROLE
// roles.set(
//     soroban_sdk::String::from_slice(&env, "DEFAULT_ADMIN_ROLE"),
//     Vec::from_array(&env, [ngo.clone()]),
// );

// // Store the roles map in contract storage
// let roles_key = Symbol::new(&env, "roles");
// env.storage().instance().set(&roles_key, &roles);
}

pub fn allocate_allowance(env: Env, beneficiary: Address, amount: u64) {
    // Ensure caller has manager role
    // let caller = env.source_account();
    // Self::ensure_role(&env, caller, "PROJECT_MANAGER_ROLE");

    // Get maximum spending limit from storage
    let max_spending_limit_key = Symbol::new(&env, "max_spending_limit");
    let max_spending_limit: u64 = env.storage()
        .instance()
        .get(&max_spending_limit_key)
        .unwrap_or(0); // Default to 0 if not found

    // Check if the amount exceeds the spending limit
    if amount > max_spending_limit {
        panic!("Amount exceeds spending limit");
    }

    // Get beneficiaries map from storage
    let beneficiaries_key = Symbol::new(&env, "beneficiaries");
    let mut beneficiaries: Map<Address, Beneficiary> = env.storage()
        .instance()
        .get(&beneficiaries_key)
        .unwrap_or_else(|| Map::new(&env)); // Default to empty Map if not found

    // Add or update the beneficiary's allowance
    beneficiaries.set(
        beneficiary.clone(),
        Beneficiary {
            wallet: beneficiary.clone(),
            allowance: amount,
            is_active: true,
        },
    );

    // Save updated beneficiaries map back to storage
    let beneficiaries_raw = beneficiaries; // Store the updated map
    env.storage().instance().set(&beneficiaries_key, &beneficiaries_raw);
}

pub fn claim_funds(env: Env, vendor: Address, beneficiary_address: Address, donor: Address,token_address: Address,
     amount: u64) {
    // ensures is the caller of function
    donor.require_auth();
    // Get beneficiaries and vendors from storage
    let beneficiaries_key = Symbol::new(&env, "beneficiaries");
    let vendors_key = Symbol::new(&env, "vendors");

    // Retrieve beneficiaries map from storage
    let beneficiaries: Map<Address, Beneficiary> = env.storage()
        .instance()
        .get(&beneficiaries_key)
        .unwrap_or_else(|| Map::new(&env)); // Default to empty Map if not found

    // Retrieve vendors map from storage
    let vendors: Map<Address, Vendor> = env.storage()
        .instance()
        .get(&vendors_key)
        .unwrap_or_else(|| Map::new(&env)); // Default to empty Map if not found

    // Check if vendor is registered
    let vendor_info: Vendor = vendors.get(vendor.clone()).unwrap_or_else(|| panic!("Vendor not found"));
    if !vendor_info.is_registered {
        panic!("Vendor not registered");
    }

    // Check if beneficiary is active
    let beneficiary_info: Beneficiary = beneficiaries.get(beneficiary_address.clone()).unwrap_or_else(|| panic!("Beneficiary not found"));
    if !beneficiary_info.is_active {
        panic!("Beneficiary not active");
    }

    // Check if beneficiary has enough allowance
    // let donor_wallet: Address = donor;
    if amount > beneficiary_info.allowance {
        panic!("Beneficiary does not have enough allowance");
    }

    // Transfer funds
    let beneficiary = beneficiary_info.wallet.clone();
    // let transfer = Transfer {
    //     from: donor_wallet,
    //     to: beneficiary,
    //     amount: amount,
    // };

     // Call the `transfer` method on the token contract
    let mut beneficiaries: Map<Address, Beneficiary> = env.storage()
        .instance()
        .get(&beneficiaries_key)
        .unwrap_or_else(|| Map::new(&env));

      // Convert the amount from u64 to i128, as the token contract expects an i128.
      let amount_i128: i128 = amount as i128;

       // Call the `transfer` method on the token contract.
        env.invoke_contract::<()>( 
            &token_address,
            &symbol_short!("transfer"),
                 (&donor, &vendor, &amount_i128).into_val(&env) // Arguments for `transfer`
                    );

    // Update beneficiary's allowance
    let mut updated_beneficiary = beneficiary_info;
    updated_beneficiary.allowance -= amount;
    beneficiaries.set(beneficiary.clone(), updated_beneficiary);

    // Save updated beneficiaries map back to storage
    env.storage().instance().set(&beneficiaries_key, &beneficiaries);
}

pub fn deregister_vendor(env: Env, vendor: Address) {
    // Ensure caller has manager role
    // let caller = env.source_account();
    // Self::ensure_role(&env, caller, "PROJECT_MANAGER_ROLE");

    // Remove vendor
    let vendors_key = Symbol::new(&env, "vendors");

    // Retrieve vendors map from storage using get
    let mut vendors: Map<Address, Vendor> = env
        .storage()
        .instance()
        .get::<_, Map<Address, Vendor>>(&vendors_key)
        .unwrap_or_else(|| Map::new(&env));
    
    let vendor_info: Vendor = vendors.get(vendor.clone()).unwrap_or_else(|| panic!("Vendor not found"));

    // Check if vendor is registered
    if !vendor_info.is_registered {
        panic!("Vendor not registered");
    }
    // Remove vendor from the map
    vendors.remove(vendor);

    env.storage().instance().set(&vendors_key, &vendors);
}

pub fn register_vendor(env: Env, vendor: Address) {

    // Ensure caller has manager role
    // let caller = env.source_account();
    // Self::ensure_role(&env, caller, "PROJECT_MANAGER_ROLE");

    // Get vendors map from storage
    let vendors_key = Symbol::new(&env, "vendors");
    let mut vendors: Map<Address, Vendor> = env.storage()
        .instance()
        .get(&vendors_key)
        .unwrap_or_else(|| Map::new(&env));

    // Check if vendor is already registered
    if vendors.contains_key(vendor.clone()) {
        panic!("Vendor already registered");
    }

    // Register vendor
    vendors.set(
        vendor.clone(),
        Vendor {
            wallet: vendor.clone(),
            is_registered: true,
        },
    );

    // Save updated vendors map back to storage
    env.storage().instance().set(&vendors_key, &vendors);
}

pub fn update_max_spending_limit(env: Env, new_limit: u64) {
    // Ensure caller has manager role
    // let caller = env.source_account();
    // Self::ensure_role(&env, caller, "PROJECT_MANAGER_ROLE");

    // Set the maximum spending limit
    let key = Symbol::new(&env, "max_spending_limit");
    env.storage().instance().set(&key, &new_limit);
}

//   fn ensure_role(env: &Env, caller: Address, role: &str) {
//     let roles = env.storage().instance().get::<Symbol, Map<Symbol, Vec<Address>>>("roles").unwrap_or_default();
//     let role_members = roles.get(&Symbol::from(role)).unwrap_or_default();
//     if !role_members.iter().any(|addr| *addr == caller) {
//         panic!("Caller does not have the required role");
    }
