/*
 * Project: CHATS:ProjectContract
 * Description: A contract for managing cash and item allowances for NGOs, vendors, and beneficiaries.
 *
 * Author: codes@abdull.dev <Abdulrasheed Ibrahim>
 *
 * License: GNU AFFERO GENERAL PUBLIC LICENSE 3.0
 */

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, IntoVal, Map, String, Symbol,
    TryFromVal, Val, Vec,
};

#[contract]
pub struct ProjectContract;

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct CashAllowance {
    pub amount: u64,
    pub expiry: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct ItemAllowance {
    pub quantity: u64,
    pub expiry: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct Roles {
    pub super_admins: Vec<Address>,
    pub admins: Vec<Address>,
    pub ngos: Vec<Address>,
    pub vendors: Vec<Address>,
    pub beneficiaries: Vec<Address>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum ContractError {
    Unauthorized,
    InsufficientFunds,
    AllowanceNotFound,
    InvalidRole,
    ContractPaused,
    ExpiredAllowance,
    StorageCorruption,
    InvalidProjectId,
    RoleAlreadyExists,
    InvalidInput,
    RoleLimitExceeded,
    AlreadyInitialized,
}

impl From<soroban_sdk::Error> for ContractError {
    fn from(_err: soroban_sdk::Error) -> Self {
        ContractError::Unauthorized
    }
}

impl TryFrom<ContractError> for soroban_sdk::Error {
    type Error = soroban_sdk::Error;

    fn try_from(err: ContractError) -> Result<Self, Self::Error> {
        match err {
            ContractError::Unauthorized => Ok(soroban_sdk::Error::from_contract_error(1001)),
            ContractError::InsufficientFunds => Ok(soroban_sdk::Error::from_contract_error(1002)),
            ContractError::AllowanceNotFound => Ok(soroban_sdk::Error::from_contract_error(1003)),
            ContractError::InvalidRole => Ok(soroban_sdk::Error::from_contract_error(1004)),
            ContractError::ContractPaused => Ok(soroban_sdk::Error::from_contract_error(1005)),
            ContractError::ExpiredAllowance => Ok(soroban_sdk::Error::from_contract_error(1006)),
            ContractError::StorageCorruption => Ok(soroban_sdk::Error::from_contract_error(1007)),
            ContractError::InvalidProjectId => Ok(soroban_sdk::Error::from_contract_error(1008)),
            ContractError::RoleAlreadyExists => Ok(soroban_sdk::Error::from_contract_error(1009)),
            ContractError::InvalidInput => Ok(soroban_sdk::Error::from_contract_error(1012)),
            ContractError::RoleLimitExceeded => Ok(soroban_sdk::Error::from_contract_error(1013)),
            ContractError::AlreadyInitialized => Ok(soroban_sdk::Error::from_contract_error(1014)),
        }
    }
}

impl From<&ContractError> for soroban_sdk::Error {
    fn from(err: &ContractError) -> Self {
        match err {
            ContractError::Unauthorized => soroban_sdk::Error::from_contract_error(1001),
            ContractError::InsufficientFunds => soroban_sdk::Error::from_contract_error(1002),
            ContractError::AllowanceNotFound => soroban_sdk::Error::from_contract_error(1003),
            ContractError::InvalidRole => soroban_sdk::Error::from_contract_error(1004),
            ContractError::ContractPaused => soroban_sdk::Error::from_contract_error(1005),
            ContractError::ExpiredAllowance => soroban_sdk::Error::from_contract_error(1006),
            ContractError::StorageCorruption => soroban_sdk::Error::from_contract_error(1007),
            ContractError::InvalidProjectId => soroban_sdk::Error::from_contract_error(1008),
            ContractError::RoleAlreadyExists => soroban_sdk::Error::from_contract_error(1009),
            ContractError::InvalidInput => soroban_sdk::Error::from_contract_error(1012),
            ContractError::RoleLimitExceeded => soroban_sdk::Error::from_contract_error(1013),
            ContractError::AlreadyInitialized => soroban_sdk::Error::from_contract_error(1014),
        }
    }
}

// -- storage keys --
const OWNER_KEY: &str = "owner";
const ROLES_KEY: &str = "roles";
const PAUSED_KEY: &str = "paused";
const VERSION_KEY: &str = "version";
const CASH_ALLOWANCES_KEY: &str = "cash";
const ITEM_ALLOWANCES_KEY: &str = "items";
const PAUSE_UNTIL_KEY: &str = "pause_until";
const CONTRACT_ADMINS_KEY: &str = "contract_admins";

// role constants
const ROLE_SUPER_ADMIN: u32 = 0;
const ROLE_ADMIN: u32 = 1;
const ROLE_NGO: u32 = 2;
const ROLE_VENDOR: u32 = 3;
const ROLE_BENEFICIARY: u32 = 4;

// validation limits
const MAX_ID_LEN: u32 = 100; // max length for project_id, currency, item_id. Check Project service's README for more info.
const MAX_BATCH_LEN: u32 = 200; // cap batch sizes to avoid excessive compute
const MAX_EXPIRY_AHEAD_SECS: u64 = 31_536_000; // ~365 days
const PAUSE_MAX_SECS: u64 = 3600; // max pause window
const MAX_ROLE_MEMBERS: u32 = 1000; // per-role membership cap per project (and for global contract admins)

// -- events --
#[contracttype]
pub struct RoleAssigned {
    pub project_id: String,
    pub role: u32,
    pub assignee: Address,
}

#[contracttype]
pub struct RoleRemovedEvent {
    pub project_id: String,
    pub role: u32,
    pub member: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceAllocated {
    pub project_id: String,
    pub allowee: Address,
    pub currency_or_item: String,
    pub amount: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceClaimed {
    pub project_id: String,
    pub allowee: Address,
    pub amount: u64,
}

#[contracttype]
pub struct ContractPausedEvent {
    pub by: Address,
    pub until: u64,
}

#[contracttype]
pub struct ContractUnpausedEvent {
    pub by: Address,
}

#[contracttype]
pub struct ContractAdminAddedEvent {
    pub by: Address,
    pub added: Address,
}

#[contracttype]
pub struct ContractAdminRemovedEvent {
    pub by: Address,
    pub removed: Address,
}

#[contractimpl]
impl ProjectContract {
    /// Initializes the contract owner.
    ///
    /// # Arguments
    /// - `owner`: address to set as the owner.
    ///
    /// # Errors
    /// - `ContractError::Unauthorized`
    /// - `ContractError::AlreadyInitialized`
    pub fn initialize(env: Env, owner: Address) -> Result<(), ContractError> {
        owner.require_auth();

        // Check if already initialized
        let owner_key = Symbol::new(&env, OWNER_KEY);
        if env.storage().persistent().has(&owner_key) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage().persistent().set(&owner_key, &owner);
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, VERSION_KEY), &1u32);

        Ok(())
    }

    // -- role management --
    /// Adds a role member to a project.
    ///
    /// # Arguments
    /// - `caller`: owner address.
    /// - `project_id`: project identifier.
    /// - `role`: 0=super_admin, 1=admin, 2=ngo, 3=vendor, 4=beneficiary.
    /// - `new_member`: address to add.
    ///
    /// # Errors
    /// - `ContractError::InvalidRole`
    /// - `ContractError::RoleAlreadyExists`
    /// - `ContractError::RoleLimitExceeded`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn add_role(
        env: Env,
        caller: Address,
        project_id: String,
        role: u32,
        new_member: Address,
    ) -> Result<(), ContractError> {
        require_owner(&env, &caller)?;
        is_valid_identifier(&env, &project_id)?;

        let mut roles = get_roles(&env, &project_id);

        let role_vec = match role {
            ROLE_SUPER_ADMIN => &mut roles.super_admins,
            ROLE_ADMIN => &mut roles.admins,
            ROLE_NGO => &mut roles.ngos,
            ROLE_VENDOR => &mut roles.vendors,
            ROLE_BENEFICIARY => &mut roles.beneficiaries,
            _ => return Err(ContractError::InvalidRole),
        };

        check_role_limits(role_vec, &new_member)?;
        role_vec.push_back(new_member.clone());

        set_roles(&env, &project_id, &roles);
        env.events().publish(
            (RoleAssigned {
                project_id: project_id.clone(),
                role,
                assignee: new_member,
            },),
            (),
        );
        Ok(())
    }

    /// Removes a role member from a project.
    ///
    /// # Arguments
    /// - `caller`: owner address.
    /// - `project_id`: project identifier.
    /// - `role`: role integer (0=super_admin, 1=admin, 2=ngo, 3=vendor, 4=beneficiary).
    /// - `member_to_remove`: address to remove.
    ///
    /// # Errors
    /// - `ContractError::InvalidRole`
    /// - `ContractError::Unauthorized`
    pub fn remove_role(
        env: Env,
        caller: Address,
        project_id: String,
        role: u32,
        member_to_remove: Address,
    ) -> Result<(), ContractError> {
        require_owner(&env, &caller)?;
        is_valid_identifier(&env, &project_id)?;
        let mut roles = get_roles(&env, &project_id);

        let role_vec = match role {
            ROLE_SUPER_ADMIN => &mut roles.super_admins,
            ROLE_ADMIN => &mut roles.admins,
            ROLE_NGO => &mut roles.ngos,
            ROLE_VENDOR => &mut roles.vendors,
            ROLE_BENEFICIARY => &mut roles.beneficiaries,
            _ => return Err(ContractError::InvalidRole),
        };

        *role_vec = to_soroban_vec(&env, role_vec.iter().filter(|m| *m != member_to_remove));
        set_roles(&env, &project_id, &roles);
        env.events().publish(
            (RoleRemovedEvent {
                project_id: project_id.clone(),
                role,
                member: member_to_remove,
            },),
            (),
        );
        Ok(())
    }

    // -- pause control --
    /// Pauses the contract for a short window.
    ///
    /// # Arguments
    /// - `caller`: owner or contract admin.
    ///
    /// # Errors
    /// - `ContractError::Unauthorized`
    pub fn pause_contract(env: Env, caller: Address) -> Result<(), ContractError> {
        require_owner_or_contract_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, PAUSED_KEY), &true);
        let until = env.ledger().timestamp().saturating_add(PAUSE_MAX_SECS);
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, PAUSE_UNTIL_KEY), &until);
        env.events()
            .publish((ContractPausedEvent { by: caller, until },), ());
        Ok(())
    }

    /// Unpauses the contract immediately.
    ///
    /// # Arguments
    /// - `caller`: owner or contract admin.
    ///
    /// # Errors
    /// - `ContractError::Unauthorized`
    pub fn unpause_contract(env: Env, caller: Address) -> Result<(), ContractError> {
        require_owner_or_contract_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, PAUSED_KEY), &false);
        env.storage()
            .persistent()
            .remove(&Symbol::new(&env, PAUSE_UNTIL_KEY));
        env.events()
            .publish((ContractUnpausedEvent { by: caller },), ());
        Ok(())
    }

    /// Upgrades the contract to a new WASM hash.
    ///
    /// # Arguments
    /// - `caller`: owner or contract admin.
    /// - `new_wasm_hash`: The hash of the new WASM code.
    ///
    /// # Errors
    /// - `ContractError::Unauthorized`
    pub fn upgrade(
        env: Env,
        caller: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        require_owner_or_contract_admin(&env, &caller)?;
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    /// Adds a global contract admin.
    ///
    /// # Arguments
    /// - `caller`: owner or contract admin.
    /// - `new_admin`: address to grant admin rights.
    ///
    /// # Errors
    /// - `ContractError::RoleAlreadyExists`
    /// - `ContractError::RoleLimitExceeded`
    /// - `ContractError::Unauthorized`
    /// - `ContractError::InvalidInput`
    pub fn add_contract_admin(
        env: Env,
        caller: Address,
        new_admin: Address,
    ) -> Result<(), ContractError> {
        require_owner_or_contract_admin(&env, &caller)?;
        let mut admins: Vec<Address> = get_contract_admins(&env);
        check_role_limits(&admins, &new_admin)?;
        admins.push_back(new_admin.clone());
        set_contract_admins(&env, &admins);
        env.events().publish(
            (ContractAdminAddedEvent {
                by: caller,
                added: new_admin,
            },),
            (),
        );
        Ok(())
    }

    /// Removes a global contract admin.
    ///
    /// # Arguments
    /// - `caller`: owner or contract admin.
    /// - `admin`: admin address to remove.
    ///
    /// # Errors
    /// - `ContractError::Unauthorized`
    pub fn remove_contract_admin(
        env: Env,
        caller: Address,
        admin: Address,
    ) -> Result<(), ContractError> {
        require_owner_or_contract_admin(&env, &caller)?;
        let mut admins: Vec<Address> = get_contract_admins(&env);
        admins = to_soroban_vec(&env, admins.iter().filter(|a| *a != admin));
        set_contract_admins(&env, &admins);
        env.events().publish(
            (ContractAdminRemovedEvent {
                by: caller,
                removed: admin,
            },),
            (),
        );
        Ok(())
    }

    // -- allowance management --
    /// Allocates a cash allowance.
    ///
    /// # Arguments
    /// - `caller`: ngo address.
    /// - `project_id`: project identifier.
    /// - `allowee`: beneficiary/vendor.
    /// - `amount`: positive amount.
    /// - `currency`: currency code.
    /// - `expiry`: future timestamp or none.
    ///
    /// # Errors
    /// - `ContractError::ContractPaused`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn allocate_cash_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        allowee: Address,
        amount: u64,
        currency: String,
        expiry: Option<u64>,
    ) -> Result<(), ContractError> {
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, ROLE_NGO)?;
        is_valid_amount(amount)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &currency)?;
        is_future_expiry(&env, expiry)?;

        let storage_key = (Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .unwrap_or_else(|| Map::new(&env));

        let key = (allowee, currency);

        let new_allowance = if let Some(existing) = allowances.get(key.clone()) {
            CashAllowance {
                amount: is_valid_sum(existing.amount, amount)?,
                expiry: merge_expiry(existing.expiry, expiry),
            }
        } else {
            CashAllowance { amount, expiry }
        };

        allowances.set(key.clone(), new_allowance);
        env.storage().persistent().set(&storage_key, &allowances);

        let (event_allowee, event_currency) = (key.0, key.1);
        env.events().publish(
            (AllowanceAllocated {
                project_id: project_id.clone(),
                allowee: event_allowee,
                currency_or_item: event_currency,
                amount,
            },),
            (),
        );
        Ok(())
    }

    /// Claims a cash allowance, optionally crediting a vendor.
    ///
    /// # Arguments
    /// - `caller`: beneficiary address.
    /// - `project_id`: project identifier.
    /// - `currency`: currency code.
    /// - `amount`: positive amount.
    /// - `vendor`: optional vendor to credit.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::ExpiredAllowance`
    /// - `ContractError::InsufficientFunds`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn claim_cash_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        currency: String,
        amount: u64,
        vendor: Option<Address>,
    ) -> Result<(), ContractError> {
        require_role(&env, &caller, &project_id, ROLE_BENEFICIARY)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &currency)?;
        is_valid_amount(amount)?;

        let storage_key = (Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::AllowanceNotFound)?;

        let ben_key = (caller.clone(), currency.clone());
        let mut ben_allowance: CashAllowance = allowances
            .get(ben_key.clone())
            .ok_or(ContractError::AllowanceNotFound)?;

        check_expiry(&env, ben_allowance.expiry)?;
        if ben_allowance.amount < amount {
            return Err(ContractError::InsufficientFunds);
        }
        ben_allowance.amount -= amount;

        allowances.set(ben_key.clone(), ben_allowance);

        if let Some(vendor) = vendor {
            is_valid_transfer_parties(&caller, &vendor)?;
            let vendor_key = (vendor, currency);
            let mut vendor_allowance: CashAllowance =
                allowances.get(vendor_key.clone()).unwrap_or(CashAllowance {
                    amount: 0,
                    expiry: None,
                });
            vendor_allowance.amount = is_valid_sum(vendor_allowance.amount, amount)?;
            allowances.set(vendor_key, vendor_allowance);
        }

        env.storage().persistent().set(&storage_key, &allowances);

        let event_allowee = ben_key.0;
        env.events().publish(
            (AllowanceClaimed {
                project_id: project_id.clone(),
                allowee: event_allowee,
                amount,
            },),
            (),
        );
        Ok(())
    }

    /// Allocates an item allowance.
    ///
    /// # Arguments
    /// - `caller`: ngo address.
    /// - `project_id`: project identifier.
    /// - `allowee`: beneficiary/vendor.
    /// - `item_id`: item identifier.
    /// - `quantity`: positive quantity.
    /// - `expiry`: future timestamp or none.
    ///
    /// # Errors
    /// - `ContractError::ContractPaused`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn allocate_item_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        allowee: Address,
        item_id: String,
        quantity: u64,
        expiry: Option<u64>,
    ) -> Result<(), ContractError> {
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, ROLE_NGO)?;
        is_valid_quantity(quantity)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &item_id)?;
        check_expiry(&env, expiry)?;

        let storage_key = (Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .unwrap_or_else(|| Map::new(&env));

        let key = (allowee, item_id);
        let new_allowance = if let Some(existing) = allowances.get(key.clone()) {
            let new_qty = is_valid_sum(existing.quantity, quantity)?;
            let new_expiry = merge_expiry(existing.expiry, expiry);
            ItemAllowance {
                quantity: new_qty,
                expiry: new_expiry,
            }
        } else {
            ItemAllowance { quantity, expiry }
        };

        allowances.set(key.clone(), new_allowance);
        env.storage().persistent().set(&storage_key, &allowances);

        let (event_allowee, event_item) = (key.0, key.1);
        env.events().publish(
            (AllowanceAllocated {
                project_id: project_id.clone(),
                allowee: event_allowee,
                currency_or_item: event_item,
                amount: quantity,
            },),
            (),
        );
        Ok(())
    }

    /// Claims an item allowance and credits a vendor.
    ///
    /// # Arguments
    /// - `caller`: beneficiary address.
    /// - `vendor`: vendor address.
    /// - `project_id`: project identifier.
    /// - `item_id`: item identifier.
    /// - `quantity`: positive quantity.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::ExpiredAllowance`
    /// - `ContractError::InsufficientFunds`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn claim_item_allowance(
        env: Env,
        caller: Address,
        vendor: Address,
        project_id: String,
        item_id: String,
        quantity: u64,
    ) -> Result<(), ContractError> {
        require_role(&env, &caller, &project_id, ROLE_BENEFICIARY)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &item_id)?;
        is_valid_quantity(quantity)?;
        is_valid_transfer_parties(&caller, &vendor)?;
        is_valid_vendor(&env, &project_id, &vendor)?;

        let storage_key = (Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::AllowanceNotFound)?;

        let ben_key = (caller.clone(), item_id.clone());
        let mut ben_allowance: ItemAllowance = allowances
            .get(ben_key.clone())
            .ok_or(ContractError::AllowanceNotFound)?;

        check_expiry(&env, ben_allowance.expiry)?;
        if ben_allowance.quantity < quantity {
            return Err(ContractError::InsufficientFunds);
        }
        ben_allowance.quantity -= quantity;

        allowances.set(ben_key.clone(), ben_allowance);

        let vendor_key = (vendor, item_id);
        let mut vendor_allowance: ItemAllowance =
            allowances.get(vendor_key.clone()).unwrap_or(ItemAllowance {
                quantity: 0,
                expiry: None,
            });
        vendor_allowance.quantity = is_valid_sum(vendor_allowance.quantity, quantity)?;
        allowances.set(vendor_key, vendor_allowance);

        env.storage().persistent().set(&storage_key, &allowances);

        let event_allowee = ben_key.0;
        env.events().publish(
            (AllowanceClaimed {
                project_id: project_id.clone(),
                allowee: event_allowee,
                amount: quantity,
            },),
            (),
        );
        Ok(())
    }

    /// Claims multiple item allowances in a single transaction.
    ///
    /// # Arguments
    /// - `caller`: beneficiary address.
    /// - `vendor`: vendor address that will receive the items.
    /// - `project_id`: project identifier.
    /// - `claims`: vector of (item_id, quantity) tuples to claim.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::InsufficientFunds`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    /// - `ContractError::ExpiredAllowance`
    ///
    /// # Behavior
    /// This function validates ALL claims before making ANY changes (atomic).
    /// If any validation fails, the entire batch is rejected and no state changes occur.
    pub fn claim_item_allowances_batch(
        env: Env,
        caller: Address,
        vendor: Address,
        project_id: String,
        claims: Vec<(String, u64)>,
    ) -> Result<(), ContractError> {
        require_role(&env, &caller, &project_id, ROLE_BENEFICIARY)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_transfer_parties(&caller, &vendor)?;
        let roles = get_roles(&env, &project_id);
        if !roles.vendors.contains(&vendor) {
            return Err(ContractError::InvalidInput);
        }
        is_valid_batch_len(claims.len())?;

        let storage_key = (Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::AllowanceNotFound)?;

        // Single pass: validate and execute atomically
        // We consume the claims vector to avoid SDK iterator issues
        for (item_id, quantity) in claims {
            // Validate inputs
            is_valid_identifier(&env, &item_id)?;
            is_valid_quantity(quantity)?;

            // Check beneficiary has sufficient allowance
            let ben_key = (caller.clone(), item_id.clone());
            let ben_allowance = allowances
                .get(ben_key.clone())
                .ok_or(ContractError::AllowanceNotFound)?;

            // Check expiry
            if let Some(expiry) = ben_allowance.expiry {
                if env.ledger().timestamp() > expiry {
                    return Err(ContractError::ExpiredAllowance);
                }
            }

            // Check sufficient balance
            if ben_allowance.quantity < quantity {
                return Err(ContractError::InsufficientFunds);
            }

            // Execute claim: deduct from beneficiary
            let mut ben_allowance_updated = ben_allowance;
            ben_allowance_updated.quantity -= quantity;
            allowances.set(ben_key, ben_allowance_updated);

            // Credit to vendor
            let vendor_key = (vendor.clone(), item_id.clone());
            let mut vendor_allowance =
                allowances.get(vendor_key.clone()).unwrap_or(ItemAllowance {
                    quantity: 0,
                    expiry: None,
                });
            vendor_allowance.quantity = is_valid_sum(vendor_allowance.quantity, quantity)?;
            allowances.set(vendor_key, vendor_allowance);

            // Emit event for this claim
            env.events().publish(
                (AllowanceClaimed {
                    project_id: project_id.clone(),
                    allowee: caller.clone(),
                    amount: quantity,
                },),
                (),
            );
        }

        // Commit all changes
        env.storage().persistent().set(&storage_key, &allowances);
        Ok(())
    }

    // -- batch operations --
    /// Allocates multiple cash allowances.
    ///
    /// # Arguments
    /// - `caller`: ngo address.
    /// - `project_id`: project identifier.
    /// - `allowances`: tuples of (allowee, currency, amount, expiry).
    ///
    /// # Errors
    /// - `ContractError::ContractPaused`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn allocate_cash_allowances_batch(
        env: Env,
        caller: Address,
        project_id: String,
        allowances: Vec<(Address, String, u64, Option<u64>)>,
    ) -> Result<(), ContractError> {
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, ROLE_NGO)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_batch_len(allowances.len())?;

        let storage_key = (Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone());
        let mut allowances_map: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .unwrap_or_else(|| Map::new(&env));

        for (allowee, currency, amount, expiry) in allowances {
            is_valid_amount(amount)?;
            is_valid_identifier(&env, &currency)?;
            is_future_expiry(&env, expiry)?;
            let key = (allowee.clone(), currency.clone());
            let new_allowance = if let Some(existing) = allowances_map.get(key.clone()) {
                let new_amount = is_valid_sum(existing.amount, amount)?;
                let new_expiry = merge_expiry(existing.expiry, expiry);
                CashAllowance {
                    amount: new_amount,
                    expiry: new_expiry,
                }
            } else {
                CashAllowance { amount, expiry }
            };
            allowances_map.set(key, new_allowance);
            env.events().publish(
                (AllowanceAllocated {
                    project_id: project_id.clone(),
                    allowee,
                    currency_or_item: currency,
                    amount,
                },),
                (),
            );
        }

        env.storage()
            .persistent()
            .set(&storage_key, &allowances_map);
        Ok(())
    }

    /// Allocates multiple item allowances.
    ///
    /// # Arguments
    /// - `caller`: ngo address.
    /// - `project_id`: project identifier.
    /// - `allowances`: tuples of (allowee, item_id, quantity, expiry).
    ///
    /// # Errors
    /// - `ContractError::ContractPaused`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn allocate_item_allowances_batch(
        env: Env,
        caller: Address,
        project_id: String,
        allowances: Vec<(Address, String, u64, Option<u64>)>,
    ) -> Result<(), ContractError> {
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, ROLE_NGO)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_batch_len(allowances.len())?;

        let storage_key = (Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone());
        let mut allowances_map: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .unwrap_or_else(|| Map::new(&env));

        for (allowee, item_id, quantity, expiry) in allowances {
            is_valid_quantity(quantity)?;
            is_valid_identifier(&env, &item_id)?;
            is_future_expiry(&env, expiry)?;
            let key = (allowee.clone(), item_id.clone());
            let new_allowance = if let Some(existing) = allowances_map.get(key.clone()) {
                let new_qty = is_valid_sum(existing.quantity, quantity)?;
                let new_expiry = merge_expiry(existing.expiry, expiry);
                ItemAllowance {
                    quantity: new_qty,
                    expiry: new_expiry,
                }
            } else {
                ItemAllowance { quantity, expiry }
            };
            allowances_map.set(key, new_allowance);
            env.events().publish(
                (AllowanceAllocated {
                    project_id: project_id.clone(),
                    allowee,
                    currency_or_item: item_id,
                    amount: quantity,
                },),
                (),
            );
        }

        env.storage()
            .persistent()
            .set(&storage_key, &allowances_map);
        Ok(())
    }

    // -- transfer functions --
    /// Transfers a cash allowance to another address.
    ///
    /// # Arguments
    /// - `caller`: beneficiary address.
    /// - `project_id`: project identifier.
    /// - `new_allowee`: recipient address.
    /// - `currency`: currency code.
    /// - `amount`: positive amount.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::InsufficientFunds`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    /// - `ContractError::ExpiredAllowance`
    pub fn transfer_cash_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        new_allowee: Address,
        currency: String,
        amount: u64,
    ) -> Result<(), ContractError> {
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, ROLE_BENEFICIARY)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &currency)?;
        is_valid_amount(amount)?;
        is_valid_transfer_parties(&caller, &new_allowee)?;

        let storage_key = (Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::AllowanceNotFound)?;

        let src_key = (caller.clone(), currency.clone());
        let mut src_allowance: CashAllowance = allowances
            .get(src_key.clone())
            .ok_or(ContractError::AllowanceNotFound)?;

        check_expiry(&env, src_allowance.expiry)?;
        if src_allowance.amount < amount {
            return Err(ContractError::InsufficientFunds);
        }
        src_allowance.amount -= amount;

        let dest_key = (new_allowee, currency);
        let mut dest_allowance: CashAllowance =
            allowances.get(dest_key.clone()).unwrap_or(CashAllowance {
                amount: 0,
                expiry: None,
            });
        dest_allowance.amount = is_valid_sum(dest_allowance.amount, amount)?;

        allowances.set(src_key.clone(), src_allowance);
        allowances.set(dest_key, dest_allowance);

        env.storage().persistent().set(&storage_key, &allowances);

        let event_allowee = src_key.0;
        env.events().publish(
            (AllowanceClaimed {
                project_id: project_id.clone(),
                allowee: event_allowee,
                amount,
            },),
            (),
        );
        Ok(())
    }

    /// Transfers an item allowance to another address.
    ///
    /// # Arguments
    /// - `caller`: beneficiary address.
    /// - `project_id`: project identifier.
    /// - `new_allowee`: recipient address.
    /// - `item_id`: item identifier.
    /// - `quantity`: positive quantity.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::InsufficientFunds`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    /// - `ContractError::ExpiredAllowance`
    pub fn transfer_item_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        new_allowee: Address,
        item_id: String,
        quantity: u64,
    ) -> Result<(), ContractError> {
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, ROLE_BENEFICIARY)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &item_id)?;
        is_valid_quantity(quantity)?;
        is_valid_transfer_parties(&caller, &new_allowee)?;

        let storage_key = (Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::AllowanceNotFound)?;

        let src_key = (caller.clone(), item_id.clone());
        let mut src_allowance: ItemAllowance = allowances
            .get(src_key.clone())
            .ok_or(ContractError::AllowanceNotFound)?;

        check_expiry(&env, src_allowance.expiry)?;
        if src_allowance.quantity < quantity {
            return Err(ContractError::InsufficientFunds);
        }
        src_allowance.quantity -= quantity;

        let dest_key = (new_allowee, item_id);
        let mut dest_allowance: ItemAllowance =
            allowances.get(dest_key.clone()).unwrap_or(ItemAllowance {
                quantity: 0,
                expiry: None,
            });
        dest_allowance.quantity = is_valid_sum(dest_allowance.quantity, quantity)?;

        allowances.set(src_key.clone(), src_allowance);
        allowances.set(dest_key, dest_allowance);

        env.storage().persistent().set(&storage_key, &allowances);

        let event_allowee = src_key.0;
        env.events().publish(
            (AllowanceClaimed {
                project_id: project_id.clone(),
                allowee: event_allowee,
                amount: quantity,
            },),
            (),
        );
        Ok(())
    }

    // -- redemption functions --
    /// Redeems item claims credited to a vendor.
    ///
    /// # Arguments
    /// - `vendor`: vendor address.
    /// - `project_id`: optional project id (defaults to "default_project").
    /// - `item_id`: item identifier.
    /// - `quantity`: quantity to redeem.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::InsufficientFunds`
    /// - `ContractError::ContractPaused`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn redeem_item_claims(
        env: Env,
        vendor: Address,
        project_id: Option<String>,
        item_id: String,
        quantity: u64,
    ) -> Result<(), ContractError> {
        let project_id = project_id.unwrap_or(String::from_str(&env, "default_project"));
        check_paused(&env)?;

        require_role(&env, &vendor, &project_id, ROLE_VENDOR)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &item_id)?;
        is_valid_quantity(quantity)?;

        let storage_key = (Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::AllowanceNotFound)?;

        let item_key = (vendor.clone(), item_id);
        let mut vendor_allowance: ItemAllowance = allowances
            .get(item_key.clone())
            .ok_or(ContractError::AllowanceNotFound)?;

        if vendor_allowance.quantity < quantity {
            return Err(ContractError::InsufficientFunds);
        }

        vendor_allowance.quantity -= quantity;
        if vendor_allowance.quantity == 0 {
            allowances.remove(item_key.clone());
        } else {
            allowances.set(item_key.clone(), vendor_allowance);
        }

        env.storage().persistent().set(&storage_key, &allowances);

        let event_allowee = item_key.0;
        env.events().publish(
            (AllowanceClaimed {
                project_id: project_id.clone(),
                allowee: event_allowee,
                amount: quantity,
            },),
            (),
        );
        Ok(())
    }

    /// Redeems cash claims credited to a vendor.
    ///
    /// # Arguments
    /// - `vendor`: vendor address.
    /// - `project_id`: optional project id (defaults to "default_project").
    /// - `currency`: currency code.
    /// - `amount`: amount to redeem.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::InsufficientFunds`
    /// - `ContractError::ContractPaused`
    /// - `ContractError::InvalidInput`
    /// - `ContractError::Unauthorized`
    pub fn redeem_cash_claims(
        env: Env,
        vendor: Address,
        project_id: Option<String>,
        currency: String,
        amount: u64,
    ) -> Result<(), ContractError> {
        let project_id = project_id.unwrap_or(String::from_str(&env, "default_project"));
        check_paused(&env)?;

        require_role(&env, &vendor, &project_id, ROLE_VENDOR)?;
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &currency)?;
        is_valid_amount(amount)?;

        let storage_key = (Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone());
        let mut allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::AllowanceNotFound)?;

        let key = (vendor.clone(), currency);
        let mut vendor_allowance: CashAllowance = allowances
            .get(key.clone())
            .ok_or(ContractError::AllowanceNotFound)?;

        if vendor_allowance.amount < amount {
            return Err(ContractError::InsufficientFunds);
        }

        vendor_allowance.amount -= amount;

        if vendor_allowance.amount == 0 {
            allowances.remove(key);
        } else {
            allowances.set(key, vendor_allowance);
        }

        env.storage().persistent().set(&storage_key, &allowances);

        env.events().publish(
            (AllowanceClaimed {
                project_id: project_id.clone(),
                allowee: vendor,
                amount,
            },),
            (),
        );
        Ok(())
    }

    // -- query functions --
    /// Gets a cash allowance.
    ///
    /// # Arguments
    /// - `project_id`: project identifier.
    /// - `allowee`: address.
    /// - `currency`: currency code.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::InvalidInput`
    pub fn get_cash_allowance(
        env: Env,
        project_id: String,
        allowee: Address,
        currency: String,
    ) -> Result<CashAllowance, ContractError> {
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &currency)?;
        let allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id))
            .ok_or(ContractError::AllowanceNotFound)?;

        allowances
            .get((allowee, currency))
            .ok_or(ContractError::AllowanceNotFound)
    }

    /// Gets an item allowance.
    ///
    /// # Arguments
    /// - `project_id`: project identifier.
    /// - `allowee`: address.
    /// - `item_id`: item identifier.
    ///
    /// # Errors
    /// - `ContractError::AllowanceNotFound`
    /// - `ContractError::InvalidInput`
    pub fn get_item_allowance(
        env: Env,
        project_id: String,
        allowee: Address,
        item_id: String,
    ) -> Result<ItemAllowance, ContractError> {
        is_valid_identifier(&env, &project_id)?;
        is_valid_identifier(&env, &item_id)?;
        let allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
            .ok_or(ContractError::AllowanceNotFound)?;

        allowances
            .get((allowee, item_id))
            .ok_or(ContractError::AllowanceNotFound)
    }

    /// Lists all cash allowances for a project.
    ///
    /// # Arguments
    /// - `project_id`: project identifier.
    pub fn get_all_cash_allowances(
        env: Env,
        project_id: String,
    ) -> Map<(Address, String), CashAllowance> {
        let _ = is_valid_identifier(&env, &project_id);
        env.storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id))
            .unwrap_or_else(|| Map::new(&env))
    }

    /// Lists all item allowances for a project.
    ///
    /// # Arguments
    /// - `project_id`: project identifier.
    pub fn get_all_item_allowances(
        env: Env,
        project_id: String,
    ) -> Map<(Address, String), ItemAllowance> {
        let _ = is_valid_identifier(&env, &project_id);
        env.storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
            .unwrap_or_else(|| Map::new(&env))
    }

    /// Sums total cash allowance across projects for a beneficiary.
    ///
    /// # Arguments
    /// - `beneficiary`: address.
    /// - `project_ids`: list of project identifiers.
    ///
    /// # Errors
    /// - `ContractError::InvalidInput` if overflow occurs or validation fails
    pub fn get_total_cash_allowance(
        env: Env,
        beneficiary: Address,
        project_ids: Vec<String>,
    ) -> Result<u64, ContractError> {
        let mut total_amount: u64 = 0;
        // Bound the number of project IDs
        is_valid_batch_len(project_ids.len())?;

        for project_id in project_ids.iter() {
            // Validate each project identifier
            is_valid_identifier(&env, &project_id)?;
            let allowances: Map<(Address, String), CashAllowance> =
                Self::get_all_cash_allowances(env.clone(), project_id.clone());

            for ((allowee, _currency), allowance) in allowances.iter() {
                if allowee == beneficiary {
                    total_amount = is_valid_sum(total_amount, allowance.amount)?;
                }
            }
        }

        Ok(total_amount)
    }

    /// Sums total item allowance across projects for a beneficiary.
    ///
    /// # Arguments
    /// - `beneficiary`: address.
    /// - `project_ids`: list of project identifiers.
    ///
    /// # Errors
    /// - `ContractError::InvalidInput` if overflow occurs or validation fails
    pub fn get_total_item_allowance(
        env: Env,
        beneficiary: Address,
        project_ids: Vec<String>,
    ) -> Result<u64, ContractError> {
        let mut total_quantity: u64 = 0;
        is_valid_batch_len(project_ids.len())?;

        for project_id in project_ids.iter() {
            is_valid_identifier(&env, &project_id)?;
            let allowances: Map<(Address, String), ItemAllowance> =
                Self::get_all_item_allowances(env.clone(), project_id.clone());

            for ((allowee, _item_id), allowance) in allowances.iter() {
                if allowee == beneficiary {
                    total_quantity = is_valid_sum(total_quantity, allowance.quantity)?;
                }
            }
        }

        Ok(total_quantity)
    }

    /// Gets roles for a project.
    ///
    /// # Arguments
    /// - `project_id`: project identifier.
    pub fn get_roles(env: Env, project_id: String) -> Roles {
        // Validate project identifier
        let _ = is_valid_identifier(&env, &project_id);
        get_roles(&env, &project_id)
    }

    pub fn get_version(env: Env) -> String {
        String::from_str(&env, "1.0.0")
    }

    pub fn get_owner(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&Symbol::new(&env, OWNER_KEY))
            .expect("Owner not set")
    }

    pub fn get_contract_admins(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&Symbol::new(&env, CONTRACT_ADMINS_KEY))
            .unwrap_or(Vec::new(&env))
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&Symbol::new(&env, PAUSED_KEY))
            .unwrap_or(false)
    }

    /// returns pause expiry timestamp if paused, None otherwise.
    pub fn get_pause_until(env: Env) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&Symbol::new(&env, PAUSE_UNTIL_KEY))
    }

    /// Checks if an address has a specific role in a project.
    ///
    /// # Arguments
    /// - `project_id`: The project identifier.
    /// - `address`: The address to check.
    /// - `role`: The role integer (0=super_admin, 1=admin, 2=ngo, 3=vendor, 4=beneficiary).
    pub fn has_role(env: Env, project_id: String, address: Address, role: u32) -> bool {
        let _ = is_valid_identifier(&env, &project_id);
        let roles = get_roles(&env, &project_id);

        match role {
            ROLE_SUPER_ADMIN => roles.super_admins.iter().any(|a| a == address),
            ROLE_ADMIN => roles.admins.iter().any(|a| a == address),
            ROLE_NGO => roles.ngos.iter().any(|a| a == address),
            ROLE_VENDOR => roles.vendors.iter().any(|a| a == address),
            ROLE_BENEFICIARY => roles.beneficiaries.iter().any(|a| a == address),
            _ => false,
        }
    }

    /// Transfers contract ownership to a new address.
    ///
    /// # Arguments
    /// - `caller`: Current owner.
    /// - `new_owner`: New owner address.
    ///
    /// # Errors
    /// - `ContractError::Unauthorized`
    /// - `ContractError::InvalidInput`
    pub fn transfer_ownership(
        env: Env,
        caller: Address,
        new_owner: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let owner: Address = env
            .storage()
            .persistent()
            .get(&Symbol::new(&env, OWNER_KEY))
            .ok_or(ContractError::Unauthorized)?;

        if caller != owner {
            return Err(ContractError::Unauthorized);
        }

        // validate new owner is not the same as current owner
        if new_owner == owner {
            return Err(ContractError::InvalidInput);
        }

        let old_owner = owner.clone();
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, OWNER_KEY), &new_owner);

        env.events().publish(
            (Symbol::new(&env, "ownership_transferred"),),
            (old_owner, new_owner),
        );

        Ok(())
    }

    /// Extends the TTL (Time To Live) for contract storage.
    ///
    /// # Arguments
    /// - `caller`: owner or contract admin.
    /// - `extend_to`: Number of ledgers to extend.
    ///
    /// # Errors
    /// - `ContractError::Unauthorized`
    /// - `ContractError::InvalidInput`
    pub fn extend_ttl(env: Env, caller: Address, extend_to: u32) -> Result<(), ContractError> {
        require_owner_or_contract_admin(&env, &caller)?;

        // validate extend_to is reasonable (!too large to avoid abuse)
        if extend_to == 0 || extend_to > 5_184_000 {
            // max ~1 year (assuming 17280 ledgers/day)
            return Err(ContractError::InvalidInput);
        }

        // Extend critical storage keys only if they exist
        let owner_key = Symbol::new(&env, OWNER_KEY);
        if env.storage().persistent().has(&owner_key) {
            env.storage()
                .persistent()
                .extend_ttl(&owner_key, extend_to, extend_to);
        }

        let admins_key = Symbol::new(&env, CONTRACT_ADMINS_KEY);
        if env.storage().persistent().has(&admins_key) {
            env.storage()
                .persistent()
                .extend_ttl(&admins_key, extend_to, extend_to);
        }

        let paused_key = Symbol::new(&env, PAUSED_KEY);
        if env.storage().persistent().has(&paused_key) {
            env.storage()
                .persistent()
                .extend_ttl(&paused_key, extend_to, extend_to);
        }

        Ok(())
    }
}

// -- helper functions --
fn check_paused(env: &Env) -> Result<(), ContractError> {
    let paused: bool = env
        .storage()
        .persistent()
        .get(&Symbol::new(env, PAUSED_KEY))
        .unwrap_or(false);
    if !paused {
        return Ok(());
    }

    let now = env.ledger().timestamp();
    let until: Option<u64> = env
        .storage()
        .persistent()
        .get(&Symbol::new(env, PAUSE_UNTIL_KEY));
    if let Some(u) = until {
        if now >= u {
            env.storage()
                .persistent()
                .set(&Symbol::new(env, PAUSED_KEY), &false);
            env.storage()
                .persistent()
                .remove(&Symbol::new(env, PAUSE_UNTIL_KEY));
            return Ok(());
        }
    }
    Err(ContractError::ContractPaused)
}

fn check_expiry(env: &Env, expiry: Option<u64>) -> Result<(), ContractError> {
    if let Some(expiry) = expiry {
        if env.ledger().timestamp() > expiry {
            return Err(ContractError::ExpiredAllowance);
        }
    }
    Ok(())
}

fn is_valid_amount(amount: u64) -> Result<(), ContractError> {
    if amount == 0 {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

fn is_valid_quantity(quantity: u64) -> Result<(), ContractError> {
    if quantity == 0 {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

fn is_valid_identifier(_env: &Env, s: &String) -> Result<(), ContractError> {
    // disallow empty and overly long strings
    if s.len() == 0 || s.len() > MAX_ID_LEN {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

fn is_future_expiry(env: &Env, expiry: Option<u64>) -> Result<(), ContractError> {
    if let Some(e) = expiry {
        let now = env.ledger().timestamp();
        if e <= now {
            return Err(ContractError::InvalidInput);
        }
        if e - now > MAX_EXPIRY_AHEAD_SECS {
            return Err(ContractError::InvalidInput);
        }
    }
    Ok(())
}

fn is_valid_batch_len(len: u32) -> Result<(), ContractError> {
    if len > MAX_BATCH_LEN {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

fn is_valid_transfer_parties(caller: &Address, other: &Address) -> Result<(), ContractError> {
    if caller == other {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

fn is_valid_vendor(env: &Env, project_id: &String, addr: &Address) -> Result<(), ContractError> {
    let roles = get_roles(env, project_id);
    if roles.vendors.contains(addr) {
        Ok(())
    } else {
        Err(ContractError::InvalidInput)
    }
}

fn is_valid_sum(a: u64, b: u64) -> Result<u64, ContractError> {
    a.checked_add(b).ok_or(ContractError::InvalidInput)
}

// Merge expiry when adding to an existing allowance.
// Policy:
// - If existing is None, keep None (unbounded remains unbounded).
// - If new is None, keep existing as-is (don't relax existing expiry).
// - If both Some, take the later (max).
fn merge_expiry(existing: Option<u64>, new: Option<u64>) -> Option<u64> {
    match (existing, new) {
        (None, _) => None,
        (Some(e), None) => Some(e),
        (Some(a), Some(b)) => Some(core::cmp::max(a, b)),
    }
}

fn require_owner(env: &Env, caller: &Address) -> Result<(), ContractError> {
    caller.require_auth();
    let owner: Address = env
        .storage()
        .persistent()
        .get(&Symbol::new(env, OWNER_KEY))
        .ok_or(ContractError::Unauthorized)?;

    if caller == &owner {
        Ok(())
    } else {
        Err(ContractError::Unauthorized)
    }
}

fn require_role(
    env: &Env,
    caller: &Address,
    project_id: &String,
    required_role: u32,
) -> Result<(), ContractError> {
    caller.require_auth();
    let roles = get_roles(env, project_id);

    let has_access = match required_role {
        ROLE_SUPER_ADMIN => roles.super_admins.contains(caller),
        ROLE_ADMIN => roles.admins.contains(caller) || roles.super_admins.contains(caller),
        ROLE_NGO => roles.ngos.contains(caller) || roles.admins.contains(caller),
        ROLE_BENEFICIARY => roles.beneficiaries.contains(caller),
        ROLE_VENDOR => roles.vendors.contains(caller),
        _ => {
            return Err(ContractError::InvalidRole);
        }
    };

    if has_access {
        Ok(())
    } else {
        Err(ContractError::Unauthorized)
    }
}

fn get_contract_admins(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&Symbol::new(env, CONTRACT_ADMINS_KEY))
        .unwrap_or_else(|| Vec::new(env))
}

fn set_contract_admins(env: &Env, admins: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&Symbol::new(env, CONTRACT_ADMINS_KEY), admins);
}

fn require_owner_or_contract_admin(env: &Env, caller: &Address) -> Result<(), ContractError> {
    caller.require_auth();
    if let Some(owner) = env
        .storage()
        .persistent()
        .get::<_, Address>(&Symbol::new(env, OWNER_KEY))
    {
        if &owner == caller {
            return Ok(());
        }
    }
    let admins = get_contract_admins(env);
    if admins.contains(caller) {
        return Ok(());
    }
    Err(ContractError::Unauthorized)
}

fn get_roles(env: &Env, project_id: &String) -> Roles {
    let storage_key = (Symbol::new(env, ROLES_KEY), project_id.clone());
    env.storage()
        .persistent()
        .get(&storage_key)
        .unwrap_or_else(|| Roles {
            super_admins: Vec::new(env),
            admins: Vec::new(env),
            ngos: Vec::new(env),
            vendors: Vec::new(env),
            beneficiaries: Vec::new(env),
        })
}

fn set_roles(env: &Env, project_id: &String, roles: &Roles) {
    let storage_key = (Symbol::new(env, ROLES_KEY), project_id.clone());
    env.storage().persistent().set(&storage_key, roles);
}

fn to_soroban_vec<T>(env: &Env, iter: impl IntoIterator<Item = T>) -> Vec<T>
where
    T: Clone + TryFromVal<Env, Val> + IntoVal<Env, Val>,
{
    let mut result = Vec::new(env);
    for item in iter {
        result.push_back(item);
    }
    result
}

fn check_role_limits(vec: &Vec<Address>, member: &Address) -> Result<(), ContractError> {
    if vec.contains(member) {
        return Err(ContractError::RoleAlreadyExists);
    }
    if vec.len() >= MAX_ROLE_MEMBERS {
        return Err(ContractError::RoleLimitExceeded);
    }
    Ok(())
}

#[cfg(test)]
mod test;
