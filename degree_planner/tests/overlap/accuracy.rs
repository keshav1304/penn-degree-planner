//! Accuracy net for cross-degree overlap discovery.
//!
//! Locks planner *results* (courses, slot pairs, schedule groups) so search
//! optimizations cannot silently change dual-degree behavior. Slow CIS/EE+WH
//! computes already live in `planner`; this file covers cases those tests
//! do not (empty plan, grad-grad claims, taken exclusion, remap, group ids).

use std::collections::{HashMap, HashSet};

use degree_planner::cross_degree::{CrossDegreeState, overlap_plan_applicable};
use degree_planner::major::resolve_major;
use degree_planner::overlap_planner::{
    OverlapPlan, OverlapSlotRef, compute_overlap_plan, is_overlap_schedule_group_id,
    overlap_group_schedule_id, remap_overlap_plan_degree_indices,
};
use degree_planner::penn_data::courses_data;
use degree_planner::requirement::validate_courses_for_degree;
use degree_planner::scheduler::{DegreeInput, ScheduleInput, generate_schedule};

use crate::common::assert_overlap_plan_accuracy;

struct DualOverlap {
    plan: OverlapPlan,
    per_degree: Vec<degree_planner::requirement::DegreeValidationResult>,
    schools: Vec<String>,
    majors: Vec<String>,
    taken: HashSet<String>,
}

fn catalog_cu_map() -> &'static HashMap<String, f64> {
    courses_data::cu_map()
}

fn compute_dual(
    school1: &str,
    major1: &str,
    concs1: &[&str],
    school2: &str,
    major2: &str,
    concs2: &[&str],
    taken: &[&str],
) -> DualOverlap {
    let concs1: Vec<String> = concs1.iter().map(|s| (*s).to_string()).collect();
    let concs2: Vec<String> = concs2.iter().map(|s| (*s).to_string()).collect();
    let m1 = resolve_major(school1, major1, &concs1)
        .unwrap_or_else(|| panic!("resolve {school1}:{major1}"));
    let m2 = resolve_major(school2, major2, &concs2)
        .unwrap_or_else(|| panic!("resolve {school2}:{major2}"));
    let taken_vec: Vec<String> = taken.iter().map(|s| (*s).to_string()).collect();
    let taken_set: HashSet<String> = taken_vec.iter().cloned().collect();
    let cu = catalog_cu_map();
    let per_degree = vec![
        validate_courses_for_degree(m1.requirements.clone(), &taken_vec, cu),
        validate_courses_for_degree(m2.requirements.clone(), &taken_vec, cu),
    ];
    let schools = vec![school1.to_string(), school2.to_string()];
    let majors = vec![major1.to_string(), major2.to_string()];
    let cross = CrossDegreeState::new(schools.clone(), majors.clone());
    let plan = compute_overlap_plan(
        &per_degree,
        &[&m1, &m2],
        &schools,
        &majors,
        &taken_set,
        &cross,
        cu,
        None,
    );
    DualOverlap {
        plan,
        per_degree,
        schools,
        majors,
        taken: taken_set,
    }
}

fn assert_dual_accuracy(dual: &DualOverlap, label: &str) {
    assert_overlap_plan_accuracy(
        &dual.plan,
        &dual.per_degree,
        &dual.schools,
        &dual.majors,
        &dual.taken,
        label,
    );
}

fn pair_labels(plan: &OverlapPlan) -> Vec<Vec<&str>> {
    plan.pairs
        .iter()
        .map(|p| p.slots.iter().map(|s| s.label.as_str()).collect())
        .collect()
}

fn is_writ_label(label: &str) -> bool {
    let lower = label.to_lowercase();
    lower.contains("writ") || lower.contains("writing sem")
}

fn plan_suggests(plan: &OverlapPlan, course: &str) -> bool {
    plan.opportunities
        .iter()
        .any(|o| o.suggested_courses.iter().any(|c| c == course))
}

#[test]
fn overlap_plan_applicable_requires_two_or_more_degrees() {
    assert!(!overlap_plan_applicable(&[]));
    assert!(!overlap_plan_applicable(&["SEAS".into()]));
    assert!(overlap_plan_applicable(&["SEAS".into(), "WH".into()]));
    assert!(overlap_plan_applicable(&["SEAS".into(), "SEAS_MS".into()]));
    assert!(overlap_plan_applicable(&[
        "SEAS_MS".into(),
        "SEAS_MS".into()
    ]));
}

#[test]
fn single_degree_compute_overlap_plan_is_empty() {
    let cis = resolve_major("SEAS", "CIS", &[]).expect("CIS");
    let cu = catalog_cu_map();
    let empty: Vec<String> = vec![];
    let per_degree = vec![validate_courses_for_degree(
        cis.requirements.clone(),
        &empty,
        cu,
    )];
    let schools = vec!["SEAS".into()];
    let majors = vec!["CIS".into()];
    let cross = CrossDegreeState::new(schools.clone(), majors.clone());
    let plan = compute_overlap_plan(
        &per_degree,
        &[&cis],
        &schools,
        &majors,
        &HashSet::new(),
        &cross,
        cu,
        None,
    );
    assert!(plan.opportunities.is_empty());
    assert!(plan.pairs.is_empty());
    assert!(plan.hints_by_slot.is_empty());
}

#[test]
fn two_graduate_degrees_may_discover_overlap_but_cannot_claim_the_same_course() {
    let dual = compute_dual("SEAS_MS", "MS_ROBO", &[], "SEAS_MS", "MS_EE", &[], &[]);
    assert_dual_accuracy(&dual, "MS_ROBO+MS_EE");

    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: "MS_ROBO".into(),
                school: "SEAS_MS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "MS_EE".into(),
                school: "SEAS_MS".into(),
                kind: "major".into(),
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
    if let Some(summary) = &output.cross_degree_summary {
        for (course, allocs) in &summary.course_allocations {
            let grad_degrees: HashSet<_> = allocs
                .iter()
                .filter(|a| a.school == "SEAS_MS")
                .map(|a| a.degree_index)
                .collect();
            assert!(
                grad_degrees.len() <= 1,
                "{course} must not be claimed on two graduate degrees: {allocs:?}"
            );
        }
    }
}

#[test]
fn neur_wh_overlap_is_accurate() {
    let dual = compute_dual("CAS", "NEUR", &[], "WH", "WH_NOFL", &["FNCE"], &[]);
    assert!(
        !dual.plan.opportunities.is_empty(),
        "NEUR+WH should find overlaps"
    );
    assert!(!dual.plan.pairs.is_empty(), "NEUR+WH should select pairs");
    assert_dual_accuracy(&dual, "NEUR+WH");

    assert!(
        dual.plan.pairs.iter().any(|p| {
            p.slots.iter().any(|s| s.label == "Unrestricted Electives")
                && p.slots
                    .iter()
                    .map(|s| s.degree_index)
                    .collect::<HashSet<_>>()
                    .len()
                    == 2
        }),
        "WH Unrestricted Electives must still pair as a catch-all; pairs: {:?}",
        pair_labels(&dual.plan)
    );

    let writ_opps: Vec<_> = dual
        .plan
        .opportunities
        .iter()
        .filter(|o| {
            o.slots.iter().any(|s| is_writ_label(&s.label))
                || o.suggested_courses.iter().any(|c| c.starts_with("WRIT "))
        })
        .collect();
    assert!(!writ_opps.is_empty(), "CAS+WH should surface WRIT overlap");
    for opp in writ_opps {
        assert!(
            opp.suggested_courses.iter().any(|c| c.starts_with("WRIT ")),
            "WRIT overlap should suggest WRIT courses, got {:?}",
            opp.suggested_courses
        );
    }
}

#[test]
fn cas_double_major_overlap_stays_major_slots_only() {
    let dual = compute_dual("CAS", "NEUR", &[], "CAS", "ECON", &[], &[]);
    assert_dual_accuracy(&dual, "NEUR+ECON");
    for opp in &dual.plan.opportunities {
        for slot in &opp.slots {
            assert!(
                slot.slot_key.starts_with("1:f"),
                "CAS double major overlap must be major-only, got {:?}",
                slot
            );
        }
        assert!(
            !opp.slots.iter().any(|s| is_writ_label(&s.label)),
            "CAS+CAS must not WRIT-overlap via the optimizer: {:?}",
            opp.explanation
        );
    }
}

#[test]
fn taken_courses_are_excluded_from_overlap_suggestions() {
    let dual = compute_dual(
        "CAS",
        "NEUR",
        &[],
        "WH",
        "WH_NOFL",
        &["FNCE"],
        &["CHEM 1011", "WRIT 0100"],
    );
    assert_dual_accuracy(&dual, "NEUR+WH taken");
    assert!(
        !plan_suggests(&dual.plan, "CHEM 1011"),
        "taken CHEM 1011 must not be re-suggested"
    );
    assert!(
        !plan_suggests(&dual.plan, "WRIT 0100"),
        "taken WRIT 0100 must not be re-suggested"
    );
    assert!(
        !dual.plan.opportunities.is_empty(),
        "remaining open slots should still yield overlap after taking shared courses"
    );
}

#[test]
fn overlap_group_id_is_order_independent_and_stable() {
    let a = OverlapSlotRef {
        degree_index: 1,
        slot_key: "3:p0".into(),
        label: "Elective".into(),
        school: "WH".into(),
        major: "WH_NOFL".into(),
    };
    let b = OverlapSlotRef {
        degree_index: 0,
        slot_key: "2:c1".into(),
        label: "Sector".into(),
        school: "CAS".into(),
        major: "NEUR".into(),
    };
    let forward = overlap_group_schedule_id(&[a.clone(), b.clone()]);
    let reverse = overlap_group_schedule_id(&[b, a]);
    assert_eq!(forward, reverse);
    assert_eq!(forward, "req:overlap:0@2:c1+1@3:p0");
    assert!(is_overlap_schedule_group_id(&forward));
    assert!(!is_overlap_schedule_group_id("req:0:2:c1"));
}

#[test]
fn remap_overlap_plan_degree_indices_rewrites_slots_and_hint_keys() {
    let mut dual = compute_dual("CAS", "NEUR", &[], "WH", "WH_NOFL", &["FNCE"], &[]);
    assert!(!dual.plan.pairs.is_empty());
    remap_overlap_plan_degree_indices(&mut dual.plan, &[2, 0]);

    for opp in &dual.plan.opportunities {
        for slot in &opp.slots {
            assert!(
                slot.degree_index == 0 || slot.degree_index == 2,
                "remapped degree_index should be 0 or 2, got {}",
                slot.degree_index
            );
        }
    }
    for key in dual.plan.hints_by_slot.keys() {
        assert!(
            key.starts_with("0:") || key.starts_with("2:"),
            "remapped hint key should use payload indices, got {key}"
        );
    }
    for key in dual.plan.slot_explanations.keys() {
        assert!(
            key.starts_with("0:") || key.starts_with("2:"),
            "remapped explanation key should use payload indices, got {key}"
        );
    }
}
