#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String, Symbol, Vec};

    use crate::{
        CashAllowance, ItemAllowance, ProjectContract, ProjectContractClient, Roles,
        CASH_ALLOWANCES_KEY, ITEM_ALLOWANCES_KEY,
    };

    const PROJECT_ID: &str = "project1";
    const ITEM_ID: &str = "item1";
    const CURRENCY: &str = "USD";

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
                String::from_str(env, PROJECT_ID),
            );
            let cash_map: Map<(Address, String), CashAllowance> = Map::new(env);
            env.storage().persistent().set(&cash_key, &cash_map);

            let items_key = (
                Symbol::new(env, ITEM_ALLOWANCES_KEY),
                String::from_str(env, PROJECT_ID),
            );
            let items_map: Map<(Address, String), ItemAllowance> = Map::new(env);
            env.storage().persistent().set(&items_key, &items_map);

            // setup roles
            let roles_key = (Symbol::new(env, "roles"), String::from_str(env, PROJECT_ID));
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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

        client.remove_role(
            &owner,
            &project_id,
            &String::from_str(&env, "beneficiary"),
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
        let project_id = String::from_str(&env, PROJECT_ID);

        client.remove_role(
            &attacker,
            &project_id,
            &String::from_str(&env, "beneficiary"),
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
            &String::from_str(&env, PROJECT_ID),
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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);
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
        let project_id = String::from_str(&env, PROJECT_ID);

        // fill up NGO role to capacity (1000)
        for _ in 0..1000 {
            let addr = Address::generate(&env);
            client.add_role(&owner, &project_id, &String::from_str(&env, "ngo"), &addr);
        }

        // adding one more; should panic with RoleLimitExceeded (1013)
        let extra = Address::generate(&env);
        client.add_role(&owner, &project_id, &String::from_str(&env, "ngo"), &extra);
    }

    #[test]
    fn test_cash_allocation_accumulates_and_merges_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_claim_cash_with_non_vendor_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        let project_id = String::from_str(&env, PROJECT_ID);

        // allocate to beneficiary
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None,
        );

        // try to claim to a non-vendor address
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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id = String::from_str(&env, PROJECT_ID);

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
        let project_id1 = String::from_str(&env, "project1");
        let project_id2 = String::from_str(&env, "project2");
        
        // setup roles for project2
        let roles_key = (Symbol::new(&env, "roles"), project_id2.clone());
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
        let _total = client.get_total_cash_allowance(
            &beneficiary,
            &vec![&env, project_id1, project_id2],
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1012)")]
    fn test_get_total_item_overflow_exploit() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _owner, _sa, ngo, beneficiary, _vendor) = setup_contract(&env);
        
        // create two projects
        let project_id1 = String::from_str(&env, "project1");
        let project_id2 = String::from_str(&env, "project2");
        
        // setup roles for project2
        let roles_key = (Symbol::new(&env, "roles"), project_id2.clone());
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
        let _total = client.get_total_item_allowance(
            &beneficiary,
            &vec![&env, project_id1, project_id2],
        );
    }
}
