//! Integration tests for the global 3 CU undergrad↔masters double-count cap.

use std::collections::{HashMap, HashSet};

use degree_planner::cross_degree::{
    self, crosses_undergrad_grad, enforce_claim_rules, is_graduate_degree, CrossDegreeState,
    CrossDegreeSummary, CrossDegreeViolationKind, UNDERGRAD_GRAD_CU_LIMIT,
};
use degree_planner::major::resolve_major;
use degree_planner::overlap_planner::compute_overlap_plan;
use degree_planner::penn_data::courses_data;
use degree_planner::requirement::{
    resolve_cross_degree_conflicts, validate_courses_for_degree,
};
use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

const CU_EPS: f64 = 0.001;

fn catalog_cu_map() -> &'static HashMap<String, f64> {
    courses_data::cu_map()
}

fn unit_cu(courses: &[&str], cu: f64) -> HashMap<String, f64> {
    courses
        .iter()
        .map(|c| (c.to_string(), cu))
        .collect()
}

fn mixed_cu_map() -> HashMap<String, f64> {
    HashMap::from([
        ("CIS 5190".into(), 1.0),
        ("CIS 5200".into(), 1.0),
        ("CIS 5210".into(), 1.0),
        ("ESE 3010".into(), 1.0),
        ("MEAM 1100".into(), 0.5),
    ])
}

fn shared_undergrad_grad_cu_from_claims(
    claims: &HashMap<String, HashSet<usize>>,
    degree_schools: &[String],
    cu_map: &HashMap<String, f64>,
) -> f64 {
    claims
        .iter()
        .filter(|(course, indices)| {
            degree_planner::course::is_valid_course_code(course)
                && crosses_undergrad_grad(course, indices, degree_schools)
        })
        .map(|(course, _)| cu_map.get(course.as_str()).copied().unwrap_or(1.0))
        .sum()
}

fn shared_undergrad_grad_cu_from_summary(
    summary: &CrossDegreeSummary,
    degree_schools: &[String],
    cu_map: &HashMap<String, f64>,
) -> f64 {
    summary
        .course_allocations
        .iter()
        .filter(|(course, allocs)| {
            degree_planner::course::is_valid_course_code(course)
                && allocs.iter().any(|a| a.uses_undergrad_grad_budget)
                && {
                    let idx: HashSet<_> = allocs.iter().map(|a| a.degree_index).collect();
                    crosses_undergrad_grad(course, &idx, degree_schools)
                }
        })
        .map(|(course, _)| cu_map.get(course.as_str()).copied().unwrap_or(1.0))
        .sum()
}

fn ug_ms_pair_count(
    plan: &degree_planner::overlap_planner::OverlapPlan,
) -> usize {
    plan.pairs
        .iter()
        .filter(|pair| {
            let schools: HashSet<_> = pair.slots.iter().map(|s| s.school.as_str()).collect();
            schools.contains("SEAS_MS") && schools.iter().any(|s| *s != "SEAS_MS")
        })
        .count()
}

fn cis_ms_schedule_input(taken: Vec<String>) -> ScheduleInput {
    ScheduleInput {
        taken,
        degrees: vec![
            DegreeInput {
                major: "CIS".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "MS_ROBO".into(),
                school: "SEAS_MS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
        ],
        frozen: vec![],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    }
}

#[test]
fn can_claim_allows_three_cu_then_rejects_fourth() {
    let schools = vec!["SEAS".into(), "SEAS_MS".into()];
    let mut state = CrossDegreeState::new(schools, vec!["CIS".into(), "MS_ROBO".into()]);
    let cu = unit_cu(&["CIS 5190", "CIS 5200", "CIS 5210", "ESE 3010"], 1.0);

    for course in ["CIS 5190", "CIS 5200", "CIS 5210"] {
        state.register_claim(course, 0, &cu);
        assert!(
            state.can_claim(course, 1, &cu).is_ok(),
            "{course} should be claimable on masters within budget"
        );
        state.register_claim(course, 1, &cu);
    }
    assert!(
        (state.undergrad_grad_cu_used - 3.0).abs() < CU_EPS,
        "expected 3 CU used, got {}",
        state.undergrad_grad_cu_used
    );

    state.register_claim("ESE 3010", 0, &cu);
    assert!(
        matches!(
            state.can_claim("ESE 3010", 1, &cu),
            Err(CrossDegreeViolationKind::UndergradGradCuCap)
        ),
        "fourth undergrad↔masters overlap should be rejected"
    );
}

#[test]
fn detect_violations_flags_budget_exceeded() {
    let schools = vec!["SEAS".into(), "SEAS_MS".into()];
    let cu = unit_cu(&["CIS 5190", "CIS 5200", "CIS 5210", "ESE 3010"], 1.0);
    let allocations = HashMap::from([
        ("CIS 5190".into(), HashSet::from([0, 1])),
        ("CIS 5200".into(), HashSet::from([0, 1])),
        ("CIS 5210".into(), HashSet::from([0, 1])),
        ("ESE 3010".into(), HashSet::from([0, 1])),
    ]);

    let violations = cross_degree::detect_violations(&allocations, &schools, &cu);
    assert!(
        violations
            .iter()
            .any(|v| v.kind == CrossDegreeViolationKind::UndergradGradCuCap),
        "expected global undergrad↔masters cap violation, got {:?}",
        violations
    );
}

#[test]
fn enforce_claim_rules_trims_to_three_cu() {
    let schools = vec!["SEAS".into(), "SEAS_MS".into()];
    let mut state = CrossDegreeState::new(schools.clone(), vec!["CIS".into(), "MS_ROBO".into()]);
    let cu = unit_cu(&["CIS 5190", "CIS 5200", "CIS 5210", "ESE 3010"], 1.0);

    for course in ["CIS 5190", "CIS 5200", "CIS 5210", "ESE 3010"] {
        state.register_claim(course, 0, &cu);
        state.register_claim(course, 1, &cu);
    }
    enforce_claim_rules(&mut state, &cu);

    let shared = shared_undergrad_grad_cu_from_claims(&state.claims, &schools, &cu);
    assert!(
        shared <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS,
        "shared CU {shared} exceeds cap"
    );
    assert!(
        shared >= UNDERGRAD_GRAD_CU_LIMIT - CU_EPS,
        "should keep as much overlap as allowed, got {shared}"
    );

    for (_, indices) in &state.claims {
        let grad_indices: Vec<_> = indices
            .iter()
            .copied()
            .filter(|&i| is_graduate_degree(&schools[i]))
            .collect();
        assert!(
            grad_indices.len() <= 1,
            "each course should appear on at most one graduate degree"
        );
    }
}

#[test]
fn fractional_cu_counts_toward_cap() {
    let schools = vec!["SEAS".into(), "SEAS_MS".into()];
    let mut state = CrossDegreeState::new(schools.clone(), vec!["CIS".into(), "MS_ROBO".into()]);
    let cu = mixed_cu_map();

    for course in ["CIS 5190", "CIS 5200", "CIS 5210"] {
        state.register_claim(course, 0, &cu);
        state.register_claim(course, 1, &cu);
    }
    assert!(state.can_claim("MEAM 1100", 0, &cu).is_ok());
    state.register_claim("MEAM 1100", 0, &cu);
    assert!(
        matches!(
            state.can_claim("MEAM 1100", 1, &cu),
            Err(CrossDegreeViolationKind::UndergradGradCuCap)
        ),
        "0.5 CU course should not fit when 3.0 CU already used"
    );

    let mut state2 = CrossDegreeState::new(schools, vec!["CIS".into(), "MS_ROBO".into()]);
    for course in ["CIS 5190", "CIS 5200"] {
        state2.register_claim(course, 0, &cu);
        state2.register_claim(course, 1, &cu);
    }
    state2.register_claim("MEAM 1100", 0, &cu);
    assert!(
        state2.can_claim("MEAM 1100", 1, &cu).is_ok(),
        "2.5 CU used should leave room for 0.5 CU overlap"
    );
}

#[test]
fn dual_undergrad_plus_masters_cap_is_global() {
    let schools = vec!["SEAS".into(), "WH".into(), "SEAS_MS".into()];
    let mut state = CrossDegreeState::new(
        schools.clone(),
        vec!["EE".into(), "WH_NOFL_MT".into(), "MS_ROBO".into()],
    );
    let cu = catalog_cu_map();

    state.register_claim("CIS 5190", 0, &cu);
    state.register_claim("CIS 5190", 2, &cu);
    state.register_claim("BEPP 2500", 1, &cu);
    state.register_claim("BEPP 2500", 2, &cu);
    state.register_claim("FNCE 1010", 1, &cu);
    state.register_claim("FNCE 1010", 2, &cu);
    state.register_claim("MGMT 2370", 0, &cu);
    state.register_claim("MGMT 2370", 2, &cu);

    enforce_claim_rules(&mut state, &cu);
    let shared = shared_undergrad_grad_cu_from_claims(&state.claims, &schools, &cu);
    assert!(
        shared <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS,
        "global cap should apply across EE and WH toward MS, got {shared}"
    );
}

#[test]
fn resolve_cross_degree_conflicts_trims_validation() {
    let schools = vec!["SEAS".into(), "SEAS_MS".into()];
    let majors = vec!["CIS".into(), "MS_ROBO".into()];
    let cu_map = catalog_cu_map();
    let taken = vec![
        "CIS 5190".into(),
        "CIS 5200".into(),
        "CIS 5210".into(),
        "ESE 3010".into(),
    ];

    let cis = resolve_major("SEAS", "CIS", &[]).expect("CIS");
    let ms = resolve_major("SEAS_MS", "MS_ROBO", &[]).expect("MS_ROBO");
    let mut per_degree = vec![
        validate_courses_for_degree(cis.requirements.clone(), &taken, &cu_map),
        validate_courses_for_degree(ms.requirements.clone(), &taken, &cu_map),
    ];

    let summary = resolve_cross_degree_conflicts(
        &mut per_degree,
        &schools,
        &majors,
        &cu_map,
        None,
        None,
        None,
    );

    let shared = shared_undergrad_grad_cu_from_summary(&summary, &schools, &cu_map);
    assert!(shared <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS);
    assert!(summary.violations.is_empty());

    let ms_shared: Vec<String> = per_degree[1]
        .fulfilled
        .iter()
        .chain(per_degree[1].unfulfilled.iter().filter(|m| m.partial))
        .flat_map(|m| m.course_ids.iter().cloned())
        .filter(|c| {
            degree_planner::course::is_valid_course_code(c)
                && summary
                    .course_allocations
                    .get(c)
                    .map(|a| {
                        let idx: HashSet<_> = a.iter().map(|x| x.degree_index).collect();
                        crosses_undergrad_grad(c, &idx, &schools)
                    })
                    .unwrap_or(false)
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    assert!(
        ms_shared.len() <= UNDERGRAD_GRAD_CU_LIMIT as usize,
        "MS validation should list at most 3 shared courses, got {:?}",
        ms_shared
    );
}

#[test]
fn overlap_plan_caps_undergrad_masters_pairs() {
    let cu_map = catalog_cu_map();
    let taken: Vec<String> = vec![];
    let ee = resolve_major("SEAS", "EE", &["Robotics".into()]).expect("EE");
    let wh = resolve_major("WH", "WH_NOFL_MT", &["FNCE".into()]).expect("WH");
    let ms = resolve_major("SEAS_MS", "MS_ROBO", &[]).expect("MS");
    let per_degree = vec![
        validate_courses_for_degree(ee.requirements.clone(), &taken, &cu_map),
        validate_courses_for_degree(wh.requirements.clone(), &taken, &cu_map),
        validate_courses_for_degree(ms.requirements.clone(), &taken, &cu_map),
    ];
    let schools = vec!["SEAS".into(), "WH".into(), "SEAS_MS".into()];
    let majors = vec!["EE".into(), "WH_NOFL_MT".into(), "MS_ROBO".into()];
    let cross = CrossDegreeState::new(schools.clone(), majors.clone());

    let plan = compute_overlap_plan(
        &per_degree,
        &[&ee, &wh, &ms],
        &schools,
        &majors,
        &taken.iter().cloned().collect(),
        &cross,
        &cu_map,
        None,
    );

    assert!(
        ug_ms_pair_count(&plan) <= UNDERGRAD_GRAD_CU_LIMIT as usize,
        "overlap plan selected {} undergrad↔masters pairs",
        ug_ms_pair_count(&plan)
    );
}

#[test]
fn schedule_enforces_cap_with_taken_overlapping_courses() {
    let output = generate_schedule(cis_ms_schedule_input(vec![
        "CIS 5190".into(),
        "CIS 5200".into(),
        "CIS 5210".into(),
        "ESE 3010".into(),
    ]));
    assert!(output.error.is_none(), "{:?}", output.error);

    let schools: Vec<String> = output
        .degree_results
        .iter()
        .map(|r| r.school.clone())
        .collect();
    let cu_map = catalog_cu_map();
    let summary = output.cross_degree_summary.as_ref().expect("summary");

    let shared_cu = shared_undergrad_grad_cu_from_summary(summary, &schools, &cu_map);
    assert!(shared_cu <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS);
    assert!(summary.violations.is_empty());

    let ug_ms_on_ms: Vec<String> = output.degree_results[1]
        .fulfilled_requirements
        .iter()
        .flat_map(|m| m.course_ids.iter().cloned())
        .filter(|c| {
            degree_planner::course::is_valid_course_code(c)
                && output.degree_results[0]
                    .fulfilled_requirements
                    .iter()
                    .any(|m| m.course_ids.iter().any(|id| id == c))
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    assert!(
        ug_ms_on_ms.len() <= UNDERGRAD_GRAD_CU_LIMIT as usize,
        "MS fulfilled should show at most 3 shared courses: {:?}",
        ug_ms_on_ms
    );
}

#[test]
fn schedule_triple_degree_caps_ug_ms_overlap_blocks() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: "EE".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec!["Robotics".into()],
                concentration: None,
            },
            DegreeInput {
                major: "WH_NOFL_MT".into(),
                school: "WH".into(),
                kind: "major".to_string(),
                concentrations: vec!["FNCE".into()],
                concentration: None,
            },
            DegreeInput {
                major: "MS_ROBO".into(),
                school: "SEAS_MS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
        ],
        frozen: vec![],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);

    let plan = output.overlap_plan.as_ref().expect("overlap plan");
    assert!(ug_ms_pair_count(plan) <= UNDERGRAD_GRAD_CU_LIMIT as usize);

    let ug_ms_groups = output
        .overlap_schedule_groups
        .iter()
        .filter(|g| {
            let has_ms = g.members.iter().any(|m| is_graduate_degree(&m.school));
            let has_ug = g.members.iter().any(|m| !is_graduate_degree(&m.school));
            has_ms && has_ug
        })
        .count();
    assert!(ug_ms_groups <= UNDERGRAD_GRAD_CU_LIMIT as usize);

    let schools: Vec<String> = output
        .degree_results
        .iter()
        .map(|r| r.school.clone())
        .collect();
    let shared_cu = shared_undergrad_grad_cu_from_summary(
        output.cross_degree_summary.as_ref().expect("summary"),
        &schools,
        &catalog_cu_map(),
    );
    assert!(shared_cu <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS);
}

#[test]
fn allocation_summary_marks_undergrad_grad_budget_courses() {
    let schools = vec!["SEAS".into(), "SEAS_MS".into()];
    let mut state = CrossDegreeState::new(schools, vec!["CIS".into(), "MS_ROBO".into()]);
    let cu = unit_cu(&["CIS 5190"], 1.0);
    state.register_claim("CIS 5190", 0, &cu);
    state.register_claim("CIS 5190", 1, &cu);

    let summary = state.to_summary();
    let allocs = summary
        .course_allocations
        .get("CIS 5190")
        .expect("CIS 5190 allocations");
    assert_eq!(allocs.len(), 2);
    assert!(allocs.iter().all(|a| a.uses_undergrad_grad_budget));
}

#[test]
fn pure_undergrad_overlap_does_not_consume_masters_budget() {
    let schools = vec!["SEAS".into(), "WH".into()];
    let mut state = CrossDegreeState::new(schools, vec!["EE".into(), "WH_NOFL".into()]);
    let cu = catalog_cu_map();
    state.register_claim("BEPP 2500", 0, &cu);
    state.register_claim("BEPP 2500", 1, &cu);
    state.register_claim("MATH 1400", 0, &cu);
    state.register_claim("MATH 1400", 1, &cu);

    assert!(
        state.undergrad_grad_cu_used.abs() < CU_EPS,
        "undergrad-only overlaps should not use masters budget"
    );
    let summary = state.to_summary();
    let budget_courses: Vec<_> = summary
        .course_allocations
        .iter()
        .filter(|(_, allocs)| allocs.iter().any(|a| a.uses_undergrad_grad_budget))
        .map(|(c, _)| c.clone())
        .collect();
    assert!(budget_courses.is_empty());
}
