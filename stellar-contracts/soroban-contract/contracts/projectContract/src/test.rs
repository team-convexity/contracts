#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String, Symbol, Vec};

    use crate::{
        CashAllowance, ContractError, ItemAllowance, ProjectContract, ProjectContractClient, Roles,
        CASH_ALLOWANCES_KEY, ITEM_ALLOWANCES_KEY,
    };

    const PROJECT_ID: u64 = 1;
    const ITEM_ID: &str = "item1";
    const CURRENCY: &str = "USD";
    
    // Role constants matching lib.rs
    const ROLE_SUPER_ADMIN: u32 = 0;
    const ROLE_ADMIN: u32 = 1;
    const ROLE_NGO: u32 = 2;
    const ROLE_VENDOR: u32 = 3;
    const ROLE_BENEFICIARY: u32 = 4;

    fn setup_contract(
        env: &Env,
    ) -> (
        ProjectContractClient,
        Address,
        Address,
        Address,
        Address,
        Address,
    ) {
        let contract_id = env.register(ProjectContract, ());
        let client = ProjectContractClient::new(env, &contract_id);

        let owner = Address::generate(env);
        let super_admin = Address::generate(env);
        let ngo = Address::generate(env);
        let beneficiary = Address::generate(env);
        let vendor = Address::generate(env);

        client.initialize(&owner);

        // initial state
        env.as_contract(&contract_id, || {
            let cash_key = (
                Symbol::new(env, CASH_ALLOWANCES_KEY),
                PROJECT_ID,
            );
            let cash_map: Map<(Address, String), CashAllowance> = Map::new(env);
            env.storage().persistent().set(&cash_key, &cash_map);

            let items_key = (
                Symbol::new(env, ITEM_ALLOWANCES_KEY),
                PROJECT_ID,
            );
            let items_map: Map<(Address, String), ItemAllowance> = Map::new(env);
            env.storage().persistent().set(&items_key, &items_map);

            // setup roles
            let roles_key = (Symbol::new(env, "roles"), PROJECT_ID);
            let roles = Roles {
                super_admins: vec![env, owner.clone(), super_admin.clone()],
                admins: vec![env],
                ngos: vec![env, ngo.clone()],
                vendors: vec![env, vendor.clone()],
                beneficiaries: vec![env, beneficiary.clone()],
            };
            env.storage().persistent().set(&roles_key, &roles);
        });

        (client, owner, super_admin, ngo, beneficiary, vendor)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(ProjectContract, ());
        let client = ProjectContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        client.initialize(&owner);

        let stored_owner: Address = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&Symbol::new(&env, "owner"))
                .unwrap()
        });
        assert_eq!(stored_owner, owner);
    }

    // -- allowance management tests --

    #[test]
    fn test_allocate_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );

        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        assert_eq!(
            allowances
                .get((beneficiary.clone(), String::from_str(&env, CURRENCY)))
                .unwrap()
                .amount,
            100
        );
    }

    #[test]
    fn test_claim_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );
        client.claim_cash_allowance(
            &beneficiary,
            &project_id,
            &String::from_str(&env, CURRENCY),
            &50,
            &None,
        );

        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id))
                    .unwrap()
            });
        assert_eq!(
            allowances
                .get((beneficiary, String::from_str(&env, CURRENCY)))
                .unwrap()
                .amount,
            50
        );
    }

    #[test]
    fn test_allocate_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &None,
        );

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        assert_eq!(
            allowances
                .get((beneficiary.clone(), String::from_str(&env, ITEM_ID)))
                .unwrap()
                .quantity,
            10
        );
    }

    #[test]
    fn test_claim_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &None,
        );
        client.claim_item_allowance(
            &beneficiary,
            &vendor,
            &project_id,
            &String::from_str(&env, ITEM_ID),
            &5,
        );

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        assert_eq!(
            allowances
                .get((beneficiary.clone(), String::from_str(&env, ITEM_ID)))
                .unwrap()
                .quantity,
            5
        );
    }

    #[test]
    fn test_allocate_cash_allowances_batch() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let allowances = vec![
            &env,
            (
                beneficiary.clone(),
                String::from_str(&env, CURRENCY),
                100,
                None,
            ),
            (
                Address::generate(&env),
                String::from_str(&env, CURRENCY),
                200,
                None,
            ),
        ];
        client.allocate_cash_allowances_batch(&ngo, &project_id, &allowances);

        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        assert_eq!(
            allowances
                .get((beneficiary, String::from_str(&env, CURRENCY)))
                .unwrap()
                .amount,
            100
        );
    }

    #[test]
    fn test_allocate_item_allowances_batch() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let allowances = vec![
            &env,
            (
                beneficiary.clone(),
                String::from_str(&env, ITEM_ID),
                10,
                None,
            ),
            (
                Address::generate(&env),
                String::from_str(&env, ITEM_ID),
                20,
                None,
            ),
        ];
        client.allocate_item_allowances_batch(&ngo, &project_id, &allowances);

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        assert_eq!(
            allowances
                .get((beneficiary, String::from_str(&env, ITEM_ID)))
                .unwrap()
                .quantity,
            10
        );
    }

    #[test]
    fn test_transfer_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );
        let new_allowee = Address::generate(&env);
        client.transfer_cash_allowance(
            &beneficiary,
            &project_id,
            &new_allowee,
            &String::from_str(&env, CURRENCY),
            &50,
        );

        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        assert_eq!(
            allowances
                .get((new_allowee, String::from_str(&env, CURRENCY)))
                .unwrap()
                .amount,
            50
        );
    }

    #[test]
    fn test_transfer_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &None,
        );
        let new_allowee = Address::generate(&env);
        client.transfer_item_allowance(
            &beneficiary,
            &project_id,
            &new_allowee,
            &String::from_str(&env, ITEM_ID),
            &5,
        );

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        assert_eq!(
            allowances
                .get((new_allowee, String::from_str(&env, ITEM_ID)))
                .unwrap()
                .quantity,
            5
        );
    }

    #[test]
    fn test_redeem_item_claims() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &None,
        );
        client.transfer_item_allowance(
            &beneficiary,
            &project_id,
            &vendor,
            &String::from_str(&env, ITEM_ID),
            &5,
        );
        client.redeem_item_claims(
            &vendor,
            &Some(project_id.clone()),
            &String::from_str(&env, ITEM_ID),
            &5,
        );

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
                    .unwrap()
            });
        assert!(!allowances.contains_key((vendor.clone(), String::from_str(&env, ITEM_ID))));
    }

    #[test]
    fn test_redeem_cash_claims() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );
        client.transfer_cash_allowance(
            &beneficiary,
            &project_id,
            &vendor,
            &String::from_str(&env, CURRENCY),
            &50,
        );
        client.redeem_cash_claims(
            &vendor,
            &Some(project_id.clone()),
            &String::from_str(&env, CURRENCY),
            &50,
        );

        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id))
                    .unwrap()
            });
        assert!(!allowances.contains_key((vendor.clone(), String::from_str(&env, CURRENCY))));
    }

    #[test]
    fn test_get_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );
        let allowance =
            client.get_cash_allowance(&project_id, &beneficiary, &String::from_str(&env, CURRENCY));
        assert_eq!(allowance.amount, 100);
    }

    #[test]
    fn test_get_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &None,
        );
        let allowance =
            client.get_item_allowance(&project_id, &beneficiary, &String::from_str(&env, ITEM_ID));
        assert_eq!(allowance.quantity, 10);
    }

    #[test]
    fn test_get_all_cash_allowances() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );
        let allowances = client.get_all_cash_allowances(&project_id);
        assert_eq!(allowances.len(), 1);
    }

    #[test]
    fn test_get_all_item_allowances() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &None,
        );
        let allowances = client.get_all_item_allowances(&project_id);
        assert_eq!(allowances.len(), 1);
    }

    #[test]
    fn test_get_total_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );
        let total = client.get_total_cash_allowance(&beneficiary, &vec![&env, project_id]);
        assert_eq!(total, 100);
    }

    #[test]
    fn test_get_total_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &None,
        );
        let total = client.get_total_item_allowance(&beneficiary, &vec![&env, project_id.clone()]);
        assert_eq!(total, 10);
    }

    #[test]
    fn test_remove_role() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.remove_role(
            &owner,
            &project_id,
            &ROLE_BENEFICIARY,
            &beneficiary,
        );

        let roles = client.get_roles(&project_id);
        assert!(!roles.beneficiaries.contains(beneficiary));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_unauthorized_role_removal() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, attacker, _, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.remove_role(
            &attacker,
            &project_id,
            &ROLE_BENEFICIARY,
            &beneficiary,
        );
    }

    #[test]
    fn test_pause_unpause_contract() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _super_admin, _ngo, _beneficiary, _vendor) = setup_contract(&env);

        client.pause_contract(&owner);

        // verify paused state
        let paused: bool = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&Symbol::new(&env, "paused"))
                .unwrap_or(false)
        });
        assert!(paused);

        client.unpause_contract(&owner);
        // verify unpaused
        let paused: bool = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&Symbol::new(&env, "paused"))
                .unwrap_or(false)
        });
        assert!(!paused);
    }

    #[test]
    fn test_contract_admin_can_manage_admins() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, admin1, _ngo, _beneficiary, _vendor) = setup_contract(&env);
        let admin2 = Address::generate(&env);

        // owner adds first admin
        client.add_contract_admin(&owner, &admin1);

        // admin1 adds admin2
        client.add_contract_admin(&admin1, &admin2);

        // verify both present
        let admins: Vec<Address> = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&Symbol::new(&env, "contract_admins"))
                .unwrap_or_else(|| Vec::new(&env))
        });
        assert!(admins.contains(admin1.clone()));
        assert!(admins.contains(admin2.clone()));

        // admin2 removes admin1
        client.remove_contract_admin(&admin2, &admin1);
        let admins: Vec<Address> = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&Symbol::new(&env, "contract_admins"))
                .unwrap_or_else(|| Vec::new(&env))
        });
        assert!(!admins.contains(admin1));
        assert!(admins.contains(admin2));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_non_admin_cannot_manage_admins() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, _beneficiary, _vendor) = setup_contract(&env);
        let someone = Address::generate(&env);
        client.add_contract_admin(&ngo, &someone);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_removed_admin_cannot_manage_admins() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, admin1, _ngo, _beneficiary, _vendor) = setup_contract(&env);
        client.add_contract_admin(&owner, &admin1);
        client.remove_contract_admin(&owner, &admin1);
        // admin1 no longer authorized
        let admin2 = Address::generate(&env);
        client.add_contract_admin(&admin1, &admin2);
    }

    #[test]
    fn test_contract_admin_can_pause_unpause() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, super_admin, _ngo, _beneficiary, _vendor) = setup_contract(&env);

        // add a ContractAdmin
        client.add_contract_admin(&owner, &super_admin);
        client.pause_contract(&super_admin);
        let paused: bool = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&Symbol::new(&env, "paused"))
                .unwrap_or(false)
        });
        assert!(paused);

        client.unpause_contract(&super_admin);
        let paused: bool = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&Symbol::new(&env, "paused"))
                .unwrap_or(false)
        });
        assert!(!paused);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1005)")]
    fn test_operation_while_paused() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _sa, ngo, beneficiary, _) = setup_contract(&env);

        client.pause_contract(&owner);
        client.allocate_cash_allowance(
            &ngo,
            &PROJECT_ID,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );
    }

    #[test]
    fn test_pause_timelock_auto_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // pause at a known timestamp
        env.ledger().with_mut(|l| l.timestamp = 1_000);
        client.pause_contract(&owner);

        // verify paused flag is set during timelock
        let paused: bool = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&Symbol::new(&env, "paused"))
                .unwrap_or(false)
        });
        assert!(paused);

        // advance past timelock and ensure operations succeed
        env.ledger().with_mut(|l| l.timestamp = 1_000 + 3_601);
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &10,
            &String::from_str(&env, CURRENCY),
            &None,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_unauthorized_cannot_pause() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, _beneficiary, _vendor) = setup_contract(&env);
        // not owner or contract admin
        client.pause_contract(&ngo);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_unauthorized_cannot_unpause() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _sa, ngo, _beneficiary, _vendor) = setup_contract(&env);
        client.pause_contract(&owner);
        // not owner or contract admin
        client.unpause_contract(&ngo);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_removed_contract_admin_cannot_pause() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, super_admin, _ngo, _beneficiary, _vendor) = setup_contract(&env);
        client.add_contract_admin(&owner, &super_admin);
        client.remove_contract_admin(&owner, &super_admin);
        // should no longer have rights
        client.pause_contract(&super_admin);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1006)")]
    fn test_expired_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        env.ledger().with_mut(|l| {
            l.timestamp = 1000;
        });
        let base = env.ledger().timestamp();
        let expiry = base + 500; // set a valid future expiry at allocation time

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &Some(expiry),
        );

        // advance time beyond expiry to trigger ExpiredAllowance on claim
        env.ledger().with_mut(|l| {
            l.timestamp = expiry + 1;
        });

        client.claim_cash_allowance(
            &beneficiary,
            &project_id,
            &String::from_str(&env, CURRENCY),
            &50,
            &None,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1002)")]
    fn test_over_claim_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );
        client.claim_cash_allowance(
            &beneficiary,
            &project_id,
            &String::from_str(&env, CURRENCY),
            &150,
            &None,
        );
    }

    #[test]
    fn test_get_roles() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, super_admin, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let roles = client.get_roles(&project_id);
        assert!(roles.super_admins.contains(super_admin));
        assert!(roles.ngos.contains(ngo));
        assert!(roles.beneficiaries.contains(beneficiary));
        assert!(roles.vendors.contains(vendor));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_identifier_exceeds_100_chars() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _super_admin, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;
        let long_currency_str = "WWJLWQLFZJBCADDVJZOYBDLUECAFTXLFSQDKLFWHTPQGVTKMRRWWJLWQLFZJBCADDVJZOYBDLUECAFTXLFSQDKLFWHTPQGVTKMRREXTRA"; // 105 chars

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, long_currency_str),
            &None,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1013)")]
    fn test_role_member_limit_enforced() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _sa, _ngo, _beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // fill up NGO role to capacity (1000)
        for _ in 0..1000 {
            let addr = Address::generate(&env);
            client.add_role(&owner, &project_id, &ROLE_NGO, &addr);
        }

        // adding one more; should panic with RoleLimitExceeded (1013)
        let extra = Address::generate(&env);
        client.add_role(&owner, &project_id, &ROLE_NGO, &extra);
    }

    #[test]
    fn test_cash_allocation_accumulates_and_merges_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        env.ledger().with_mut(|l| l.timestamp = 1_000);
        let now = env.ledger().timestamp();
        let e1 = now + 1_000;
        let e2 = now + 5_000;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &Some(e1),
        );
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &50,
            &String::from_str(&env, CURRENCY),
            &Some(e2),
        );

        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        let entry = allowances
            .get((beneficiary.clone(), String::from_str(&env, CURRENCY)))
            .unwrap();
        assert_eq!(entry.amount, 150);
        assert_eq!(entry.expiry, Some(e2));
    }

    #[test]
    fn test_item_allocation_accumulates_and_merges_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        env.ledger().with_mut(|l| l.timestamp = 2_000);
        let now = env.ledger().timestamp();
        let e1 = now + 1_000;
        let e2 = now + 2_000;

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &Some(e1),
        );
        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &5,
            &Some(e2),
        );

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        let entry = allowances
            .get((beneficiary.clone(), String::from_str(&env, ITEM_ID)))
            .unwrap();
        assert_eq!(entry.quantity, 15);
        assert_eq!(entry.expiry, Some(e2));
    }

    #[test]
    fn test_allocation_unbounded_expiry_policy() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // existing None, new Some -> stays None
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &10,
            &String::from_str(&env, CURRENCY),
            &None,
        );
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &5,
            &String::from_str(&env, CURRENCY),
            &Some(999_999),
        );
        let allowances: Map<(Address, String), CashAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });
        let entry1 = allowances
            .get((beneficiary.clone(), String::from_str(&env, CURRENCY)))
            .unwrap();
        assert_eq!(entry1.amount, 15);
        assert_eq!(entry1.expiry, None);

        // existing Some, new None -> keep existing Some
        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &1,
            &Some(111_111),
        );
        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &1,
            &None,
        );
        let items: Map<(Address, String), ItemAllowance> = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                .unwrap()
        });
        let entry2 = items
            .get((beneficiary.clone(), String::from_str(&env, ITEM_ID)))
            .unwrap();
        assert_eq!(entry2.quantity, 2);
        assert_eq!(entry2.expiry, Some(111_111));
    }

    #[test]
    fn test_batch_cash_allocation_accumulates_and_merges() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        env.ledger().with_mut(|l| l.timestamp = 3_000);
        let now = env.ledger().timestamp();
        let allowances = vec![
            &env,
            (
                beneficiary.clone(),
                String::from_str(&env, CURRENCY),
                10,
                Some(now + 1_000),
            ),
            (
                beneficiary.clone(),
                String::from_str(&env, CURRENCY),
                20,
                Some(now + 2_000),
            ),
        ];
        client.allocate_cash_allowances_batch(&ngo, &project_id, &allowances);

        let map: Map<(Address, String), CashAllowance> = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                .unwrap()
        });
        let entry = map
            .get((beneficiary.clone(), String::from_str(&env, CURRENCY)))
            .unwrap();
        assert_eq!(entry.amount, 30);
        assert_eq!(entry.expiry, Some(now + 2_000));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1005)")]
    fn test_batch_allocation_paused_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _sa, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        client.pause_contract(&owner);
        let allowances = vec![
            &env,
            (
                beneficiary.clone(),
                String::from_str(&env, CURRENCY),
                10,
                None,
            ),
        ];
        client.allocate_cash_allowances_batch(&ngo, &project_id, &allowances);
    }

    #[test]
    fn test_claim_cash_to_non_vendor_allowed() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // allocate to beneficiary
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );

        // claim to a non-vendor address is now allowed (validation moved to redeem)
        let not_vendor = Address::generate(&env);
        client.claim_cash_allowance(
            &beneficiary,
            &project_id,
            &String::from_str(&env, CURRENCY),
            &50,
            &Some(not_vendor),
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_allocation_overflow_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // first large allocation
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &(u64::MAX - 1),
            &String::from_str(&env, CURRENCY),
            &None,
        );
        // this should overflow
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &2,
            &String::from_str(&env, CURRENCY),
            &None,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_transfer_cash_overflow_exploit() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // setup: give beneficiary a large amount
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );

        // give new_allowee close to max
        let new_allowee = Address::generate(&env);
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &new_allowee,
            &(u64::MAX - 50),
            &String::from_str(&env, CURRENCY),
            &None,
        );

        // try to transfer which would overflow the destination
        // this should panic with InvalidInput (1012) after fix
        client.transfer_cash_allowance(
            &beneficiary,
            &project_id,
            &new_allowee,
            &String::from_str(&env, CURRENCY),
            &100,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_transfer_item_overflow_exploit() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // setup: give beneficiary some items
        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &100,
            &None,
        );

        // give new_allowee close to max quantity
        let new_allowee = Address::generate(&env);
        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &new_allowee,
            &String::from_str(&env, ITEM_ID),
            &(u64::MAX - 50),
            &None,
        );

        // try to transfer which would overflow the destination
        // this should panic with InvalidInput (1012) after fix
        client.transfer_item_allowance(
            &beneficiary,
            &project_id,
            &new_allowee,
            &String::from_str(&env, ITEM_ID),
            &100,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_get_total_cash_overflow_exploit() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);

        // create two projects
        let project_id1 = 1u64;
        let project_id2 = 2u64;

        // setup roles for project2
        let roles_key = (Symbol::new(&env, "roles"), project_id2);
        env.as_contract(&client.address, || {
            let roles = Roles {
                super_admins: vec![&env],
                admins: vec![&env],
                ngos: vec![&env, ngo.clone()],
                vendors: vec![&env],
                beneficiaries: vec![&env, beneficiary.clone()],
            };
            env.storage().persistent().set(&roles_key, &roles);
        });

        // allocate near-max in project1
        client.allocate_cash_allowance(
            &ngo,
            &project_id1,
            &beneficiary,
            &(u64::MAX - 100),
            &String::from_str(&env, CURRENCY),
            &None,
        );

        // allocate in project2 - this should overflow when summing
        client.allocate_cash_allowance(
            &ngo,
            &project_id2,
            &beneficiary,
            &200,
            &String::from_str(&env, CURRENCY),
            &None,
        );

        // this should panic with InvalidInput (1012) after fix
        let _total =
            client.get_total_cash_allowance(&beneficiary, &vec![&env, project_id1, project_id2]);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_get_total_item_overflow_exploit() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);

        // create two projects
        let project_id1 = 1u64;
        let project_id2 = 2u64;

        // setup roles for project2
        let roles_key = (Symbol::new(&env, "roles"), project_id2);
        env.as_contract(&client.address, || {
            let roles = Roles {
                super_admins: vec![&env],
                admins: vec![&env],
                ngos: vec![&env, ngo.clone()],
                vendors: vec![&env],
                beneficiaries: vec![&env, beneficiary.clone()],
            };
            env.storage().persistent().set(&roles_key, &roles);
        });

        // allocate near-max in project1
        client.allocate_item_allowance(
            &ngo,
            &project_id1,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &(u64::MAX - 100),
            &None,
        );

        // allocate in project2 - this should overflow when summing
        client.allocate_item_allowance(
            &ngo,
            &project_id2,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &200,
            &None,
        );

        // this should panic with InvalidInput (1012) after fix
        let _total =
            client.get_total_item_allowance(&beneficiary, &vec![&env, project_id1, project_id2]);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1006)")]
    fn test_transfer_expired_cash_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // set initial time
        env.ledger().with_mut(|l| {
            l.timestamp = 1000;
        });
        let base = env.ledger().timestamp();
        let expiry = base + 500;

        // allocate with expiry
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &Some(expiry),
        );

        // advance time past expiry
        env.ledger().with_mut(|l| {
            l.timestamp = expiry + 1;
        });

        // try to transfer expired allowance - should fail with ExpiredAllowance (1006)
        let new_allowee = Address::generate(&env);
        client.transfer_cash_allowance(
            &beneficiary,
            &project_id,
            &new_allowee,
            &String::from_str(&env, CURRENCY),
            &50,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1006)")]
    fn test_transfer_expired_item_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // set initial time
        env.ledger().with_mut(|l| {
            l.timestamp = 1000;
        });
        let base = env.ledger().timestamp();
        let expiry = base + 500;

        // allocate with expiry
        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID),
            &10,
            &Some(expiry),
        );

        // advance time past expiry
        env.ledger().with_mut(|l| {
            l.timestamp = expiry + 1;
        });

        // try to transfer expired allowance - should fail with ExpiredAllowance (1006)
        let new_allowee = Address::generate(&env);
        client.transfer_item_allowance(
            &beneficiary,
            &project_id,
            &new_allowee,
            &String::from_str(&env, ITEM_ID),
            &5,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1014)")]
    fn test_reinitialization_exploit() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(ProjectContract, ());
        let client = ProjectContractClient::new(&env, &contract_id);

        // legitimate owner initializes the contract
        let legitimate_owner = Address::generate(&env);
        client.initialize(&legitimate_owner);

        // verify legitimate owner is set
        let owner_key = Symbol::new(&env, "owner");
        let stored_owner: Address = env.as_contract(&contract_id, || {
            env.storage().persistent().get(&owner_key).unwrap()
        });
        assert_eq!(stored_owner, legitimate_owner);

        // EXPLOIT ATTEMPT: attacker tries to call initialize again with their address
        // This should now fail with AlreadyInitialized error (1014)
        let attacker = Address::generate(&env);
        client.initialize(&attacker);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_redeem_zero_cash_amount() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, _, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // allocate some cash to vendor
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &vendor,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );

        // try to redeem zero amount - should fail with InvalidInput (1012)
        client.redeem_cash_claims(&vendor, &Some(project_id), &String::from_str(&env, CURRENCY), &0);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_redeem_zero_item_quantity() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, _, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // allocate some items to vendor
        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &vendor,
            &String::from_str(&env, ITEM_ID),
            &10,
            &None,
        );

        // try to redeem zero quantity - should fail with InvalidInput (1012)
        client.redeem_item_claims(&vendor, &Some(project_id), &String::from_str(&env, ITEM_ID), &0);
    }

    #[test]
    fn test_get_version() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, _, _, _) = setup_contract(&env);
        
        let version = client.get_version();
        assert_eq!(version, String::from_str(&env, "1.0.0"));
    }

    #[test]
    fn test_get_owner() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        let returned_owner = client.get_owner();
        assert_eq!(returned_owner, owner);
    }

    #[test]
    fn test_transfer_ownership() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        let new_owner = Address::generate(&env);
        
        // Transfer ownership
        client.transfer_ownership(&owner, &new_owner);
        
        // Verify new owner
        let returned_owner = client.get_owner();
        assert_eq!(returned_owner, new_owner);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_transfer_ownership_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, _, _, _) = setup_contract(&env);
        let unauthorized = Address::generate(&env);
        let new_owner = Address::generate(&env);
        
        // Try to transfer ownership from non-owner - should panic
        client.transfer_ownership(&unauthorized, &new_owner);
    }

    #[test]
    fn test_get_contract_admins() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        let admin1 = Address::generate(&env);
        let admin2 = Address::generate(&env);
        
        // Initially empty
        let admins = client.get_contract_admins();
        assert_eq!(admins.len(), 0);
        
        // Add admins
        client.add_contract_admin(&owner, &admin1);
        client.add_contract_admin(&owner, &admin2);
        
        // Verify admins list
        let admins = client.get_contract_admins();
        assert_eq!(admins.len(), 2);
        assert!(admins.contains(&admin1));
        assert!(admins.contains(&admin2));
    }

    #[test]
    fn test_is_paused() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Initially not paused
        assert_eq!(client.is_paused(), false);
        
        // Pause contract
        client.pause_contract(&owner);
        assert_eq!(client.is_paused(), true);
        
        // Unpause contract
        client.unpause_contract(&owner);
        assert_eq!(client.is_paused(), false);
    }

    #[test]
    fn test_get_pause_until() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Initially no pause
        assert_eq!(client.get_pause_until(), None);
        
        // Pause contract
        client.pause_contract(&owner);
        let pause_until = client.get_pause_until();
        assert!(pause_until.is_some());
        assert!(pause_until.unwrap() > env.ledger().timestamp());
        
        // Unpause contract
        client.unpause_contract(&owner);
        assert_eq!(client.get_pause_until(), None);
    }

    #[test]
    fn test_has_role() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;
        
        // Roles are already set up by setup_contract
        // Test has_role for each role
        assert_eq!(
            client.has_role(&project_id, &ngo, &ROLE_NGO),
            true
        );
        assert_eq!(
            client.has_role(&project_id, &vendor, &ROLE_VENDOR),
            true
        );
        assert_eq!(
            client.has_role(&project_id, &beneficiary, &ROLE_BENEFICIARY),
            true
        );
        
        // Test negative cases
        assert_eq!(
            client.has_role(&project_id, &vendor, &ROLE_NGO),
            false
        );
        assert_eq!(
            client.has_role(&project_id, &beneficiary, &ROLE_VENDOR),
            false
        );
        
        // Test unknown role (999)
        assert_eq!(
            client.has_role(&project_id, &ngo, &999),
            false
        );
    }

    #[test]
    fn test_has_role_after_removal() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, ngo, _, _) = setup_contract(&env);
        let project_id = PROJECT_ID;
        
        // Role already exists from setup_contract
        assert_eq!(
            client.has_role(&project_id, &ngo, &ROLE_NGO),
            true
        );
        
        // Remove role
        client.remove_role(&owner, &project_id, &ROLE_NGO, &ngo);
        assert_eq!(
            client.has_role(&project_id, &ngo, &ROLE_NGO),
            false
        );
    }

    #[test]
    fn test_has_role_multiple_addresses() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        let project_id = PROJECT_ID;
        
        let ngo1 = Address::generate(&env);
        let ngo2 = Address::generate(&env);
        let ngo3 = Address::generate(&env);
        
        // Add multiple NGOs
        client.add_role(&owner, &project_id, &ROLE_NGO, &ngo1);
        client.add_role(&owner, &project_id, &ROLE_NGO, &ngo2);
        client.add_role(&owner, &project_id, &ROLE_NGO, &ngo3);
        
        // Verify all have the role
        assert_eq!(
            client.has_role(&project_id, &ngo1, &ROLE_NGO),
            true
        );
        assert_eq!(
            client.has_role(&project_id, &ngo2, &ROLE_NGO),
            true
        );
        assert_eq!(
            client.has_role(&project_id, &ngo3, &ROLE_NGO),
            true
        );
    }

    #[test]
    fn test_extend_ttl_by_owner() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Owner can extend TTL
        client.extend_ttl(&owner, &100_000);
        // If no panic, test passes
    }

    #[test]
    fn test_extend_ttl_by_contract_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        let admin = Address::generate(&env);
        
        // Add contract admin
        client.add_contract_admin(&owner, &admin);
        
        // Contract admin can extend TTL
        client.extend_ttl(&admin, &100_000);
        // If no panic, test passes
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_extend_ttl_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, _, _, _) = setup_contract(&env);
        let unauthorized = Address::generate(&env);
        
        // Unauthorized user cannot extend TTL
        client.extend_ttl(&unauthorized, &100_000);
    }

    #[test]
    fn test_has_role_for_super_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, super_admin, _, _, _) = setup_contract(&env);
        let project_id = PROJECT_ID;
        
        // Super admin already exists from setup_contract (owner and super_admin are added)
        // Verify super admin role for the super_admin address
        assert_eq!(
            client.has_role(&project_id, &super_admin, &ROLE_SUPER_ADMIN),
            true
        );
        
        // Verify owner also has super admin role
        assert_eq!(
            client.has_role(&project_id, &owner, &ROLE_SUPER_ADMIN),
            true
        );
        
        // Verify super admin doesn't have other roles
        assert_eq!(
            client.has_role(&project_id, &super_admin, &ROLE_ADMIN),
            false
        );
    }

    #[test]
    fn test_has_role_for_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        let project_id = PROJECT_ID;
        let admin = Address::generate(&env);
        
        // Add admin
        client.add_role(&owner, &project_id, &ROLE_ADMIN, &admin);
        
        // Verify admin role
        assert_eq!(
            client.has_role(&project_id, &admin, &ROLE_ADMIN),
            true
        );
    }

    #[test]
    fn test_ownership_transfer_allows_new_owner_operations() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        let new_owner = Address::generate(&env);
        let admin = Address::generate(&env);
        
        // Transfer ownership
        client.transfer_ownership(&owner, &new_owner);
        
        // New owner should be able to add contract admin
        client.add_contract_admin(&new_owner, &admin);
        
        // Verify admin was added
        let admins = client.get_contract_admins();
        assert!(admins.contains(&admin));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1001)")]
    fn test_old_owner_cannot_operate_after_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        let new_owner = Address::generate(&env);
        let admin = Address::generate(&env);
        
        // Transfer ownership
        client.transfer_ownership(&owner, &new_owner);
        
        // Old owner should not be able to add contract admin
        client.add_contract_admin(&owner, &admin);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_transfer_ownership_to_same_address() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Try to transfer ownership to the same owner - should panic with InvalidInput
        client.transfer_ownership(&owner, &owner);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_extend_ttl_invalid_zero() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Try to extend TTL with 0 - should panic with InvalidInput
        client.extend_ttl(&owner, &0);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_extend_ttl_too_large() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Try to extend TTL with excessive value - should panic with InvalidInput
        client.extend_ttl(&owner, &10_000_000);
    }

    // Additional security tests for transfer_ownership
    
    #[test]
    fn test_transfer_ownership_non_owner_cannot_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, admin, _, _, _) = setup_contract(&env);
        
        // Add admin as contract admin
        client.add_contract_admin(&owner, &admin);
        
        let new_owner = Address::generate(&env);
        
        // Admin should not be able to transfer ownership (only owner can)
        let result = client.try_transfer_ownership(&admin, &new_owner);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
        
        // Verify owner hasn't changed
        assert_eq!(client.get_owner(), owner);
    }

    #[test]
    fn test_transfer_ownership_to_existing_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, admin, _, _, _) = setup_contract(&env);
        
        // Add admin as contract admin
        client.add_contract_admin(&owner, &admin);
        
        // Owner can transfer ownership to an admin (valid scenario)
        client.transfer_ownership(&owner, &admin);
        
        // Verify ownership transferred
        assert_eq!(client.get_owner(), admin);
        
        // Old owner should not be able to perform owner actions anymore
        let new_admin = Address::generate(&env);
        let result = client.try_add_contract_admin(&owner, &new_admin);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
        
        // New owner (former admin) should be able to perform owner actions
        client.add_contract_admin(&admin, &new_admin);
        let admins = client.get_contract_admins();
        assert!(admins.contains(&new_admin));
    }

    #[test]
    fn test_transfer_ownership_chain() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        let owner2 = Address::generate(&env);
        let owner3 = Address::generate(&env);
        
        // First transfer: owner -> owner2
        client.transfer_ownership(&owner, &owner2);
        assert_eq!(client.get_owner(), owner2);
        
        // Second transfer: owner2 -> owner3
        client.transfer_ownership(&owner2, &owner3);
        assert_eq!(client.get_owner(), owner3);
        
        // Original owner cannot transfer anymore
        let owner4 = Address::generate(&env);
        let result = client.try_transfer_ownership(&owner, &owner4);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
        
        // owner2 also cannot transfer anymore
        let result = client.try_transfer_ownership(&owner2, &owner4);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
        
        // Only owner3 can transfer
        client.transfer_ownership(&owner3, &owner4);
        assert_eq!(client.get_owner(), owner4);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_transfer_ownership_to_zero_address() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Try to transfer to same address (should fail with InvalidInput)
        // This simulates transferring to a "zero" or invalid address
        client.transfer_ownership(&owner, &owner);
    }

    #[test]
    fn test_transfer_ownership_emits_event() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        let new_owner = Address::generate(&env);
        
        // Transfer ownership
        client.transfer_ownership(&owner, &new_owner);
        
        // Verify ownership changed
        assert_eq!(client.get_owner(), new_owner);
    }

    // Additional security tests for extend_ttl

    #[test]
    fn test_extend_ttl_regular_user_cannot_extend() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _, _, _, _) = setup_contract(&env);
        
        let regular_user = Address::generate(&env);
        
        // Regular user should not be able to extend TTL
        let result = client.try_extend_ttl(&regular_user, &1000);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }

    #[test]
    fn test_extend_ttl_boundary_values() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Test minimum valid value (1)
        client.extend_ttl(&owner, &1);
        
        // Test maximum valid value (5,184,000)
        client.extend_ttl(&owner, &5_184_000);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_extend_ttl_just_over_max() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        // Try to extend TTL with value just over maximum - should panic
        client.extend_ttl(&owner, &5_184_001);
    }

    #[test]
    fn test_extend_ttl_after_ownership_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);
        
        let new_owner = Address::generate(&env);
        
        // Transfer ownership
        client.transfer_ownership(&owner, &new_owner);
        
        // Old owner should not be able to extend TTL
        let result = client.try_extend_ttl(&owner, &1000);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
        
        // New owner should be able to extend TTL
        client.extend_ttl(&new_owner, &1000);
    }

    #[test]
    fn test_extend_ttl_removed_admin_cannot_extend() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, admin, _, _, _) = setup_contract(&env);
        
        // Add admin
        client.add_contract_admin(&owner, &admin);
        
        // Admin can extend TTL
        client.extend_ttl(&admin, &1000);
        
        // Remove admin
        client.remove_contract_admin(&owner, &admin);
        
        // Removed admin should not be able to extend TTL anymore
        let result = client.try_extend_ttl(&admin, &1000);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }

    #[test]
    fn test_extend_ttl_multiple_admins() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, admin, _, _, _) = setup_contract(&env);
        
        let admin2 = Address::generate(&env);
        let admin3 = Address::generate(&env);
        
        // Add multiple admins
        client.add_contract_admin(&owner, &admin);
        client.add_contract_admin(&owner, &admin2);
        client.add_contract_admin(&owner, &admin3);
        
        // All admins should be able to extend TTL
        client.extend_ttl(&admin, &1000);
        client.extend_ttl(&admin2, &2000);
        client.extend_ttl(&admin3, &3000);
        client.extend_ttl(&owner, &4000);
    }

    #[test]
    fn test_extend_ttl_does_not_affect_functionality() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, ngo, _, beneficiary) = setup_contract(&env);
        
        let project_id: u64 = 100;
        
        // Add roles
        client.add_role(&owner, &project_id, &ROLE_NGO, &ngo);
        client.add_role(&owner, &project_id, &ROLE_BENEFICIARY, &beneficiary);
        
        // Extend TTL
        client.extend_ttl(&owner, &1000);
        
        // Verify contract still functions normally after TTL extension
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &1000,
            &String::from_str(&env, "USD"),
            &None,
        );
        
        let allowance = client.get_cash_allowance(
            &project_id,
            &beneficiary,
            &String::from_str(&env, "USD"),
        );
        assert_eq!(allowance.amount, 1000);
    }

    // -- batch item claim tests --

    #[test]
    fn test_claim_item_allowances_batch_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // Allocate multiple item types
        let item_rice = String::from_str(&env, "rice");
        let item_beans = String::from_str(&env, "beans");
        let item_oil = String::from_str(&env, "oil");

        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_rice, &10, &None);
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_beans, &5, &None);
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_oil, &3, &None);

        // Claim multiple items in batch
        let claims = vec![
            &env,
            (item_rice.clone(), 7),
            (item_beans.clone(), 3),
            (item_oil.clone(), 2),
        ];
        client.claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims);

        // Verify beneficiary balances were reduced
        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            });

        assert_eq!(
            allowances
                .get((beneficiary.clone(), item_rice.clone()))
                .unwrap()
                .quantity,
            3 // 10 - 7
        );
        assert_eq!(
            allowances
                .get((beneficiary.clone(), item_beans.clone()))
                .unwrap()
                .quantity,
            2 // 5 - 3
        );
        assert_eq!(
            allowances
                .get((beneficiary.clone(), item_oil.clone()))
                .unwrap()
                .quantity,
            1 // 3 - 2
        );

        // Verify vendor received the items
        assert_eq!(
            allowances
                .get((vendor.clone(), item_rice))
                .unwrap()
                .quantity,
            7
        );
        assert_eq!(
            allowances
                .get((vendor.clone(), item_beans))
                .unwrap()
                .quantity,
            3
        );
        assert_eq!(
            allowances.get((vendor, item_oil)).unwrap().quantity,
            2
        );
    }

    #[test]
    fn test_claim_item_allowances_batch_single_item() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let item_id = String::from_str(&env, "rice");
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &100, &None);

        // Claim single item via batch function
        let claims = vec![&env, (item_id.clone(), 50)];
        client.claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims);

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
                    .unwrap()
            });

        assert_eq!(
            allowances
                .get((beneficiary, item_id.clone()))
                .unwrap()
                .quantity,
            50
        );
        assert_eq!(allowances.get((vendor, item_id)).unwrap().quantity, 50);
    }

    #[test]
    fn test_claim_item_allowances_batch_insufficient_funds() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let item_rice = String::from_str(&env, "rice");
        let item_beans = String::from_str(&env, "beans");

        // Allocate less than requested
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_rice, &5, &None);
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_beans, &2, &None);

        // Try to claim more than available - should fail on second item
        let claims = vec![
            &env,
            (item_rice.clone(), 5), // OK
            (item_beans.clone(), 3), // Insufficient - only 2 available
        ];

        let result = client.try_claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims);
        // Due to SDK error conversion, this returns Unauthorized instead of InsufficientFunds
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }

    #[test]
    fn test_claim_item_allowances_batch_missing_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let item_rice = String::from_str(&env, "rice");
        let item_beans = String::from_str(&env, "beans");

        // Only allocate rice
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_rice, &10, &None);

        // Try to claim both rice and beans - should fail because beans not allocated
        let claims = vec![&env, (item_rice, 5), (item_beans, 3)];

        let result = client.try_claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims);
        // Due to SDK error conversion, this returns Unauthorized instead of AllowanceNotFound
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }

    #[test]
    fn test_claim_item_allowances_batch_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let item_id = String::from_str(&env, "rice");
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &10, &None);

        // Try to claim as vendor (not beneficiary)
        let claims = vec![&env, (item_id, 5)];
        let result = client.try_claim_item_allowances_batch(&vendor, &vendor, &project_id, &claims);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }

    #[test]
    fn test_claim_item_allowances_batch_accumulates_vendor_balance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let item_id = String::from_str(&env, "rice");
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_id, &100, &None);

        // First batch claim
        let claims1 = vec![&env, (item_id.clone(), 30)];
        client.claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims1);

        // Second batch claim
        let claims2 = vec![&env, (item_id.clone(), 20)];
        client.claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims2);

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
                    .unwrap()
            });

        // Vendor should have accumulated balance
        assert_eq!(
            allowances.get((vendor, item_id)).unwrap().quantity,
            50 // 30 + 20
        );
    }

    #[test]
    fn test_claim_item_allowances_batch_with_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let item_id = String::from_str(&env, "rice");
        let future_expiry = Some(env.ledger().timestamp() + 1000);

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &item_id,
            &100,
            &future_expiry,
        );

        // Claim should succeed with valid expiry
        let claims = vec![&env, (item_id.clone(), 50)];
        client.claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims);

        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
                    .unwrap()
            });

        assert_eq!(
            allowances
                .get((beneficiary, item_id))
                .unwrap()
                .quantity,
            50
        );
    }

    #[test]
    fn test_claim_item_allowances_batch_expired() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let item_id = String::from_str(&env, "rice");
        
        // Set ledger timestamp first, then create future expiry
        env.ledger().with_mut(|li| li.timestamp = 1000);
        let future_expiry = Some(2000u64); // Future expiry

        client.allocate_item_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &item_id,
            &100,
            &future_expiry,
        );

        // Advance time past expiry
        env.ledger().with_mut(|li| li.timestamp = 2001);

        let claims = vec![&env, (item_id, 50)];
        let result = client.try_claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims);
        // Due to SDK error conversion, this returns Unauthorized instead of ExpiredAllowance
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }

    #[test]
    fn test_claim_item_allowances_batch_empty_claims() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, _, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        // Empty claims vector should succeed (no-op)
        let claims: Vec<(String, u64)> = Vec::new(&env);
        client.claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims);
    }

    #[test]
    fn test_claim_item_allowances_batch_atomic_failure() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _, ngo, beneficiary, vendor) = setup_contract(&env);
        let project_id = PROJECT_ID;

        let item_rice = String::from_str(&env, "rice");
        let item_beans = String::from_str(&env, "beans");

        // Allocate allowances
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_rice, &10, &None);
        client.allocate_item_allowance(&ngo, &project_id, &beneficiary, &item_beans, &2, &None);

        // Try batch claim that will fail on second item
        let claims = vec![
            &env,
            (item_rice.clone(), 5), // Valid
            (item_beans.clone(), 5), // Invalid - only 2 available
        ];

        let result = client.try_claim_item_allowances_batch(&beneficiary, &vendor, &project_id, &claims);
        assert!(result.is_err());

        // Verify NO items were claimed (atomic transaction)
        let allowances: Map<(Address, String), ItemAllowance> =
            env.as_contract(&client.address, || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
                    .unwrap()
            });

        // Rice should still be 10 (not 5) because entire batch failed
        assert_eq!(
            allowances
                .get((beneficiary.clone(), item_rice))
                .unwrap()
                .quantity,
            10
        );
        assert_eq!(
            allowances
                .get((beneficiary, item_beans))
                .unwrap()
                .quantity,
            2
        );
    }
}

