#[cfg(test)]
mod test {
    use soroban_sdk::{ testutils::Address as _, vec, Address, Env, Map, String, Symbol };
    use soroban_sdk::testutils::Ledger;

    use crate::{
        CashAllowance,
        ItemAllowance,
        ProjectContract,
        ProjectContractClient,
        CASH_ALLOWANCES_KEY,
        ITEM_ALLOWANCES_KEY,
        Roles,
    };

    const PROJECT_ID: &str = "project1";
    const ITEM_ID: &str = "item1";
    const CURRENCY: &str = "USD";

    fn setup_contract(
        env: &Env
    ) -> (ProjectContractClient, Address, Address, Address, Address, Address) {
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
            env.storage().persistent().get(&Symbol::new(&env, "owner")).unwrap()
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
            &None
        );

        let allowances: Map<(Address, String), CashAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            }
        );
        assert_eq!(
            allowances.get((beneficiary.clone(), String::from_str(&env, CURRENCY))).unwrap().amount,
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
            &None
        );
        client.claim_cash_allowance(
            &beneficiary,
            &project_id,
            &String::from_str(&env, CURRENCY),
            &50,
            &None
        );

        let allowances: Map<(Address, String), CashAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id))
                    .unwrap()
            }
        );
        assert_eq!(
            allowances.get((beneficiary, String::from_str(&env, CURRENCY))).unwrap().amount,
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
            &None
        );

        let allowances: Map<(Address, String), ItemAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            }
        );
        assert_eq!(
            allowances
                .get((beneficiary.clone(), String::from_str(&env, ITEM_ID)))
                .unwrap().quantity,
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
            &None
        );
        client.claim_item_allowance(
            &beneficiary,
            &vendor,
            &project_id,
            &String::from_str(&env, ITEM_ID),
            &5
        );

        let allowances: Map<(Address, String), ItemAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            }
        );
        assert_eq!(
            allowances
                .get((beneficiary.clone(), String::from_str(&env, ITEM_ID)))
                .unwrap().quantity,
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
            (beneficiary.clone(), String::from_str(&env, CURRENCY), 100, None),
            (Address::generate(&env), String::from_str(&env, CURRENCY), 200, None)
        ];
        client.allocate_cash_allowances_batch(&ngo, &project_id, &allowances);

        let allowances: Map<(Address, String), CashAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            }
        );
        assert_eq!(
            allowances.get((beneficiary, String::from_str(&env, CURRENCY))).unwrap().amount,
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
            (beneficiary.clone(), String::from_str(&env, ITEM_ID), 10, None),
            (Address::generate(&env), String::from_str(&env, ITEM_ID), 20, None)
        ];
        client.allocate_item_allowances_batch(&ngo, &project_id, &allowances);

        let allowances: Map<(Address, String), ItemAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            }
        );
        assert_eq!(
            allowances.get((beneficiary, String::from_str(&env, ITEM_ID))).unwrap().quantity,
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
            &None
        );
        let new_allowee = Address::generate(&env);
        client.transfer_cash_allowance(
            &beneficiary,
            &project_id,
            &new_allowee,
            &String::from_str(&env, CURRENCY),
            &50
        );

        let allowances: Map<(Address, String), CashAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            }
        );
        assert_eq!(
            allowances.get((new_allowee, String::from_str(&env, CURRENCY))).unwrap().amount,
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
            &None
        );
        let new_allowee = Address::generate(&env);
        client.transfer_item_allowance(
            &beneficiary,
            &project_id,
            &new_allowee,
            &String::from_str(&env, ITEM_ID),
            &5
        );

        let allowances: Map<(Address, String), ItemAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
                    .unwrap()
            }
        );
        assert_eq!(
            allowances.get((new_allowee, String::from_str(&env, ITEM_ID))).unwrap().quantity,
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
            &None
        );
        client.transfer_item_allowance(
            &beneficiary,
            &project_id,
            &vendor,
            &String::from_str(&env, ITEM_ID),
            &5
        );
        client.redeem_item_claims(
            &vendor,
            &Some(project_id.clone()),
            &String::from_str(&env, ITEM_ID),
            &5
        );

        let allowances: Map<(Address, String), ItemAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
                    .unwrap()
            }
        );
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
            &None
        );
        client.transfer_cash_allowance(
            &beneficiary,
            &project_id,
            &vendor,
            &String::from_str(&env, CURRENCY),
            &50
        );
        client.redeem_cash_claims(
            &vendor,
            &Some(project_id.clone()),
            &String::from_str(&env, CURRENCY),
            &50
        );

        let allowances: Map<(Address, String), CashAllowance> = env.as_contract(
            &client.address,
            || {
                env.storage()
                    .persistent()
                    .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id))
                    .unwrap()
            }
        );
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
            &None
        );
        let allowance = client.get_cash_allowance(
            &project_id,
            &beneficiary,
            &String::from_str(&env, CURRENCY)
        );
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
            &None
        );
        let allowance = client.get_item_allowance(
            &project_id,
            &beneficiary,
            &String::from_str(&env, ITEM_ID)
        );
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
            &None
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
            &None
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
            &None
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
            &None
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
            &beneficiary
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
            &beneficiary
        );
    }

    #[test]
    fn test_pause_unpause_contract() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, _, _, _) = setup_contract(&env);

        client.pause_contract(&owner);

        // verify paused state
        let paused: bool = env.as_contract(&client.address, || {
            env.storage().persistent().get(&Symbol::new(&env, "paused")).unwrap_or(false)
        });
        assert!(paused);

        client.unpause_contract(&owner);
        // verify unpaused
        let paused: bool = env.as_contract(&client.address, || {
            env.storage().persistent().get(&Symbol::new(&env, "paused")).unwrap_or(false)
        });
        assert!(!paused);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1005)")]
    fn test_operation_while_paused() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner, _, ngo, beneficiary, _) = setup_contract(&env);
        let project_id = String::from_str(&env, PROJECT_ID);

        client.pause_contract(&owner);
        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &None
        );
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
        let expiry = env.ledger().timestamp() - 500;

        client.allocate_cash_allowance(
            &ngo,
            &project_id,
            &beneficiary,
            &100,
            &String::from_str(&env, CURRENCY),
            &Some(expiry)
        );

        client.claim_cash_allowance(
            &beneficiary,
            &project_id,
            &String::from_str(&env, CURRENCY),
            &50,
            &None
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
            &None
        );
        client.claim_cash_allowance(
            &beneficiary,
            &project_id,
            &String::from_str(&env, CURRENCY),
            &150,
            &None
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
}
