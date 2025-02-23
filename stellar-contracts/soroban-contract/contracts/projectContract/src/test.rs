#[cfg(test)]
mod test {
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String, Symbol};

    use crate::{
        CashAllowance, ItemAllowance, ProjectContract, ProjectContractClient, CASH_ALLOWANCES_KEY,
        ITEM_ALLOWANCES_KEY,
    };

    fn setup_contract(env: &Env) -> (ProjectContractClient, Address, Address, Address, Address) {
        let contract_id = env.register(ProjectContract, ());
        let client = ProjectContractClient::new(env, &contract_id);

        let owner = Address::generate(env);
        let ngo = Address::generate(env);
        let beneficiary = Address::generate(env);
        let vendor = Address::generate(env);

        // initialize the contract
        client.initialize(&owner);

        // add roles
        let project_id = String::from_str(env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        client.add_role(&owner, &project_id, &String::from_str(env, "ngo"), &ngo);
        client.add_role(
            &owner,
            &project_id,
            &String::from_str(env, "beneficiary"),
            &beneficiary,
        );
        client.add_role(
            &owner,
            &project_id,
            &String::from_str(env, "vendor"),
            &vendor,
        );

        (client, owner, ngo, beneficiary, vendor)
    }

    // -- allowance management tests --

    #[test]
    fn test_allocate_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let currency = String::from_str(&env, "ChatsUSDC");

        // allocate cash allowance
        client.allocate_cash_allowance(&ngo, &project_id, &beneficiary, &100, &currency, &None);

        // verify the allowance was allocated
        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Cash allowances not found"))
            });

        let allowance = allowances
            .get((beneficiary.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Cash allowance not found"));
        assert_eq!(allowance.amount, 100);
    }

    #[test]
    fn test_claim_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let currency = String::from_str(&env, "ChatsUSDC");

        // allocate cash allowance
        client.allocate_cash_allowance(&ngo, &project_id, &beneficiary, &100, &currency, &None);

        // claim cash allowance
        client.claim_cash_allowance(&beneficiary, &project_id, &currency, &50, &None);

        // verify the remaining allowance
        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Cash allowances not found"))
            });

        let allowance = allowances
            .get((beneficiary.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Cash allowance not found"));
        assert_eq!(allowance.amount, 50);
    }

    #[test]
    fn test_allocate_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id = String::from_str(&env, "QmVuZWZpY2lhcnk6mBF_cv2JQg0AAAAAAAAAAQ");

        // allocate item allowance
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &10, &None);

        // verify the allowance was allocated
        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Item allowances not found"))
            });

        let allowance = allowances
            .get((beneficiary.clone(), item_id.clone()))
            .unwrap_or_else(|| panic!("Item allowance not found"));
        assert_eq!(allowance.quantity, 10);
    }

    #[test]
    fn test_claim_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id = String::from_str(&env, "QmVuZWZpY2lhcnk6mBF_cv2JQg0AAAAAAAAAAQ");

        // allocate item allowance
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &10, &None);

        // claim item allowance
        client.claim_item_allowance(
            &beneficiary,
            &Address::generate(&env),
            &project_id,
            &item_id,
            &5,
        );

        // verify the remaining allowance
        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Item allowances not found"))
            });

        let allowance = allowances
            .get((beneficiary.clone(), item_id.clone()))
            .unwrap_or_else(|| panic!("Item allowance not found"));
        assert_eq!(allowance.quantity, 5);
    }

    #[test]
    fn test_allocate_cash_allowances_batch() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let currency = String::from_str(&env, "ChatsUSDC");

        // allocate cash allowances in batch
        let allowances = vec![
            &env,
            (beneficiary.clone(), currency.clone(), 100, None),
            (Address::generate(&env), currency.clone(), 200, None),
        ];
        client.allocate_cash_allowances_batch(&ngo, &project_id, &allowances);

        // verify the allowances were allocated
        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Cash allowances not found"))
            });

        let allowance = allowances
            .get((beneficiary.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Cash allowance not found"));
        assert_eq!(allowance.amount, 100);
    }

    #[test]
    fn test_allocate_item_allowances_batch() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id = String::from_str(&env, "QmVuZWZpY2lhcnk6mBF_cv2JQg0AAAAAAAAAAQ");

        // allocate item allowances in batch
        let allowances = vec![
            &env,
            (beneficiary.clone(), item_id.clone(), 10, None),
            (Address::generate(&env), item_id.clone(), 20, None),
        ];
        client.allocate_item_allowances_batch(&ngo, &project_id, &allowances);

        // verify the allowances were allocated
        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Item allowances not found"))
            });

        let allowance = allowances
            .get((beneficiary.clone(), item_id.clone()))
            .unwrap_or_else(|| panic!("Item allowance not found"));
        assert_eq!(allowance.quantity, 10);
    }

    #[test]
    fn test_transfer_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let currency = String::from_str(&env, "ChatsUSDC");

        // allocate cash allowance
        client.allocate_cash_allowance(&ngo, &project_id, &beneficiary, &100, &currency, &None);

        // transfer cash allowance
        let new_allowee = Address::generate(&env);
        client.transfer_cash_allowance(&beneficiary, &project_id, &new_allowee, &currency, &50);

        // verify the transfer
        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Cash allowances not found"))
            });

        // check that the sender's allowance is reduced to 50
        let src_allowance = allowances
            .get((beneficiary.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Source cash allowance not found"));
        assert_eq!(src_allowance.amount, 50);

        // check that the new allowee has the transferred amount
        let dest_allowance = allowances
            .get((new_allowee.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Destination cash allowance not found"));
        assert_eq!(dest_allowance.amount, 50);

        // snsure the sender's allowance is not zero
        assert_ne!(src_allowance.amount, 0);
    }

    #[test]
    fn test_transfer_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id = String::from_str(&env, "QmVuZWZpY2lhcnk6mBF_cv2JQg0AAAAAAAAAAQ");

        // allocate item allowance
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &10, &None);

        // transfer item allowance
        let new_allowee = Address::generate(&env);
        client.transfer_item_allowance(&beneficiary, &project_id, &new_allowee, &item_id, &5);

        // verify the transfer
        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Item allowances not found"))
            });

        // check that the sender's allowance is reduced to 5
        let src_allowance = allowances
            .get((beneficiary.clone(), item_id.clone()))
            .unwrap_or_else(|| panic!("Source item allowance not found"));
        assert_eq!(src_allowance.quantity, 5);

        // check that the new allowee has the transferred quantity
        let dest_allowance = allowances
            .get((new_allowee.clone(), item_id.clone()))
            .unwrap_or_else(|| panic!("Destination item allowance not found"));
        assert_eq!(dest_allowance.quantity, 5);

        // snsure the sender's allowance is not zero
        assert_ne!(src_allowance.quantity, 0);
    }

    #[test]
    fn test_redeem_item_claims() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, vendor) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id = String::from_str(&env, "QmVuZWZpY2lhcnk6mBF_cv2JQg0AAAAAAAAAAQ");

        // allocate item allowance to the beneficiary
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &10, &None);

        // transfer some item allowance to the vendor
        client.transfer_item_allowance(&beneficiary, &project_id, &vendor, &item_id, &5);

        // redeem item claims
        client.redeem_item_claims(&vendor, &Some(project_id.clone()), &item_id, &5);

        // verify the redemption
        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Item allowances not found"))
            });

        let vendor_allowance = allowances
            .get((vendor.clone(), item_id.clone()))
            .unwrap_or_else(|| panic!("Vendor item allowance not found"));

        // assert that the vendor's allowance be fully redeemed at this point
        assert_eq!(vendor_allowance.quantity, 0);
    }

    #[test]
    fn test_redeem_cash_claims() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, vendor) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let currency = String::from_str(&env, "ChatsUSDC");

        // allocate cash allowance to the beneficiary
        client.allocate_cash_allowance(&ngo, &project_id, &beneficiary, &100, &currency, &None);

        // verify the beneficiary's allowance before transfer
        let allowances_before_transfer: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Cash allowances not found"))
            });

        let beneficiary_allowance_before = allowances_before_transfer
            .get((beneficiary.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Beneficiary cash allowance not found"));
        assert_eq!(beneficiary_allowance_before.amount, 100);

        // transfer some cash allowance to the vendor
        client.transfer_cash_allowance(&beneficiary, &project_id, &vendor, &currency, &50);

        // verify the beneficiary's allowance after transfer
        let allowances_after_transfer: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Cash allowances not found"))
            });

        let beneficiary_allowance_after = allowances_after_transfer
            .get((beneficiary.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Beneficiary cash allowance not found"));
        assert_eq!(beneficiary_allowance_after.amount, 50);

        // verify the vendor's allowance before redemption
        let vendor_allowance_before = allowances_after_transfer
            .get((vendor.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Vendor cash allowance not found"));
        assert_eq!(vendor_allowance_before.amount, 50);

        // redeem cash claims
        client.redeem_cash_claims(&vendor, &Some(project_id.clone()), &currency, &50);

        // verify the redemption
        let allowances_after_redemption: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .instance()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap_or_else(|| panic!("Cash allowances not found"))
            });

        // check vendor's allowance is now 0 (entry removed when amount reaches 0)
        let vendor_allowance_post =
            allowances_after_redemption.get((vendor.clone(), currency.clone()));
        assert!(vendor_allowance_post.is_none());

        // verify the beneficiary's allowance remains unchanged after redemption
        let beneficiary_allowance_post = allowances_after_redemption
            .get((beneficiary.clone(), currency.clone()))
            .unwrap_or_else(|| panic!("Beneficiary cash allowance not found"));
        assert_eq!(beneficiary_allowance_post.amount, 50);
    }

    #[test]
    fn test_get_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let currency = String::from_str(&env, "ChatsUSDC");

        // allocate cash allowance
        client.allocate_cash_allowance(&ngo, &project_id, &beneficiary, &100, &currency, &None);

        // query cash allowance
        let allowance = client.get_cash_allowance(&project_id, &beneficiary, &currency);
        assert_eq!(allowance.amount, 100);
    }

    #[test]
    fn test_get_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id = String::from_str(&env, "QmVuZWZpY2lhcnk6mBF_cv2JQg0AAAAAAAAAAQ");

        // allocate item allowance
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &10, &None);

        // query item allowance
        let allowance = client.get_item_allowance(&project_id, &beneficiary, &item_id);
        assert_eq!(allowance.quantity, 10);
    }

    #[test]
    fn test_get_all_cash_allowances() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let currency = String::from_str(&env, "ChatsUSDC");

        // allocate cash allowance
        client.allocate_cash_allowance(&ngo, &project_id, &beneficiary, &100, &currency, &None);

        // query all cash allowances
        let allowances = client.get_all_cash_allowances(&project_id);
        assert_eq!(allowances.len(), 1);
    }

    #[test]
    fn test_get_all_item_allowances() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id = String::from_str(&env, "QmVuZWZpY2lhcnk6mBF_cv2JQg0AAAAAAAAAAQ");

        // allocate item allowance
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &10, &None);

        // query all item allowances
        let allowances = client.get_all_item_allowances(&project_id);
        assert_eq!(allowances.len(), 1);
    }

    #[test]
    fn test_get_total_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let currency = String::from_str(&env, "ChatsUSDC");

        // allocate cash allowance
        client.allocate_cash_allowance(&ngo, &project_id, &beneficiary, &100, &currency, &None);

        // query total cash allowance
        let total = client.get_total_cash_allowance(&beneficiary, &vec![&env, project_id.clone()]);
        assert_eq!(total, 100);
    }

    #[test]
    fn test_get_total_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, ngo, beneficiary, _) = setup_contract(&env);

        let project_id = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id = String::from_str(&env, "QmVuZWZpY2lhcnk6mBF_cv2JQg0AAAAAAAAAAQ");

        let project_id_x = String::from_str(&env, "UHJvamVjdDrBfU3xPCxPKgAAAAAAAAAN");
        let item_id_x = String::from_str(&env, "UHJvamVDrBfUsdiuwPCxPKgAAAdsAAAQQWIW");

        // allocate item allowance
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &10, &None);
        client.allocate_item_allowance(&ngo, &project_id_x, &beneficiary, &item_id_x, &10, &None);

        // query total item allowance
        let total = client.get_total_item_allowance(&beneficiary, &vec![&env, project_id.clone()]);
        assert_eq!(total, 20);
    }
}
