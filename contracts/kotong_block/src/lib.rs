#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct Violation {
    pub driver: Address,
    pub enforcer: Address,
    pub amount: i128,
    pub is_paid: bool,
    pub timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Ticket(Symbol), // Keyed by Ticket ID (e.g., Plate Number + Date)
}

#[contract]
pub struct KotongBlockContract;

#[contractimpl]
impl KotongBlockContract {
    /// Enforcer registers a violation on-chain. This creates a permanent record.
    pub fn issue_ticket(env: Env, enforcer: Address, driver: Address, ticket_id: Symbol, amount: i128) {
        enforcer.require_auth();
        
        let violation = Violation {
            driver,
            enforcer,
            amount,
            is_paid: false,
            timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::Ticket(ticket_id), &violation);
    }

    /// Driver pays the fine. The PHPT goes to the Treasury (the contract owner).
    pub fn pay_fine(env: Env, driver: Address, treasury: Address, token: Address, ticket_id: Symbol) {
        driver.require_auth();
        let mut violation: Violation = env.storage().persistent().get(&DataKey::Ticket(ticket_id.clone())).unwrap();

        if violation.is_paid {
            panic!("Ticket already paid");
        }

        let token_client = token::Client::new(&env, &token);
        // Transfer fine from Driver to Government Treasury
        token_client.transfer(&driver, &treasury, &violation.amount);

        violation.is_paid = true;
        env.storage().persistent().set(&DataKey::Ticket(ticket_id), &violation);
        
        // Emit event for the "Paid" status to be reflected in the enforcer's app
        env.events().publish((Symbol::new(&env, "fine_paid"), driver), true);
    }
}