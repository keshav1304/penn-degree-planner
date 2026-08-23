use std::collections::HashMap;

use degree_planner::penn_data::seas_data::create_be_major;
use degree_planner::requirement::validate_courses_for_degree;

fn unit_cu(courses: &[&str]) -> HashMap<String, f64> {
    courses.iter().map(|c| (c.to_string(), 1.0)).collect()
}

fn gen_ed_unfulfilled_count(
    validation: &degree_planner::requirement::DegreeValidationResult,
) -> usize {
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
        "EAS 2030",
        "BEPP 2010",
        "ENGL 0700",
        "HIST 0100",
        "ACCT 2110",
        "PSCI 0100",
        "BEPP 2020",
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
        "PHIL 1342",
        "BEPP 2010",
        "HSOC 1330",
        "ENGL 0700",
        "ACCT 2110",
        "PSCI 0100",
        "BEPP 2020",
    ]);
    let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
    assert_eq!(
        gen_ed_unfulfilled_count(&validation),
        0,
        "Branch B (PHIL 1342) should satisfy all General Electives"
    );
}

#[test]
fn be_ethics_phil_1342_double_counts_ethics_and_one_distribution() {
    use degree_planner::requirement::evaluate_pool_constraints;

    let major = create_be_major();
    let pool_req = major
        .requirements
        .iter()
        .find(|r| matches!(r, degree_planner::Requirement::CoursePool { category, .. } if category.as_deref() == Some("General Electives")))
        .expect("General Electives pool");
    let degree_planner::Requirement::CoursePool { constraints, .. } = pool_req else {
        panic!("expected CoursePool");
    };

    let attributes = degree_planner::penn_data::attributes_data::create_attributes();
    let cu_map = unit_cu(&[
        "PHIL 1342",
        "BEPP 2010",
        "HSOC 1330",
        "ENGL 0700",
        "ACCT 2110",
        "PSCI 0100",
        "BEPP 2020",
    ]);
    let pool = vec![
        "PHIL 1342".to_string(),
        "BEPP 2010".to_string(),
        "HSOC 1330".to_string(),
        "ENGL 0700".to_string(),
        "ACCT 2110".to_string(),
        "PSCI 0100".to_string(),
        "BEPP 2020".to_string(),
    ];

    let evaluations = evaluate_pool_constraints(&pool, constraints, &attributes, &cu_map);
    assert!(evaluations.iter().all(|e| e.fulfilled));

    let ethics_eval = evaluations
        .iter()
        .find(|e| e.label == "Engineering Ethics")
        .expect("ethics constraint");
    assert_eq!(ethics_eval.course_ids, vec!["PHIL 1342"]);

    let phil_uses: Vec<_> = evaluations
        .iter()
        .filter(|e| e.course_ids == vec!["PHIL 1342"])
        .collect();
    assert_eq!(
        phil_uses.len(),
        2,
        "PHIL 1342 should satisfy ethics plus exactly one distribution slot"
    );
    assert_eq!(phil_uses[0].consumption_group, "be:ethics");
    assert_eq!(phil_uses[1].consumption_group, "be:distribution");

    let double_counted: Vec<_> = evaluations
        .iter()
        .filter(|e| {
            evaluations
                .iter()
                .filter(|other| other.course_ids == e.course_ids && !other.course_ids.is_empty())
                .count()
                > 1
        })
        .map(|e| e.course_ids[0].clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(
        double_counted,
        vec!["PHIL 1342"],
        "only the ethics course may double-count"
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
        "BIOE 4010",
        "BEPP 2010",
        "HSOC 1330",
        "ENGL 0700",
        "ACCT 2110",
        "PSCI 0100",
        "BEPP 2020",
    ]);
    let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
    assert_eq!(
        gen_ed_unfulfilled_count(&validation),
        0,
        "Branch B (BIOE 4010) should satisfy all General Electives"
    );
}
