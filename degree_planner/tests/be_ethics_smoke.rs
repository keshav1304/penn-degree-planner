use std::collections::HashMap;

use degree_planner::penn_data::seas_data::create_be_major;
use degree_planner::requirement::validate_courses_for_degree;

fn unit_cu(courses: &[&str]) -> HashMap<String, f64> {
    courses
        .iter()
        .map(|c| (c.to_string(), 1.0))
        .collect()
}

fn gen_ed_unfulfilled_count(validation: &degree_planner::requirement::DegreeValidationResult) -> usize {
    validation
        .unfulfilled
        .iter()
        .filter(|r| r.requirement.get_category() == "General Electives")
        .count()
}

#[test]
fn be_ethics_branch_a_eas_2030() {
    let major = create_be_major();
    let taken = vec![
        "EAS 2030".to_string(),
        "BEPP 2010".to_string(),
        "ENGL 0700".to_string(),
        "HIST 0100".to_string(),
        "ACCT 2110".to_string(),
        "PSCI 0100".to_string(),
        "BEPP 2020".to_string(),
    ];
    let cu_map = unit_cu(&[
        "EAS 2030", "BEPP 2010", "ENGL 0700", "HIST 0100", "ACCT 2110", "PSCI 0100", "BEPP 2020",
    ]);
    let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
    assert_eq!(
        gen_ed_unfulfilled_count(&validation),
        0,
        "Branch A (EAS 2030) should satisfy all General Electives"
    );
}

#[test]
fn be_ethics_branch_b_phil_1342() {
    let major = create_be_major();
    let taken = vec![
        "PHIL 1342".to_string(),
        "BEPP 2010".to_string(),
        "HSOC 1330".to_string(),
        "ENGL 0700".to_string(),
        "ACCT 2110".to_string(),
        "PSCI 0100".to_string(),
        "BEPP 2020".to_string(),
    ];
    let cu_map = unit_cu(&[
        "PHIL 1342", "BEPP 2010", "HSOC 1330", "ENGL 0700", "ACCT 2110", "PSCI 0100", "BEPP 2020",
    ]);
    let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
    assert_eq!(
        gen_ed_unfulfilled_count(&validation),
        0,
        "Branch B (PHIL 1342) should satisfy all General Electives"
    );
}

#[test]
fn be_ethics_branch_b_bioe_4010() {
    let major = create_be_major();
    let taken = vec![
        "BIOE 4010".to_string(),
        "BEPP 2010".to_string(),
        "HSOC 1330".to_string(),
        "ENGL 0700".to_string(),
        "ACCT 2110".to_string(),
        "PSCI 0100".to_string(),
        "BEPP 2020".to_string(),
    ];
    let cu_map = unit_cu(&[
        "BIOE 4010", "BEPP 2010", "HSOC 1330", "ENGL 0700", "ACCT 2110", "PSCI 0100", "BEPP 2020",
    ]);
    let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
    assert_eq!(
        gen_ed_unfulfilled_count(&validation),
        0,
        "Branch B (BIOE 4010) should satisfy all General Electives"
    );
}
