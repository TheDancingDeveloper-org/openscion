//! Unit tests for scenario naming.

use super::Scenario;

#[test]
fn every_scenario_has_a_distinct_kebab_case_name() {
    let names: Vec<_> = Scenario::ALL.iter().map(ToString::to_string).collect();
    assert_eq!(
        names,
        ["import", "commit-one-file", "full-clone", "sparse-clone"]
    );
}
