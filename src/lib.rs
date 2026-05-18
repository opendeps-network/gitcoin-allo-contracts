#![no_std]

mod fixed_point;
mod quadratic_funding;

use fixed_point::FixedPoint;
use quadratic_funding::QuadraticFundingEngine;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub id: u32,
    pub owner: Address,
    pub name: Symbol,
    pub description: Symbol,
    pub match_multiplier: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    pub donor: Address,
    pub project_id: u32,
    pub amount: i128,
}

#[contracttype]
pub enum DataKey {
    Admin,
    MatchingPool,
    Project(u32),
    ContributionCount,
    Contribution(u32),
    ContributionIndex(u32, Address),
    MatchingResult(u32),
}

const MATCHING_POOL_PRECISION: i128 = 1_000_000_000;

#[contract]
pub struct AlloProtocol;

#[contractimpl]
impl AlloProtocol {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::MatchingPool, &0i128);
    }

    pub fn fund_matching_pool(env: Env, sponsor: Address, amount: i128) {
        sponsor.require_auth();
        let mut pool: i128 = env.storage().instance().get(&DataKey::MatchingPool).unwrap_or(0);
        pool += amount;
        env.storage().instance().set(&DataKey::MatchingPool, &pool);
    }

    pub fn register_project(env: Env, owner: Address, name: Symbol, description: Symbol) -> u32 {
        owner.require_auth();
        let project_id: u32 = env.storage().instance().get(&DataKey::ContributionCount).unwrap_or(0) + 1;
        let project = Project {
            id: project_id,
            owner,
            name,
            description,
            match_multiplier: 0i128,
        };
        env.storage().instance().set(&DataKey::Project(project_id), &project);
        env.storage().instance().set(&DataKey::ContributionCount, &project_id);
        project_id
    }

    pub fn contribute(env: Env, donor: Address, project_id: u32, amount: i128) {
        donor.require_auth();
        let _project: Project = env.storage().instance().get(&DataKey::Project(project_id))
            .unwrap_or_else(|| panic!("project not found"));

        let count: u32 = env.storage().instance().get(&DataKey::ContributionCount)
            .unwrap_or(0);
        let next_id = count + 1;

        let contribution = Contribution {
            donor: donor.clone(),
            project_id,
            amount,
        };
        env.storage().instance().set(&DataKey::Contribution(next_id), &contribution);
        env.storage().instance().set(&DataKey::ContributionIndex(project_id, donor), &next_id);
        env.storage().instance().set(&DataKey::ContributionCount, &next_id);
    }

    pub fn calculate_matching(env: Env) -> Map<u32, i128> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("not initialized"));
        admin.require_auth();

        let contribution_count: u32 = env.storage().instance().get(&DataKey::ContributionCount)
            .unwrap_or(0);
        let matching_pool: i128 = env.storage().instance().get(&DataKey::MatchingPool)
            .unwrap_or(0);

        let mut contributions: Vec<Contribution> = Vec::new(&env);
        for i in 1..=contribution_count {
            if let Some(c) = env.storage().instance().get::<_, Contribution>(&DataKey::Contribution(i)) {
                contributions.push_back(c);
            }
        }

        let engine = QuadraticFundingEngine::new(MATCHING_POOL_PRECISION);
        let results = engine.calculate(&env, contributions, matching_pool);

        for (project_id, amount) in results.iter() {
            env.storage().instance().set(&DataKey::MatchingResult(project_id), &amount);
        }

        results
    }

    pub fn get_matching_result(env: Env, project_id: u32) -> i128 {
        env.storage().instance()
            .get::<_, i128>(&DataKey::MatchingResult(project_id))
            .unwrap_or(0)
    }

    pub fn distribute(env: Env, project_id: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("not initialized"));
        admin.require_auth();

        let project: Project = env.storage().instance().get(&DataKey::Project(project_id))
            .unwrap_or_else(|| panic!("project not found"));
        let amount: i128 = env.storage().instance()
            .get::<_, i128>(&DataKey::MatchingResult(project_id))
            .unwrap_or(0);

        if amount > 0 {
            let contract_address = env.current_contract_address();
            contract_address.require_auth();
            env.balance().receive(&project.owner, amount);
            let mut pool: i128 = env.storage().instance().get(&DataKey::MatchingPool).unwrap_or(0);
            pool -= amount;
            env.storage().instance().set(&DataKey::MatchingPool, &pool);
        }
    }

    pub fn get_project(env: Env, project_id: u32) -> Project {
        env.storage().instance().get(&DataKey::Project(project_id))
            .unwrap_or_else(|| panic!("project not found"))
    }
}

mod test;
