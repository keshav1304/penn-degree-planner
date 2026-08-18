//! Shared overlap-plan accuracy assertions for integration tests.
//!
//! Used from `test_suite.rs` and `overlap_accuracy.rs` so existing dual
//! computes (CIS+WH, EE+WH, …) gain matcher checks without a second search.

use std::collections::HashSet;

use degree_planner::course;
use degree_planner::course_relations;
use degree_planner::overlap_planner::{hint_key, OverlapPlan, OverlapSlotRef};
use degree_planner::requirement::{requirement_accepts_shared_course, DegreeValidationResult};

pub const MAX_SUGGESTED: usize = 12;

pub fn slots_key(slots: &[OverlapSlotRef]) -> Vec<(usize, String)> {
    let mut keys: Vec<_> = slots
        .iter()
        .map(|s| (s.degree_index, s.slot_key.clone()))
        .collect();
    keys.sort();
    keys
}

pub fn is_pool_flex_key(slot_key: &str) -> bool {
    slot_key.split(':').any(|seg| {
        seg.len() > 1 && seg.starts_with('p') && seg[1..].chars().all(|c| c.is_ascii_digit())
    })
}

fn slot_accepts_course(
    slot: &OverlapSlotRef,
    course_code: &str,
    per_degree: &[DegreeValidationResult],
) -> bool {
    if !course::is_valid_course_code(course_code) {
        return false;
    }
    if is_pool_flex_key(&slot.slot_key) {
        return true;
    }
    match per_degree
        .get(slot.degree_index)
        .and_then(|v| v.mapped_for_instance(&slot.slot_key))
    {
        Some(mapped) => requirement_accepts_shared_course(&mapped.requirement, course_code),
        None => true,
    }
}

/// Structural + matcher invariants that overlap optimizations must preserve.
pub fn assert_overlap_plan_accuracy(
    plan: &OverlapPlan,
    per_degree: &[DegreeValidationResult],
    schools: &[String],
    majors: &[String],
    taken: &HashSet<String>,
    label: &str,
) {
    for opp in &plan.opportunities {
        let degrees: HashSet<_> = opp.slots.iter().map(|s| s.degree_index).collect();
        assert_eq!(
            opp.slots.len(),
            2,
            "{label}: opportunity must be a 2-slot pair: {:?}",
            opp.slots
        );
        assert_eq!(
            degrees.len(),
            2,
            "{label}: opportunity must span two degrees: {:?}",
            opp.slots
        );
        assert!(
            !opp.suggested_courses.is_empty()
                && opp.suggested_courses.len() <= MAX_SUGGESTED,
            "{label}: suggested_courses empty or >{MAX_SUGGESTED}: {:?}",
            opp.suggested_courses
        );

        for course_code in &opp.suggested_courses {
            assert!(
                course::is_valid_course_code(course_code),
                "{label}: invalid suggested code {course_code}"
            );
            assert!(
                !course_relations::set_contains_equiv(taken, course_code),
                "{label}: suggested {course_code} is already taken (or equivalent)"
            );
            assert!(
                !taken
                    .iter()
                    .any(|t| course_relations::codes_conflict(t, course_code)),
                "{label}: suggested {course_code} conflicts with taken {taken:?}"
            );
            for slot in &opp.slots {
                if let Some(expected) = schools.get(slot.degree_index) {
                    assert_eq!(&slot.school, expected, "{label}: slot school mismatch");
                }
                if let Some(expected) = majors.get(slot.degree_index) {
                    assert_eq!(&slot.major, expected, "{label}: slot major mismatch");
                }
                assert!(
                    slot_accepts_course(slot, course_code, per_degree),
                    "{label}: {course_code} does not satisfy {} {} ({}) — {}",
                    slot.major,
                    slot.label,
                    slot.slot_key,
                    opp.explanation
                );
            }
        }
    }

    let mut used_slots: HashSet<(usize, String)> = HashSet::new();
    for pair in &plan.pairs {
        let degrees: HashSet<_> = pair.slots.iter().map(|s| s.degree_index).collect();
        assert_eq!(pair.slots.len(), 2, "{label}: pair is not two slots");
        assert_eq!(degrees.len(), 2, "{label}: pair must span two degrees");
        let key = slots_key(&pair.slots);
        assert!(
            plan.opportunities
                .iter()
                .any(|opp| slots_key(&opp.slots) == key),
            "{label}: selected pair has no matching opportunity: {:?}",
            pair.explanation
        );
        for slot in &pair.slots {
            assert!(
                used_slots.insert((slot.degree_index, slot.slot_key.clone())),
                "{label}: slot {}:{} appears in two selected pairs",
                slot.degree_index,
                slot.slot_key
            );
            let hk = hint_key(slot.degree_index, &slot.slot_key);
            assert!(
                plan.hints_by_slot.contains_key(&hk),
                "{label}: pair slot missing hover hints ({hk})"
            );
            assert!(
                plan.slot_explanations.contains_key(&hk),
                "{label}: pair slot missing explanation ({hk})"
            );
        }
    }

    for (key, courses) in &plan.hints_by_slot {
        assert!(
            courses.len() <= MAX_SUGGESTED,
            "{label}: hints for {key} exceed {MAX_SUGGESTED}"
        );
        for course_code in courses {
            assert!(
                course::is_valid_course_code(course_code),
                "{label}: invalid hint {course_code} on {key}"
            );
            assert!(
                !course_relations::set_contains_equiv(taken, course_code),
                "{label}: hint {course_code} on {key} is already taken"
            );
        }
    }
}
