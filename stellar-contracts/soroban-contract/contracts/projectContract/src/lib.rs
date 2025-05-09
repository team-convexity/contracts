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
    Val,
    Vec,
    Env,
    Map,
    Symbol,
    String,
    Address,
    IntoVal,
    contract,
    TryFromVal,
    contractimpl,
    contracttype,
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
    ReentrancyDetected,
}

#[derive(Clone)]
pub enum ReentrancyLock {
    AddRole,
    GetRoles,
    AllocCash,
    AllocItem,
    ClaimItem,
    ClaimCash,
    GetAllCash,
    RedeemItem,
    RedeemCash,
    Initialize,
    RemoveRole,
    GetAllItems,
    GetTotalCash,
    TransferItem,
    TransferCash,
    GetTotalItems,
    PauseContract,
    AllocCashBatch,
    AllocItemBatch,
    UnpauseContract,
    GetCashAllowance,
    GetItemAllowance,
}

impl ReentrancyLock {
    pub fn to_symbol(&self, env: &Env) -> Symbol {
        match self {
            Self::Initialize => Symbol::new(env, "lock_init"),
            Self::AddRole => Symbol::new(env, "lock_add_role"),
            Self::RemoveRole => Symbol::new(env, "lock_remove_role"),
            Self::PauseContract => Symbol::new(env, "lock_pause"),
            Self::UnpauseContract => Symbol::new(env, "lock_unpause"),
            Self::AllocCash => Symbol::new(env, "lock_alloc_cash"),
            Self::ClaimCash => Symbol::new(env, "lock_claim_cash"),
            Self::AllocCashBatch => Symbol::new(env, "lock_alloc_cash_batch"),
            Self::TransferCash => Symbol::new(env, "lock_transfer_cash"),
            Self::AllocItem => Symbol::new(env, "lock_alloc_item"),
            Self::ClaimItem => Symbol::new(env, "lock_claim_item"),
            Self::AllocItemBatch => Symbol::new(env, "lock_alloc_item_batch"),
            Self::TransferItem => Symbol::new(env, "lock_transfer_item"),
            Self::RedeemItem => Symbol::new(env, "lock_redeem_item"),
            Self::RedeemCash => Symbol::new(env, "lock_redeem_cash"),

            // just aesthetics - we dont' need check on these
            Self::GetCashAllowance => Symbol::new(env, "lock_get_cash"),
            Self::GetItemAllowance => Symbol::new(env, "lock_get_item"),
            Self::GetAllCash => Symbol::new(env, "lock_get_all_cash"),
            Self::GetAllItems => Symbol::new(env, "lock_get_all_items"),
            Self::GetTotalCash => Symbol::new(env, "lock_get_total_cash"),
            Self::GetTotalItems => Symbol::new(env, "lock_get_total_items"),
            Self::GetRoles => Symbol::new(env, "lock_get_roles"),
        }
    }
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
            ContractError::ReentrancyDetected => Ok(soroban_sdk::Error::from_contract_error(1011)),
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
            ContractError::ReentrancyDetected => soroban_sdk::Error::from_contract_error(1011),
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

// -- events --
#[contracttype]
pub struct RoleAssigned {
    pub project_id: String,
    pub role: String,
    pub assignee: Address,
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

#[contractimpl]
impl ProjectContract {
    pub fn initialize(env: Env, owner: Address) {
        owner.require_auth();
        env.storage().persistent().set(&Symbol::new(&env, OWNER_KEY), &owner);
        env.storage().persistent().set(&Symbol::new(&env, VERSION_KEY), &1u32);
    }

    // -- role management --
    pub fn add_role(
        env: Env,
        caller: Address,
        project_id: String,
        role: String,
        new_member: Address
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::AddRole)?;
        require_owner(&env, &caller)?;

        let member_clone = new_member.clone();
        let mut roles = get_roles(&env, &project_id);

        if role == String::from_str(&env, "super_admin") {
            check_role_limits(&roles.super_admins, &new_member)?;
            roles.super_admins.push_back(member_clone.clone());
        } else if role == String::from_str(&env, "admin") {
            check_role_limits(&roles.admins, &new_member)?;
            roles.admins.push_back(member_clone.clone());
        } else if role == String::from_str(&env, "ngo") {
            check_role_limits(&roles.ngos, &new_member)?;
            roles.ngos.push_back(member_clone.clone());
        } else if role == String::from_str(&env, "vendor") {
            check_role_limits(&roles.vendors, &new_member)?;
            roles.vendors.push_back(member_clone.clone());
        } else if role == String::from_str(&env, "beneficiary") {
            check_role_limits(&roles.beneficiaries, &new_member)?;
            roles.beneficiaries.push_back(member_clone.clone());
        } else {
            return Err(ContractError::InvalidRole);
        }

        set_roles(&env, &project_id, &roles);
        env.events().publish(
            (
                RoleAssigned {
                    project_id,
                    role,
                    assignee: new_member,
                },
            ),
            ()
        );
        Ok(())
    }

    pub fn remove_role(
        env: Env,
        caller: Address,
        project_id: String,
        role: String,
        member_to_remove: Address
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::RemoveRole)?;
        require_owner(&env, &caller)?;
        let mut roles = get_roles(&env, &project_id);

        let vec = if role == String::from_str(&env, "super_admin") {
            &mut roles.super_admins
        } else if role == String::from_str(&env, "admin") {
            &mut roles.admins
        } else if role == String::from_str(&env, "ngo") {
            &mut roles.ngos
        } else if role == String::from_str(&env, "vendor") {
            &mut roles.vendors
        } else if role == String::from_str(&env, "beneficiary") {
            &mut roles.beneficiaries
        } else {
            return Err(ContractError::InvalidRole);
        };

        *vec = to_soroban_vec(
            &env,
            vec.iter().filter(|m| *m != member_to_remove)
        );
        set_roles(&env, &project_id, &roles);
        Ok(())
    }

    // -- pause control --
    pub fn pause_contract(env: Env, caller: Address) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::PauseContract)?;
        require_owner(&env, &caller)?;
        env.storage().persistent().set(&Symbol::new(&env, PAUSED_KEY), &true);
        Ok(())
    }

    pub fn unpause_contract(env: Env, caller: Address) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::UnpauseContract)?;
        require_owner(&env, &caller)?;
        env.storage().persistent().set(&Symbol::new(&env, PAUSED_KEY), &false);
        Ok(())
    }

    // -- allowance management --
    pub fn allocate_cash_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        allowee: Address,
        amount: u64,
        currency: String,
        expiry: Option<u64>
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::AllocCash)?;
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, "ngo")?;

        let mut allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
            .unwrap_or_else(|| Map::new(&env));

        allowances.set((allowee.clone(), currency.clone()), CashAllowance { amount, expiry });
        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()), &allowances);

        env.events().publish(
            (
                AllowanceAllocated {
                    project_id,
                    allowee,
                    currency_or_item: currency,
                    amount,
                },
            ),
            ()
        );
        Ok(())
    }

    pub fn claim_cash_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        currency: String,
        amount: u64,
        vendor: Option<Address>
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::ClaimCash)?;
        require_role(&env, &caller, &project_id, "beneficiary")?;

        let mut allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        let mut ben_allowance: CashAllowance = allowances
            .get((caller.clone(), currency.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        check_expiry(&env, ben_allowance.expiry)?;
        if ben_allowance.amount < amount {
            return Err(ContractError::InsufficientFunds);
        }
        ben_allowance.amount -= amount;

        allowances.set((caller.clone(), currency.clone()), ben_allowance);

        if let Some(vendor) = vendor {
            let mut vendor_allowance: CashAllowance = allowances
                .get((vendor.clone(), currency.clone()))
                .unwrap_or(CashAllowance {
                    amount: 0,
                    expiry: None,
                });
            vendor_allowance.amount += amount;
            allowances.set((vendor.clone(), currency.clone()), vendor_allowance);
        }

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()), &allowances);

        env.events().publish(
            (
                AllowanceClaimed {
                    project_id,
                    allowee: caller,
                    amount,
                },
            ),
            ()
        );

        Ok(())
    }

    pub fn allocate_item_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        allowee: Address,
        item_id: String,
        quantity: u64,
        expiry: Option<u64>
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::AllocItem)?;
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, "ngo")?;

        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
            .unwrap_or_else(|| Map::new(&env));

        allowances.set((allowee.clone(), item_id.clone()), ItemAllowance { quantity, expiry });

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()), &allowances);

        env.events().publish(
            (
                AllowanceAllocated {
                    project_id,
                    allowee,
                    currency_or_item: item_id,
                    amount: quantity,
                },
            ),
            ()
        );
        Ok(())
    }

    pub fn claim_item_allowance(
        env: Env,
        caller: Address,
        vendor: Address,
        project_id: String,
        item_id: String,
        quantity: u64
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::ClaimItem)?;
        require_role(&env, &caller, &project_id, "beneficiary")?;

        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        let mut ben_allowance: ItemAllowance = allowances
            .get((caller.clone(), item_id.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        check_expiry(&env, ben_allowance.expiry)?;
        if ben_allowance.quantity < quantity {
            return Err(ContractError::InsufficientFunds);
        }
        ben_allowance.quantity -= quantity;

        allowances.set((caller.clone(), item_id.clone()), ben_allowance);

        let mut vendor_allowance: ItemAllowance = allowances
            .get((vendor.clone(), item_id.clone()))
            .unwrap_or(ItemAllowance {
                quantity: 0,
                expiry: None,
            });
        vendor_allowance.quantity += quantity;
        allowances.set((vendor.clone(), item_id.clone()), vendor_allowance);

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()), &allowances);

        env.events().publish(
            (
                AllowanceClaimed {
                    project_id,
                    allowee: caller,
                    amount: quantity,
                },
            ),
            ()
        );

        Ok(())
    }

    // -- batch operations --
    pub fn allocate_cash_allowances_batch(
        env: Env,
        caller: Address,
        project_id: String,
        allowances: Vec<(Address, String, u64, Option<u64>)>
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::AllocCashBatch)?;
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, "ngo")?;

        let mut allowances_map: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
            .unwrap_or_else(|| Map::new(&env));

        for (allowee, currency, amount, expiry) in allowances {
            allowances_map.set((allowee.clone(), currency.clone()), CashAllowance {
                amount,
                expiry,
            });

            env.events().publish(
                (
                    AllowanceAllocated {
                        project_id: project_id.clone(),
                        allowee: allowee.clone(),
                        currency_or_item: currency,
                        amount,
                    },
                ),
                ()
            );
        }

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()), &allowances_map);

        Ok(())
    }

    pub fn allocate_item_allowances_batch(
        env: Env,
        caller: Address,
        project_id: String,
        allowances: Vec<(Address, String, u64, Option<u64>)>
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::AllocItemBatch)?;
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, "ngo")?;

        let mut allowances_map: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
            .unwrap_or_else(|| Map::new(&env));

        for (allowee, item_id, quantity, expiry) in allowances {
            allowances_map.set((allowee.clone(), item_id.clone()), ItemAllowance {
                quantity,
                expiry,
            });

            env.events().publish(
                (
                    AllowanceAllocated {
                        project_id: project_id.clone(),
                        allowee: allowee.clone(),
                        currency_or_item: item_id,
                        amount: quantity,
                    },
                ),
                ()
            );
        }

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()), &allowances_map);

        Ok(())
    }

    // -- transfer functions --
    pub fn transfer_cash_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        new_allowee: Address,
        currency: String,
        amount: u64
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::TransferCash)?;
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, "beneficiary")?;

        let mut allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        let mut src_allowance: CashAllowance = allowances
            .get((caller.clone(), currency.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        if src_allowance.amount < amount {
            return Err(ContractError::InsufficientFunds);
        }
        src_allowance.amount -= amount;

        let mut dest_allowance: CashAllowance = allowances
            .get((new_allowee.clone(), currency.clone()))
            .unwrap_or(CashAllowance {
                amount: 0,
                expiry: None,
            });
        dest_allowance.amount += amount;

        allowances.set((caller.clone(), currency.clone()), src_allowance);
        allowances.set((new_allowee.clone(), currency.clone()), dest_allowance);

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()), &allowances);

        env.events().publish(
            (
                AllowanceClaimed {
                    project_id,
                    allowee: caller,
                    amount,
                },
            ),
            ()
        );

        Ok(())
    }

    pub fn transfer_item_allowance(
        env: Env,
        caller: Address,
        project_id: String,
        new_allowee: Address,
        item_id: String,
        quantity: u64
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::TransferItem)?;
        check_paused(&env)?;
        require_role(&env, &caller, &project_id, "beneficiary")?;

        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        let mut src_allowance: ItemAllowance = allowances
            .get((caller.clone(), item_id.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        if src_allowance.quantity < quantity {
            return Err(ContractError::InsufficientFunds);
        }
        src_allowance.quantity -= quantity;

        let mut dest_allowance: ItemAllowance = allowances
            .get((new_allowee.clone(), item_id.clone()))
            .unwrap_or(ItemAllowance {
                quantity: 0,
                expiry: None,
            });
        dest_allowance.quantity += quantity;

        allowances.set((caller.clone(), item_id.clone()), src_allowance);
        allowances.set((new_allowee.clone(), item_id.clone()), dest_allowance);

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()), &allowances);

        env.events().publish(
            (
                AllowanceClaimed {
                    project_id,
                    allowee: caller,
                    amount: quantity,
                },
            ),
            ()
        );

        Ok(())
    }

    // -- redemption functions --
    pub fn redeem_item_claims(
        env: Env,
        vendor: Address,
        project_id: Option<String>,
        item_id: String,
        quantity: u64
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::RedeemItem)?;
        check_paused(&env)?;
        let project_id = project_id.unwrap_or(String::from_str(&env, "default_project"));

        require_role(&env, &vendor, &project_id, "vendor")?;
        let mut allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        let mut vendor_allowance: ItemAllowance = allowances
            .get((vendor.clone(), item_id.clone()))
            .ok_or(ContractError::AllowanceNotFound)?;

        if vendor_allowance.quantity < quantity {
            return Err(ContractError::InsufficientFunds);
        }

        vendor_allowance.quantity -= quantity;
        let item_key = (vendor.clone(), item_id.clone());
        if vendor_allowance.quantity == 0 {
            allowances.remove(item_key);
        } else {
            allowances.set(item_key.clone(), vendor_allowance);
        }

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id.clone()), &allowances);

        env.events().publish(
            (
                AllowanceClaimed {
                    project_id,
                    allowee: vendor,
                    amount: quantity,
                },
            ),
            ()
        );

        Ok(())
    }

    pub fn redeem_cash_claims(
        env: Env,
        vendor: Address,
        project_id: Option<String>,
        currency: String,
        amount: u64
    ) -> Result<(), ContractError> {
        non_reentrant(&env, ReentrancyLock::RedeemCash)?;

        check_paused(&env)?;
        let project_id = project_id.unwrap_or(String::from_str(&env, "default_project"));

        require_role(&env, &vendor, &project_id, "vendor")?;

        let mut allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()))
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

        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id.clone()), &allowances);

        env.events().publish(
            (
                AllowanceClaimed {
                    project_id,
                    allowee: vendor,
                    amount,
                },
            ),
            ()
        );

        Ok(())
    }

    // -- query functions --
    pub fn get_cash_allowance(
        env: Env,
        project_id: String,
        allowee: Address,
        currency: String
    ) -> Result<CashAllowance, ContractError> {
        let allowances: Map<(Address, String), CashAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id))
            .ok_or(ContractError::AllowanceNotFound)?;

        allowances.get((allowee, currency)).ok_or(ContractError::AllowanceNotFound)
    }

    pub fn get_item_allowance(
        env: Env,
        project_id: String,
        allowee: Address,
        item_id: String
    ) -> Result<ItemAllowance, ContractError> {
        let allowances: Map<(Address, String), ItemAllowance> = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
            .ok_or(ContractError::AllowanceNotFound)?;

        allowances.get((allowee, item_id)).ok_or(ContractError::AllowanceNotFound)
    }

    pub fn get_all_cash_allowances(
        env: Env,
        project_id: String
    ) -> Map<(Address, String), CashAllowance> {
        env.storage()
            .persistent()
            .get(&(Symbol::new(&env, CASH_ALLOWANCES_KEY), project_id))
            .unwrap_or_else(|| Map::new(&env))
    }

    pub fn get_all_item_allowances(
        env: Env,
        project_id: String
    ) -> Map<(Address, String), ItemAllowance> {
        env.storage()
            .persistent()
            .get(&(Symbol::new(&env, ITEM_ALLOWANCES_KEY), project_id))
            .unwrap_or_else(|| Map::new(&env))
    }

    pub fn get_total_cash_allowance(
        env: Env,
        beneficiary: Address,
        project_ids: Vec<String>
    ) -> u64 {
        let mut total_amount: u64 = 0;

        for project_id in project_ids.iter() {
            let allowances: Map<(Address, String), CashAllowance> = Self::get_all_cash_allowances(
                env.clone(),
                project_id.clone()
            );

            for ((allowee, _currency), allowance) in allowances.iter() {
                if allowee == beneficiary {
                    total_amount += allowance.amount;
                }
            }
        }

        total_amount
    }

    pub fn get_total_item_allowance(
        env: Env,
        beneficiary: Address,
        project_ids: Vec<String>
    ) -> u64 {
        let mut total_quantity: u64 = 0;

        for project_id in project_ids.iter() {
            let allowances: Map<(Address, String), ItemAllowance> = Self::get_all_item_allowances(
                env.clone(),
                project_id.clone()
            );

            for ((allowee, _item_id), allowance) in allowances.iter() {
                if allowee == beneficiary {
                    total_quantity += allowance.quantity;
                }
            }
        }

        total_quantity
    }

    pub fn get_roles(env: Env, project_id: String) -> Roles {
        get_roles(&env, &project_id)
    }
}

// -- helper functions --
fn check_paused(env: &Env) -> Result<(), ContractError> {
    if env.storage().persistent().get(&Symbol::new(env, PAUSED_KEY)).unwrap_or(false) {
        Err(ContractError::ContractPaused)
    } else {
        Ok(())
    }
}

fn check_expiry(env: &Env, expiry: Option<u64>) -> Result<(), ContractError> {
    if let Some(expiry) = expiry {
        if env.ledger().timestamp() > expiry {
            return Err(ContractError::ExpiredAllowance);
        }
    }
    Ok(())
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
    required_role: &str
) -> Result<(), ContractError> {
    caller.require_auth();
    let roles = get_roles(env, project_id);

    let has_access = match required_role {
        "super_admin" => roles.super_admins.contains(caller),
        "admin" => roles.admins.contains(caller) || roles.super_admins.contains(caller),
        "ngo" => roles.ngos.contains(caller) || roles.admins.contains(caller),
        "beneficiary" => roles.beneficiaries.contains(caller),
        "vendor" => roles.vendors.contains(caller),
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
    where T: Clone + TryFromVal<Env, Val> + IntoVal<Env, Val>
{
    let mut result = Vec::new(env);
    for item in iter {
        result.push_back(item);
    }
    result
}

fn non_reentrant(env: &Env, lock_type: ReentrancyLock) -> Result<(), ContractError> {
    // skip check for read-only functions
    match lock_type {
        | ReentrancyLock::GetCashAllowance
        | ReentrancyLock::GetItemAllowance
        | ReentrancyLock::GetAllCash
        | ReentrancyLock::GetAllItems
        | ReentrancyLock::GetTotalCash
        | ReentrancyLock::GetTotalItems
        | ReentrancyLock::GetRoles => {
            return Ok(());
        }
        _ => {}
    }

    let lock_key = lock_type.to_symbol(env);
    if env.storage().persistent().has(&lock_key) {
        return Err(ContractError::ReentrancyDetected);
    }
    env.storage().persistent().set(&lock_key, &true);
    Ok(())
}

fn check_role_limits(vec: &Vec<Address>, member: &Address) -> Result<(), ContractError> {
    if vec.contains(member) { Err(ContractError::RoleAlreadyExists) } else { Ok(()) }
}

#[cfg(test)]
mod test;
