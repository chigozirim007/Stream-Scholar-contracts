#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{token, Address, Env};

#[test]
fn test_scholarship_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a token for testing
    let token_address = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_address);
    token_client.mint(&student, &1000);

    // Deploy the scholarship contract
    let contract_id = env.register(ScholarContract, ());
    let client = ScholarContractClient::new(&env, &contract_id);

    // Initialize the contract with a rate of 10 tokens per second
    client.init(&10);

    // Student buys access to course 1 for 100 tokens (10 seconds)
    client.buy_access(&student, &1, &100, &token_address);

    // Verify token balance
    assert_eq!(token::Client::new(&env, &token_address).balance(&student), 900);
    assert_eq!(token::Client::new(&env, &token_address).balance(&contract_id), 100);

    // Verify access
    env.ledger().set_timestamp(0);
    assert!(client.has_access(&student, &1));

    // Fast forward 5 seconds - should still have access
    env.ledger().set_timestamp(5);
    assert!(client.has_access(&student, &1));

    // Fast forward 11 seconds - should no longer have access
    env.ledger().set_timestamp(11);
    assert!(!client.has_access(&student, &1));

    // Buy more access (another 10 seconds)
    client.buy_access(&student, &1, &100, &token_address);
    
    // Now should have access again (expires at current_time + 10 = 21)
    assert!(client.has_access(&student, &1));
    
    env.ledger().set_timestamp(20);
    assert!(client.has_access(&student, &1));
    
    env.ledger().set_timestamp(22);
    assert!(!client.has_access(&student, &1));
}
