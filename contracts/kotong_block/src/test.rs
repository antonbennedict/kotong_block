#![cfg(test)]
use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, Symbol};

#[test]
fn test_happy_path_issue_and_pay_fine() {
    let env = Env::default();
    env.mock_all_auths();

    let enforcer = Address::generate(&env);
    let driver = Address::generate(&env);
    let treasury = Address::generate(&env);
    let ticket_id = Symbol::new(&env, "ABC123_2026");
    
    let token_address = env.register_stellar_asset_contract(Address::generate(&env));
    let token = token::Client::new(&env, &token_address);
    token.mint(&driver, &2000);

    let contract_id = env.register_contract(None, KotongBlockContract);
    let client = KotongBlockContractClient::new(&env, &contract_id);

    // Enforcer issues a 500 PHPT ticket
    client.issue_ticket(&enforcer, &driver, &ticket_id, &500);
    
    // Driver pays the fine
    client.pay_fine(&driver, &treasury, &token_address, &ticket_id);

    assert_eq!(token.balance(&treasury), 500);
    assert_eq!(token.balance(&driver), 1500);
}

#[test]
#[should_panic(expected = "Ticket already paid")]
fn test_prevent_double_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let driver = Address::generate(&env);
    let ticket_id = Symbol::new(&env, "XYZ789");
    let token_address = env.register_stellar_asset_contract(Address::generate(&env));
    token::Client::new(&env, &token_address).mint(&driver, &2000);
    
    let contract_id = env.register_contract(None, KotongBlockContract);
    let client = KotongBlockContractClient::new(&env, &contract_id);

    client.issue_ticket(&Address::generate(&env), &driver, &ticket_id, &500);
    client.pay_fine(&driver, &Address::generate(&env), &token_address, &ticket_id);
    client.pay_fine(&driver, &Address::generate(&env), &token_address, &ticket_id);
}

#[test]
fn test_state_verification_ticket_unpaid() {
    let env = Env::default();
    env.mock_all_auths();
    let enforcer = Address::generate(&env);
    let driver = Address::generate(&env);
    let ticket_id = Symbol::new(&env, "TKT001");
    
    let contract_id = env.register_contract(None, KotongBlockContract);
    let client = KotongBlockContractClient::new(&env, &contract_id);

    client.issue_ticket(&enforcer, &driver, &ticket_id, &1000);
    
    let violation: Violation = env.storage().persistent().get(&DataKey::Ticket(ticket_id)).unwrap();
    assert_eq!(violation.is_paid, false);
    assert_eq!(violation.amount, 1000);
}

#[test]
#[should_panic]
fn test_non_enforcer_cannot_issue_ticket() {
    let env = Env::default();
    let hacker = Address::generate(&env);
    let driver = Address::generate(&env);
    let ticket_id = Symbol::new(&env, "FAKE_TKT");
    
    let contract_id = env.register_contract(None, KotongBlockContract);
    let client = KotongBlockContractClient::new(&env, &contract_id);

    // No mock_all_auths, hacker tries to issue a ticket
    client.issue_ticket(&hacker, &driver, &ticket_id, &500);
}

#[test]
#[should_panic]
fn test_insufficient_funds_for_fine() {
    let env = Env::default();
    env.mock_all_auths();
    let driver = Address::generate(&env);
    let ticket_id = Symbol::new(&env, "TKT_LOW_BAL");
    let token_address = env.register_stellar_asset_contract(Address::generate(&env));
    
    let contract_id = env.register_contract(None, KotongBlockContract);
    let client = KotongBlockContractClient::new(&env, &contract_id);

    client.issue_ticket(&Address::generate(&env), &driver, &ticket_id, &500);
    // Driver balance is 0, payment will fail
    client.pay_fine(&driver, &Address::generate(&env), &token_address, &ticket_id);
}