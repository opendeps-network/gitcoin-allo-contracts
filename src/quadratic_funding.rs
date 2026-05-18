#![no_std]

use crate::fixed_point::FixedPoint;
use soroban_sdk::{Env, Map, Vec};

pub struct QuadraticFundingEngine {
    precision: i128,
}

#[derive(Clone)]
struct ProjectAggregate {
    total_contributed: FixedPoint,
    unique_donors: u32,
}

impl QuadraticFundingEngine {
    pub fn new(precision: i128) -> Self {
        QuadraticFundingEngine { precision }
    }

    pub fn calculate(
        &self,
        env: &Env,
        contributions: Vec<crate::Contribution>,
        matching_pool: i128,
    ) -> Map<u32, i128> {
        let mut aggregates: Map<u32, ProjectAggregate> = Map::new(env);
        let mut donor_set: Map<u32, Vec<Address>> = Map::new(env);

        let mut i: u32 = 0;
        while i < contributions.len() {
            let c = contributions.get(i).unwrap();
            let entry = aggregates.get(c.project_id);
            let donors = donor_set.get(c.project_id);

            let (agg, mut proj_donors) = match (entry, donors) {
                (Some(e), Some(d)) => (e, d),
                _ => (
                    ProjectAggregate {
                        total_contributed: FixedPoint::new(0, self.precision),
                        unique_donors: 0,
                    },
                    Vec::new(env),
                ),
            };

            let amount_fp = FixedPoint::new(c.amount * self.precision, self.precision);
            let updated_total = agg.total_contributed.add(&amount_fp);

            let mut updated_donors = agg.unique_donors;
            let mut is_new = true;
            let mut j: u32 = 0;
            while j < proj_donors.len() {
                if proj_donors.get(j).unwrap() == c.donor {
                    is_new = false;
                    break;
                }
                j += 1;
            }
            if is_new {
                proj_donors.push_back(c.donor.clone());
                updated_donors += 1;
            }

            let new_agg = ProjectAggregate {
                total_contributed: updated_total,
                unique_donors: updated_donors,
            };

            aggregates.set(c.project_id, new_agg);
            donor_set.set(c.project_id, proj_donors);
            i += 1;
        }

        let matching_pool_fp = FixedPoint::new(matching_pool * self.precision, self.precision);
        let mut total_sqrt_sum = FixedPoint::new(0, self.precision);

        let keys: Vec<u32> = aggregates.keys();
        let mut k: u32 = 0;
        while k < keys.len() {
            let pid = keys.get(k).unwrap();
            let agg = aggregates.get(pid).unwrap();
            let sqrt_per_donor = if agg.unique_donors > 0 {
                let per_donor_avg = agg.total_contributed.div(
                    &FixedPoint::new(agg.unique_donors as i128 * self.precision, self.precision)
                );
                per_donor_avg.sqrt()
            } else {
                FixedPoint::new(0, self.precision)
            };
            let donor_weight = FixedPoint::new(agg.unique_donors as i128 * self.precision, self.precision);
            let project_sqrt = sqrt_per_donor.mul(&donor_weight);
            total_sqrt_sum = total_sqrt_sum.add(&project_sqrt);
            k += 1;
        }

        let mut results: Map<u32, i128> = Map::new(env);
        let mut k2: u32 = 0;
        while k2 < keys.len() {
            let pid = keys.get(k2).unwrap();
            let agg = aggregates.get(pid).unwrap();
            let sqrt_per_donor = if agg.unique_donors > 0 {
                let per_donor_avg = agg.total_contributed.div(
                    &FixedPoint::new(agg.unique_donors as i128 * self.precision, self.precision)
                );
                per_donor_avg.sqrt()
            } else {
                FixedPoint::new(0, self.precision)
            };
            let donor_weight = FixedPoint::new(agg.unique_donors as i128 * self.precision, self.precision);
            let project_sqrt = sqrt_per_donor.mul(&donor_weight);

            let share = if total_sqrt_sum.value > 0 {
                project_sqrt.div(&total_sqrt_sum)
            } else {
                FixedPoint::new(0, self.precision)
            };

            let match_amount = share.mul(&matching_pool_fp);
            results.set(pid, match_amount.value / self.precision);
            k2 += 1;
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, vec};
    use crate::Contribution;

    #[test]
    fn test_single_project_single_donor() {
        let env = Env::default();
        let precision = 1_000_000_000;
        let engine = QuadraticFundingEngine::new(precision);

        let contributions = vec![
            &env,
            Contribution {
                donor: soroban_sdk::Address::random(&env),
                project_id: 1,
                amount: 100,
            },
        ];

        let results = engine.calculate(&env, contributions, 1000);
        assert_eq!(results.len(), 1);
        let amount = results.get(1).unwrap();
        assert!(amount > 0);
    }

    #[test]
    fn test_multiple_projects() {
        let env = Env::default();
        let precision = 1_000_000_000;
        let engine = QuadraticFundingEngine::new(precision);

        let donor_a = soroban_sdk::Address::random(&env);
        let donor_b = soroban_sdk::Address::random(&env);
        let donor_c = soroban_sdk::Address::random(&env);

        let contributions = vec![
            &env,
            Contribution { donor: donor_a.clone(), project_id: 1, amount: 100 },
            Contribution { donor: donor_b.clone(), project_id: 1, amount: 50 },
            Contribution { donor: donor_c.clone(), project_id: 2, amount: 200 },
        ];

        let results = engine.calculate(&env, contributions, 1000);
        let p1 = results.get(1).unwrap();
        let p2 = results.get(2).unwrap();
        assert!(p1 > p2);
    }
}
