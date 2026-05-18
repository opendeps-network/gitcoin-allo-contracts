use super::*;
use soroban_sdk::{vec, Env, IntoVal, Symbol};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AlloProtocol);
    let client = AlloProtocolClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::random(&env);
    client.initialize(&admin);
}

#[test]
fn test_register_project() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AlloProtocol);
    let client = AlloProtocolClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::random(&env);
    client.initialize(&admin);

    let owner = soroban_sdk::Address::random(&env);
    let project_id = client.register_project(
        &owner,
        &Symbol::new(&env, "lodash"),
        &Symbol::new(&env, "critical dependency"),
    );
    assert_eq!(project_id, 1);
}

#[test]
fn test_contribute_and_matching() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AlloProtocol);
    let client = AlloProtocolClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::random(&env);
    let sponsor = soroban_sdk::Address::random(&env);
    let owner = soroban_sdk::Address::random(&env);

    client.initialize(&admin);
    client.fund_matching_pool(&sponsor, &10_000);

    let project_id = client.register_project(
        &owner,
        &Symbol::new(&env, "lodash"),
        &Symbol::new(&env, "critical"),
    );

    let donor1 = soroban_sdk::Address::random(&env);
    let donor2 = soroban_sdk::Address::random(&env);

    client.contribute(&donor1, &project_id, &50);
    client.contribute(&donor2, &project_id, &100);

    let results = client.calculate_matching();
    let match_amount = results.get(project_id).unwrap();
    assert!(match_amount > 0);
}

#[test]
fn test_multiple_projects_matching() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AlloProtocol);
    let client = AlloProtocolClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::random(&env);
    let sponsor = soroban_sdk::Address::random(&env);

    client.initialize(&admin);
    client.fund_matching_pool(&sponsor, &100_000);

    let p1 = client.register_project(
        &soroban_sdk::Address::random(&env),
        &Symbol::new(&env, "project-a"),
        &Symbol::new(&env, "desc a"),
    );
    let p2 = client.register_project(
        &soroban_sdk::Address::random(&env),
        &Symbol::new(&env, "project-b"),
        &Symbol::new(&env, "desc b"),
    );

    let donor = soroban_sdk::Address::random(&env);
    client.contribute(&donor, &p1, &100);
    client.contribute(&donor, &p2, &100);

    let results = client.calculate_matching();
    assert_eq!(results.len(), 2);
    let m1 = results.get(p1).unwrap();
    let m2 = results.get(p2).unwrap();
    assert!(m1 > 0);
    assert!(m2 > 0);
}
