# Gitcoin Allo - Contracts

Soroban-optimized Quadratic Funding math engine for OpenDeps. Implements safe fixed-point decimal logic to compute square roots of token contributions for community-driven dependency matching pools.

## Overview

This contract implements Gitcoin's Quadratic Funding mechanism on Stellar/Soroban. Community members signal support for critical dependencies by contributing tokens; the matching pool is then distributed proportional to the square root of each project's unique donor-weighted contributions.

## Contract Functions

- `initialize(admin)` - Initialize contract with admin address
- `fund_matching_pool(sponsor, amount)` - Add funds to the matching pool
- `register_project(owner, name, description)` - Register a dependency project
- `contribute(donor, project_id, amount)` - Contribute to a project
- `calculate_matching()` - Compute QF distribution (admin only)
- `distribute(project_id)` - Send matching funds to project owner (admin only)
- `get_project(project_id)` - Get project details
- `get_matching_result(project_id)` - Get calculated match amount

## Build & Test

```bash
cargo build
cargo test
```

## Deploy

```bash
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/gitcoin_allo_contracts.wasm
```
