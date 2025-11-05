#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Vec, symbol_short, Symbol};

// Transaction proposal structure
#[contracttype]
#[derive(Clone)]
pub struct Transaction {
    pub tx_id: u64,
    pub to: Address,
    pub amount: i128,
    pub approvals: u32,
    pub executed: bool,
}

// For storing transaction count
const TX_COUNT: Symbol = symbol_short!("TX_COUNT");

// For storing required signatures
const REQ_SIGS: Symbol = symbol_short!("REQ_SIGS");

// For storing owners
const OWNERS: Symbol = symbol_short!("OWNERS");

// Mapping transaction ID to Transaction
#[contracttype]
pub enum TxBook {
    Transaction(u64)
}

// Mapping transaction ID and owner to approval status
#[contracttype]
pub enum Approvals {
    Approval(u64, Address)
}

#[contract]
pub struct MultiSigWallet;

#[contractimpl]
impl MultiSigWallet {
    
    // Initialize the wallet with owners and required signatures
    pub fn initialize(env: Env, owners: Vec<Address>, required_sigs: u32) {
        // Check if already initialized
        if env.storage().instance().has(&OWNERS) {
            log!(&env, "Wallet already initialized");
            panic!("Already initialized");
        }
        
        // Validate inputs
        let owner_count = owners.len();
        if owner_count == 0 || required_sigs == 0 || required_sigs > owner_count {
            log!(&env, "Invalid parameters");
            panic!("Invalid parameters");
        }
        
        // Store owners and required signatures
        env.storage().instance().set(&OWNERS, &owners);
        env.storage().instance().set(&REQ_SIGS, &required_sigs);
        env.storage().instance().set(&TX_COUNT, &0u64);
        
        env.storage().instance().extend_ttl(5000, 5000);
        log!(&env, "MultiSig Wallet initialized with {} owners and {} required signatures", owner_count, required_sigs);
    }
    
    // Submit a transaction proposal
    pub fn submit_transaction(env: Env, proposer: Address, to: Address, amount: i128) -> u64 {
        proposer.require_auth();
        
        // Verify proposer is an owner
        let owners: Vec<Address> = env.storage().instance().get(&OWNERS).unwrap();
        let mut is_owner = false;
        for owner in owners.iter() {
            if owner == proposer {
                is_owner = true;
                break;
            }
        }
        
        if !is_owner {
            log!(&env, "Only owners can submit transactions");
            panic!("Not an owner");
        }
        
        // Create new transaction
        let mut tx_count: u64 = env.storage().instance().get(&TX_COUNT).unwrap_or(0);
        tx_count += 1;
        
        let transaction = Transaction {
            tx_id: tx_count,
            to: to.clone(),
            amount,
            approvals: 0,
            executed: false,
        };
        
        env.storage().instance().set(&TxBook::Transaction(tx_count), &transaction);
        env.storage().instance().set(&TX_COUNT, &tx_count);
        
        env.storage().instance().extend_ttl(5000, 5000);
        log!(&env, "Transaction {} submitted: {} tokens to {:?}", tx_count, amount, to);
        
        tx_count
    }
    
    // Approve a transaction
    pub fn approve_transaction(env: Env, approver: Address, tx_id: u64) {
        approver.require_auth();
        
        // Verify approver is an owner
        let owners: Vec<Address> = env.storage().instance().get(&OWNERS).unwrap();
        let mut is_owner = false;
        for owner in owners.iter() {
            if owner == approver {
                is_owner = true;
                break;
            }
        }
        
        if !is_owner {
            log!(&env, "Only owners can approve transactions");
            panic!("Not an owner");
        }
        
        // Check if already approved
        let approval_key = Approvals::Approval(tx_id, approver.clone());
        if env.storage().instance().has(&approval_key) {
            log!(&env, "Already approved by this owner");
            panic!("Already approved");
        }
        
        // Get transaction
        let mut transaction: Transaction = env.storage().instance()
            .get(&TxBook::Transaction(tx_id))
            .unwrap_or_else(|| panic!("Transaction not found"));
        
        if transaction.executed {
            log!(&env, "Transaction already executed");
            panic!("Already executed");
        }
        
        // Record approval
        env.storage().instance().set(&approval_key, &true);
        transaction.approvals += 1;
        
        env.storage().instance().set(&TxBook::Transaction(tx_id), &transaction);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Transaction {} approved. Total approvals: {}", tx_id, transaction.approvals);
    }
    
    // Execute transaction if enough approvals
    pub fn execute_transaction(env: Env, executor: Address, tx_id: u64) {
        executor.require_auth();
        
        // Verify executor is an owner
        let owners: Vec<Address> = env.storage().instance().get(&OWNERS).unwrap();
        let mut is_owner = false;
        for owner in owners.iter() {
            if owner == executor {
                is_owner = true;
                break;
            }
        }
        
        if !is_owner {
            log!(&env, "Only owners can execute transactions");
            panic!("Not an owner");
        }
        
        // Get transaction
        let mut transaction: Transaction = env.storage().instance()
            .get(&TxBook::Transaction(tx_id))
            .unwrap_or_else(|| panic!("Transaction not found"));
        
        if transaction.executed {
            log!(&env, "Transaction already executed");
            panic!("Already executed");
        }
        
        // Check if enough approvals
        let required_sigs: u32 = env.storage().instance().get(&REQ_SIGS).unwrap();
        if transaction.approvals < required_sigs {
            log!(&env, "Not enough approvals: {}/{}", transaction.approvals, required_sigs);
            panic!("Insufficient approvals");
        }
        
        // Mark as executed
        transaction.executed = true;
        env.storage().instance().set(&TxBook::Transaction(tx_id), &transaction);
        
        env.storage().instance().extend_ttl(5000, 5000);
        log!(&env, "Transaction {} executed: {} tokens sent to {:?}", tx_id, transaction.amount, transaction.to);
    }
}