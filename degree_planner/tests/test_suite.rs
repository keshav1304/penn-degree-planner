//! Penn Degree Planner — consolidated test suite.
//!
//! Organized around what students actually need from the app:
//! - pick valid majors and understand catalog coverage
//! - see which requirements their courses satisfy
//! - plan dual degrees without breaking sharing rules
//! - discover overlap opportunities between degrees
//! - generate semester schedules that respect CU caps

use std::collections::{HashMap, HashSet};

use proptest::prelude::*;

use degree_planner::course;
use degree_planner::cross_degree::{
    self, crosses_undergrad_grad, cross_degree_optimizer_applicable, enforce_claim_rules,
    is_graduate_degree, overlap_plan_applicable,
    CrossDegreeState, CrossDegreeViolationKind, UNDERGRAD_GRAD_CU_LIMIT,
};
use degree_planner::major::{self, resolve_major};
use degree_planner::overlap_planner::{
    self, compute_overlap_plan, is_overlap_schedule_group_id, overlap_group_schedule_id,
    OverlapSlotRef,
};
use degree_planner::penn_data::{attributes_data, college_data, courses_data};
use degree_planner::requirement::{
    self, course_matches_restriction, evaluate_pool_constraints, expand_restriction_slots,
    extract_concentration_info, is_pool_constraint_slot_id, is_requirement_slot_id,
    is_schedulable_requirement_slot_id, validate_courses_for_degree,
    resolve_cross_degree_conflicts, requirement_accepts_shared_course,
    requirement_explicitly_lists_course,
    Requirement,
};
use degree_planner::schedule_template::{
    later_semesters, placement_semesters, scheduled, semester_order, ScheduleHint,
    ScheduleHintMode, Y1F, Y1S, Y2F, Y2S, Y3F, Y4F,
};
use degree_planner::scheduler::{
    self, default_semester_cu_limit, generate_schedule, undergrad_schedule_years, CU_EPS,
    DegreeInput, FrozenCourse, ScheduleInput,
};

// ─── Shared helpers ───────────────────────────────────────────────────────────

fn catalog_cu_map() -> &'static HashMap<String, f64> {
    courses_data::cu_map()
}

fn sample_cu_map() -> HashMap<String, f64> {
    HashMap::from([
        ("CIS 1200".into(), 1.0),
        ("CIS 5190".into(), 1.0),
        ("CIS 5200".into(), 1.0),
        ("CIS 5210".into(), 1.0),
        ("MATH 1400".into(), 1.0),
        ("STAT 4300".into(), 1.0),
        ("MEAM 1100".into(), 0.5),
        ("TEST 1000".into(), 1.0),
        ("TEST 1001".into(), 1.0),
        ("TEST 1002".into(), 1.0),
        ("TEST 1003".into(), 1.0),
    ])
}

fn one_cu_restriction() -> Requirement {
    Requirement::Restriction {
        category: Some("Test restriction".into()),
        department: Some(vec!["TEST".into()]),
        cu: None,
        level: None,
        max_level: None,
        attr: None,
        number: 1,
        excluding: None,
        no_school: None,
    }
}

fn dual_degree_input(
    school1: &str,
    major1: &str,
    school2: &str,
    major2: &str,
) -> ScheduleInput {
    let wh_conc = if school2 == "WH" || school1 == "WH" {
        Some("FNCE".to_string())
    } else {
        None
    };
    ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: major1.to_string(),
                school: school1.to_string(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: major2.to_string(),
                school: school2.to_string(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: wh_conc,
            },
        ],
        frozen: vec![],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    }
}

fn dual_degree_input_with_conc(
    school1: &str,
    major1: &str,
    conc1: Option<&str>,
    school2: &str,
    major2: &str,
    conc2: Option<&str>,
) -> ScheduleInput {
    ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: major1.to_string(),
                school: school1.to_string(),
                kind: "major".to_string(),
                concentrations: conc1.map(str::to_string).into_iter().collect(),
                concentration: conc1.map(str::to_string),
            },
            DegreeInput {
                major: major2.to_string(),
                school: school2.to_string(),
                kind: "major".to_string(),
                concentrations: conc2.map(str::to_string).into_iter().collect(),
                concentration: conc2.map(str::to_string),
            },
        ],
        frozen: vec![],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    }
}

/// Mirrors frontend `buildCourseMapFromDegreeResults` + allocation overlay.
fn course_has_stripe_mapping(output: &scheduler::ScheduleOutput, course: &str) -> bool {
    if !course::is_valid_course_code(course) {
        return false;
    }
    if output
        .cross_degree_summary
        .as_ref()
        .is_some_and(|s| s.course_allocations.contains_key(course))
    {
        return true;
    }
    output.degree_results.iter().any(|result| {
        let in_mapped = |mapped: &requirement::MappedRequirement| {
            mapped.course_ids.iter().any(|id| id == course)
        };
        result.fulfilled_requirements.iter().any(in_mapped)
            || result.suggested_for_unfulfilled.iter().any(in_mapped)
            || result
                .unfulfilled_requirements
                .iter()
                .any(|mapped| mapped.partial && in_mapped(mapped))
            || result.concentration_info.iter().any(|ci| {
                !ci.is_core
                    && ci
                        .matched_courses
                        .iter()
                        .flatten()
                        .any(|id| id == course)
            })
    })
}

fn assert_scheduled_courses_have_stripe_mapping(output: &scheduler::ScheduleOutput, label: &str) {
    assert!(output.error.is_none(), "{label}: {:?}", output.error);
    let unmapped: Vec<String> = output
        .schedule
        .iter()
        .flat_map(|plan| plan.courses.iter().cloned())
        .filter(|c| course::is_valid_course_code(c))
        .filter(|c| !course_has_stripe_mapping(output, c))
        .collect();
    assert!(
        unmapped.is_empty(),
        "{label}: scheduled courses missing degree stripe mapping: {:?}",
        unmapped
    );
}

/// True when this requirement subtree includes a 1-CU WRIT department restriction.
fn requirement_tree_has_writ_department(req: &Requirement) -> bool {
    match req {
        Requirement::Restriction {
            department,
            number,
            cu,
            ..
        } => {
            department.as_ref().is_some_and(|d| d.iter().any(|x| x == "WRIT"))
                && *number == 1
                && cu.is_none()
        }
        Requirement::AllOf { requirements, .. }
        | Requirement::Concentration { requirements, .. } => requirements
            .iter()
            .any(requirement_tree_has_writ_department),
        Requirement::AnyOf { possibilities, .. } => possibilities
            .iter()
            .any(requirement_tree_has_writ_department),
        Requirement::CourseGroup { possibilities, .. } => possibilities
            .iter()
            .any(requirement_tree_has_writ_department),
        Requirement::CoursePool { fixed_slots, .. } => fixed_slots
            .iter()
            .any(requirement_tree_has_writ_department),
        _ => false,
    }
}

fn major_has_writ_requirement(major: &major::Major) -> bool {
    major
        .requirements
        .iter()
        .any(requirement_tree_has_writ_department)
}

fn degree_input_has_writ(degree: &DegreeInput) -> bool {
    let mut concs: Vec<String> = if !degree.concentrations.is_empty() {
        major::normalize_degree_concentrations(&degree.school, &degree.concentrations)
    } else {
        degree.concentration.clone().into_iter().collect()
    };
    if concs.is_empty() && degree.school == "WH" {
        concs.push("FNCE".to_string());
    }
    resolve_major(&degree.school, &degree.major, &concs)
        .is_some_and(|m| major_has_writ_requirement(&m))
}

fn is_writ_slot_label(label: &str) -> bool {
    let lower = label.to_lowercase();
    lower.contains("writ")
        || lower.contains("writing sem")
        || lower.contains("writing seminar")
}

/// Count how many 1-CU WRIT units appear on the generated schedule grid.
fn writ_cu_units_on_schedule(output: &scheduler::ScheduleOutput) -> f64 {
    let cu_map = catalog_cu_map();
    let mut units = 0.0;

    for plan in &output.schedule {
        for course in &plan.courses {
            if course.starts_with("WRIT ") {
                units += cu_map.get(course).copied().unwrap_or(1.0);
            }
        }
    }

    let mut counted_overlap_groups: HashSet<String> = HashSet::new();
    for plan in &output.schedule {
        for slot in &plan.requirement_slots {
            if !is_overlap_schedule_group_id(slot) {
                continue;
            }
            let is_writ = output
                .overlap_schedule_groups
                .iter()
                .find(|g| g.group_id == *slot)
                .is_some_and(|group| {
                    group.members.iter().any(|m| is_writ_slot_label(&m.label))
                        || group
                            .members
                            .iter()
                            .any(|m| {
                                output
                                    .slot_labels
                                    .get(&m.schedule_slot_id)
                                    .is_some_and(|l| is_writ_slot_label(l))
                            })
                        || is_writ_slot_label(&group.explanation)
                });
            if is_writ {
                counted_overlap_groups.insert(slot.clone());
                units += 1.0;
            }
        }
    }

    for plan in &output.schedule {
        for slot in &plan.requirement_slots {
            if is_overlap_schedule_group_id(slot) || counted_overlap_groups.contains(slot) {
                continue;
            }
            let label = output.slot_labels.get(slot).map(String::as_str).unwrap_or("");
            if is_writ_slot_label(label) {
                units += 1.0;
            }
        }
    }

    units
}

fn overlap_plan_has_writ_opportunity(plan: &overlap_planner::OverlapPlan) -> bool {
    plan.opportunities.iter().any(|opp| {
        opp.slots.iter().any(|s| is_writ_slot_label(&s.label))
            || is_writ_slot_label(&opp.explanation)
            || opp
                .suggested_courses
                .iter()
                .any(|c| c.starts_with("WRIT "))
    })
}

fn schedule_input(label: &str, school1: &str, major1: &str, school2: &str, major2: &str) -> (String, ScheduleInput) {
    (label.to_string(), dual_degree_input(school1, major1, school2, major2))
}

fn implemented_dual_undergrad_pairs() -> Vec<(String, ScheduleInput)> {
    let cas_majors = ["NEUR", "ECON", "CIS"];
    let seas_majors = [("SEAS", "CIS"), ("SEAS", "EE"), ("SEAS", "MSE"), ("SEAS", "AI"), ("SEAS", "CMPE")];
    let wh_majors = ["WH_NOFL", "WH_FL", "WH_NOFL_MT", "WH_FL_MT"];
    let mut pairs = Vec::new();

    for cas in cas_majors {
        for wh in &wh_majors {
            pairs.push(schedule_input(
                &format!("{cas}+{wh}"),
                "CAS",
                cas,
                "WH",
                wh,
            ));
        }
    }
    for (school, seas) in seas_majors {
        for wh in &wh_majors {
            pairs.push(schedule_input(
                &format!("{seas}+{wh}"),
                school,
                seas,
                "WH",
                wh,
            ));
        }
    }
    pairs.push(schedule_input("NEUR+ECON", "CAS", "NEUR", "CAS", "ECON"));
    pairs
}

fn assert_schedule_respects_cu_limits(output: &scheduler::ScheduleOutput, label: &str) {
    let schools: Vec<String> = output
        .degree_results
        .iter()
        .map(|r| r.school.clone())
        .collect();
    for plan in &output.schedule {
        let limit = default_semester_cu_limit(&schools, plan.year, &plan.semester);
        assert!(
            plan.total_cu <= limit + CU_EPS,
            "{label}: Y{} {} has {:.1} CU (limit {:.1})",
            plan.year,
            plan.semester,
            plan.total_cu,
            limit
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Catalog & major resolution — can the student pick a valid plan?
// ═══════════════════════════════════════════════════════════════════════════════

mod catalog {
    use super::*;

    #[test]
    fn every_catalog_school_lists_at_least_one_major() {
        for school in major::degree_catalog() {
            assert!(
                !school.majors.is_empty(),
                "{} should expose majors",
                school.school_code
            );
        }
    }

    #[test]
    fn math_major_resolves_with_concentrations() {
        use degree_planner::Requirement;

        fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
            if pred(req) {
                return true;
            }
            match req {
                Requirement::AnyOf { possibilities, .. }
                | Requirement::CourseGroup { possibilities, .. } => possibilities
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                Requirement::AllOf { requirements, .. }
                | Requirement::Concentration { requirements, .. } => requirements
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                _ => false,
            }
        }

        fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
            major
                .requirements
                .iter()
                .any(|r| requirement_tree_contains(r, pred))
        }

        let general =
            resolve_major("CAS", "MATH", &["General Mathematics".into()]).expect("MATH general");
        assert_eq!(general.short_name, "MATH");
        assert!(general.concentrations.is_some());
        assert!(
            major_contains(&general, &|req| {
                matches!(
                    req,
                    Requirement::SingleCourse { possibilities, .. }
                        if possibilities.contains(&"MATH 3001".to_string())
                )
            }),
            "General Mathematics should include MATH 3001"
        );
        assert!(
            major_contains(&general, &|req| {
                matches!(
                    req,
                    Requirement::SingleCourse { possibilities, .. }
                        if possibilities.contains(&"MATH 4100".to_string())
                )
            }),
            "General Mathematics should include MATH 4100"
        );
        assert!(
            major_contains(&general, &|req| {
                matches!(
                    req,
                    Requirement::SingleCourse { possibilities, .. }
                        if possibilities.contains(&"MATH 3710".to_string())
                )
            }),
            "General Mathematics algebra should include MATH 3710 pair option"
        );
        assert!(
            major_contains(&general, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, .. }
                        if category.as_deref() == Some("Mathematics Electives")
                )
            }),
            "General Mathematics should include mathematics electives"
        );
        assert!(
            !major_contains(&general, &|req| {
                matches!(
                    req,
                    Requirement::SingleCourse { possibilities, .. }
                        if possibilities == &vec!["MATH 3200".to_string()]
                )
            }),
            "General Mathematics should not require MATH 3200 statistics"
        );

        let bio = resolve_major("CAS", "MATH", &["Biological Mathematics".into()]).expect("MATH bio");
        assert!(
            !major_contains(&bio, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, .. }
                        if category.as_deref() == Some("Mathematics Electives")
                )
            }),
            "Biological Mathematics should not include math electives"
        );
        assert!(
            major_contains(&bio, &|req| {
                matches!(
                    req,
                    Requirement::SingleCourse { possibilities, .. }
                        if possibilities.contains(&"STAT 4310".to_string())
                )
            }),
            "Biological Mathematics should include STAT 4310"
        );
        let bio_conc_in_pool = bio.requirements.iter().any(|r| {
            let Requirement::CoursePool { fixed_slots, .. } = r else {
                return false;
            };
            fixed_slots.iter().any(|slot| {
                matches!(
                    slot,
                    Requirement::Concentration { category, .. }
                        if category.as_deref() == Some("Biological Mathematics")
                )
            })
        });
        assert!(
            bio_conc_in_pool,
            "Biological Mathematics concentration should live in the major course pool"
        );
        assert!(major::major_is_implemented("CAS", "MATH"));
    }

    #[test]
    fn psyc_major_resolves_with_distribution_and_electives() {
        use degree_planner::Requirement;

        fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
            if pred(req) {
                return true;
            }
            match req {
                Requirement::AnyOf { possibilities, .. }
                | Requirement::CourseGroup { possibilities, .. } => possibilities
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                Requirement::AllOf { requirements, .. }
                | Requirement::Concentration { requirements, .. } => requirements
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                _ => false,
            }
        }

        fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
            major
                .requirements
                .iter()
                .any(|r| requirement_tree_contains(r, pred))
        }

        let psyc = resolve_major("CAS", "PSYC", &[]).expect("PSYC major");
        assert_eq!(psyc.short_name, "PSYC");
        assert!(
            major_contains(&psyc, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, .. }
                        if category.as_deref() == Some("Introductory Psychology")
                )
            }),
            "Psychology should include introductory requirement"
        );
        assert!(
            major_contains(&psyc, &|req| {
                matches!(
                    req,
                    Requirement::Restriction { category, attr, .. }
                        if category.as_deref() == Some("Biological Basis of Behavior")
                            && attr.as_ref().is_some_and(|a| a.contains(&"APCI".to_string()))
                )
            }),
            "Psychology should include APCI biological elective"
        );
        assert!(
            major_contains(&psyc, &|req| {
                matches!(
                    req,
                    Requirement::Restriction {
                        category,
                        department,
                        level: Some(4000),
                        excluding,
                        ..
                    } if category.as_deref() == Some("Research Experience")
                        && department.as_ref().is_some_and(|d| d.contains(&"PSYC".to_string()))
                        && excluding.as_ref().is_some_and(|e| e.contains(&"PSYC 4997".to_string()))
                )
            }),
            "Psychology research requirement should exclude PSYC 4997"
        );
        assert!(
            major_contains(&psyc, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, .. }
                        if category.as_deref() == Some("Psychology Electives")
                )
            }),
            "Psychology should include four elective slots"
        );
        assert!(major::major_is_implemented("CAS", "PSYC"));
    }

    #[test]
    fn bsn_major_resolves_with_requirements() {
        use degree_planner::Requirement;

        fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
            if pred(req) {
                return true;
            }
            match req {
                Requirement::AnyOf { possibilities, .. }
                | Requirement::CourseGroup { possibilities, .. } => possibilities
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                Requirement::AllOf { requirements, .. }
                | Requirement::Concentration { requirements, .. } => requirements
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                _ => false,
            }
        }

        fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
            major
                .requirements
                .iter()
                .any(|r| requirement_tree_contains(r, pred))
        }

        let bsn = resolve_major("NURS", "BSN", &[]).expect("BSN major");
        assert_eq!(bsn.short_name, "BSN");
        assert_eq!(bsn.requirements.len(), 30);
        assert!(
            major_contains(&bsn, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, .. }
                        if category.as_deref() == Some("Writing Requirement")
                )
            }),
            "BSN should include writing requirement"
        );
        assert!(
            major_contains(&bsn, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, possibilities, .. }
                        if category.as_deref() == Some("Language Requirement 1")
                            && possibilities.iter().any(|child| matches!(
                                child,
                                Requirement::Restriction { attr, .. }
                                    if attr.as_ref().is_some_and(|a| a.contains(&"WUFL".to_string()))
                            ))
                )
            }),
            "BSN language slots should use WUFL attribute per Nursing handbook"
        );
        assert!(
            major_contains(&bsn, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, .. }
                        if category.as_deref() == Some("The Planet & Our Climate")
                )
            }),
            "BSN should include planet sector"
        );
        assert!(
            major_contains(&bsn, &|req| {
                matches!(
                    req,
                    Requirement::Restriction {
                        category,
                        no_school: Some(school),
                        ..
                    } if category.as_deref() == Some("Exploration Course Requirement")
                        && school == "NURS"
                )
            }),
            "BSN should include exploration requirement outside Nursing"
        );
        assert!(
            major_contains(&bsn, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, .. }
                        if category.as_deref() == Some("Ethics Requirement")
                )
            }),
            "BSN should include ethics AnyOf"
        );
        assert!(
            major_contains(&bsn, &|req| {
                matches!(
                    req,
                    Requirement::Restriction {
                        category,
                        department,
                        level: Some(3510),
                        max_level: Some(3690),
                        ..
                    } if category.as_deref() == Some("Nursing Case Study")
                        && department.as_ref().is_some_and(|d| d.contains(&"NURS".to_string()))
                )
            }),
            "BSN should include case study restriction"
        );
        assert!(major::major_is_implemented("NURS", "BSN"));
        assert!(major::major_is_implemented("NURS", "BSN_NOFL"));
    }

    #[test]
    fn nutr_bsn_major_resolves_with_nutrition_requirements() {
        use degree_planner::Requirement;

        fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
            if pred(req) {
                return true;
            }
            match req {
                Requirement::AnyOf { possibilities, .. }
                | Requirement::CourseGroup { possibilities, .. } => possibilities
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                Requirement::AllOf { requirements, .. }
                | Requirement::Concentration { requirements, .. } => requirements
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                    .iter()
                    .any(|child| requirement_tree_contains(child, pred)),
                _ => false,
            }
        }

        fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
            major
                .requirements
                .iter()
                .any(|r| requirement_tree_contains(r, pred))
        }

        let nutr = resolve_major("NURS", "NUTR_BSN", &[]).expect("NUTR_BSN major");
        assert_eq!(nutr.short_name, "NUTR_BSN");
        assert_eq!(nutr.requirements.len(), 37);
        assert!(
            major_contains(&nutr, &|req| {
                matches!(
                    req,
                    Requirement::SingleCourse { possibilities, .. }
                        if possibilities.contains(&"NURS 3120".to_string())
                            || possibilities.contains(&"NURS 5230".to_string())
                            || possibilities.contains(&"NURS 5240".to_string())
                )
            }),
            "Nutrition Science BSN should include required nutrition courses"
        );
        let nune_count = nutr
            .requirements
            .iter()
            .filter(|req| {
                matches!(
                    req,
                    Requirement::Restriction { attr, .. }
                        if attr.as_ref().is_some_and(|a| a.contains(&"NUNE".to_string()))
                )
            })
            .count();
        assert_eq!(
            nune_count, 4,
            "Nutrition Science BSN should include four NUNE electives"
        );
        assert!(
            major_contains(&nutr, &|req| {
                matches!(
                    req,
                    Requirement::AnyOf { category, .. }
                        if category.as_deref() == Some("Diversity, Universality, Justice, & Equity")
                )
            }),
            "Nutrition Science BSN should use DUJE sector label"
        );
        assert!(major::major_is_implemented("NURS", "NUTR_BSN"));
        assert!(major::major_is_implemented("NURS", "NUTR_BSN_NOFL"));
    }

    #[test]
    fn bsn_nofl_uses_free_electives_not_language() {
        let nofl = resolve_major("NURS", "BSN_NOFL", &[]).expect("BSN_NOFL major");
        assert!(
            nofl.requirements.iter().any(|req| {
                matches!(
                    req,
                    Requirement::Restriction { category, .. }
                        if category.as_deref() == Some("Free Elective 1")
                )
            }),
            "NOFL variant should use free electives"
        );
        assert!(
            !nofl.requirements.iter().any(|req| {
                matches!(
                    req,
                    Requirement::Restriction { category, .. }
                        if category
                            .as_deref()
                            .is_some_and(|c| c.starts_with("Language Requirement"))
                )
            }),
            "NOFL variant should not include language slots"
        );
    }

    #[test]
    fn cas_lists_all_college_majors() {
        assert_eq!(college_data::CAS_DEGREE_CATALOG.len(), 56);
        assert!(college_data::cas_catalog_entry("BIOL").is_some());
        assert!(college_data::cas_catalog_entry("NOT_A_MAJOR").is_none());
    }

    #[test]
    fn minor_catalog_includes_eent() {
        let catalog = major::minor_catalog();
        let seas = catalog
            .iter()
            .find(|s| s.school_code == "SEAS")
            .expect("SEAS in minor catalog");
        assert!(
            seas.majors.iter().any(|m| m.api_code == "EENT"),
            "Engineering Entrepreneurship minor should be selectable"
        );
    }

    #[test]
    fn minor_catalog_includes_data_science() {
        let catalog = major::minor_catalog();
        let seas = catalog
            .iter()
            .find(|s| s.school_code == "SEAS")
            .expect("SEAS in minor catalog");
        assert!(
            seas.majors.iter().any(|m| m.api_code == "DATA_SCI"),
            "Data Science minor should be selectable"
        );
    }

    #[test]
    fn data_science_minor_resolves_six_cu() {
        let minor = major::resolve_minor("SEAS", "DATA_SCI", &[])
            .expect("DATA_SCI minor resolves");
        assert_eq!(minor.short_name, "DATA_SCI");
        assert_eq!(minor.name, "Data Science");

        let expanded = requirement::expand_restriction_slots(minor.requirements.clone());
        assert_eq!(
            expanded.len(),
            5,
            "4 core SingleCourse rows + 1 CourseGroup electives"
        );
        assert!(
            expanded
                .iter()
                .filter(|r| matches!(r, Requirement::SingleCourse { .. }))
                .count()
                == 4,
            "core requirements should be SingleCourse rows"
        );
        assert!(
            expanded.iter().any(|r| matches!(
                r,
                Requirement::CourseGroup {
                    category: Some(cat),
                    number: 2,
                    ..
                } if cat == "Data Science Electives"
            )),
            "electives should be a CourseGroup of 2 from 5 areas"
        );
    }

    #[test]
    fn minor_catalog_includes_wh_stat_data_science() {
        let catalog = major::minor_catalog();
        let wh = catalog
            .iter()
            .find(|s| s.school_code == "WH")
            .expect("WH in minor catalog");
        assert!(
            wh.majors.iter().any(|m| m.api_code == "STAT_DS"),
            "Statistics and Data Science minor should be selectable"
        );
    }

    #[test]
    fn stat_data_science_minor_resolves_seven_cu() {
        use degree_planner::Requirement;

        let minor = major::resolve_minor("WH", "STAT_DS", &[])
            .expect("STAT_DS minor resolves");
        assert_eq!(minor.short_name, "STAT_DS");
        assert_eq!(minor.name, "Statistics and Data Science");

        let expanded = requirement::expand_restriction_slots(minor.requirements.clone());
        assert_eq!(
            expanded.len(),
            7,
            "3 core SingleCourse rows + 4 STAT elective restriction slots"
        );
        assert_eq!(
            expanded
                .iter()
                .filter(|r| matches!(r, Requirement::SingleCourse { .. }))
                .count(),
            3,
            "core requirements should be SingleCourse rows"
        );
        let elective_slots: Vec<_> = expanded
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    Requirement::Restriction {
                        department: Some(depts),
                        level: Some(4050),
                        ..
                    } if depts == &["STAT".to_string()]
                )
            })
            .collect();
        assert_eq!(
            elective_slots.len(),
            4,
            "electives should be four STAT restriction slots at min level 4050"
        );
    }

    #[test]
    fn minor_catalog_includes_cas_math() {
        let catalog = major::minor_catalog();
        let cas = catalog
            .iter()
            .find(|s| s.school_code == "CAS")
            .expect("CAS in minor catalog");
        assert!(
            cas.majors.iter().any(|m| m.api_code == "MATH"),
            "Mathematics minor should be selectable"
        );
    }

    #[test]
    fn all_concentrations_separates_math_major_and_minor() {
        let map = major::all_concentrations();
        let major_concs = map
            .get("CAS:MATH:major")
            .expect("CAS Mathematics major concentrations");
        assert!(major_concs.contains(&"Biological Mathematics".to_string()));
        let minor_concs = map
            .get("CAS:MATH:minor")
            .expect("CAS Mathematics minor entry");
        assert!(
            minor_concs.is_empty(),
            "Mathematics minor has no concentrations: {:?}",
            minor_concs
        );
    }

    #[test]
    fn math_minor_has_no_concentrations() {
        let concs = major::concentrations_for_program("CAS", "MATH", "minor");
        assert!(
            concs.is_empty(),
            "Mathematics minor has no concentrations per catalog"
        );
        let major_concs = major::concentrations_for("CAS", "MATH");
        assert!(
            major_concs.contains(&"Biological Mathematics".to_string()),
            "MATH major still exposes concentrations"
        );
    }

    #[test]
    fn math_minor_has_no_biological_mathematics_requirements() {
        let minor = major::resolve_minor("CAS", "MATH", &["Biological Mathematics".into()])
            .expect("CAS Mathematics minor resolves");
        fn categories(reqs: &[requirement::Requirement]) -> Vec<String> {
            reqs.iter()
                .flat_map(|r| {
                    let mut cats = vec![r.get_category()];
                    match r {
                        requirement::Requirement::AnyOf { possibilities, .. }
                        | requirement::Requirement::AllOf { requirements: possibilities, .. } => {
                            cats.extend(categories(possibilities));
                        }
                        _ => {}
                    }
                    cats
                })
                .filter(|c| !c.is_empty())
                .collect()
        }
        let cats = categories(&minor.requirements);
        assert!(
            !cats.iter().any(|c| c == "Biological Mathematics"),
            "minor requirements should not include Biological Mathematics: {:?}",
            cats
        );
        assert!(minor.concentrations.is_none());
    }

    #[test]
    fn math_minor_resolves_seven_cu_minimum() {
        let minor = major::resolve_minor("CAS", "MATH", &[])
            .expect("CAS Mathematics minor resolves");
        assert_eq!(minor.short_name, "MATH");

        let expanded = requirement::expand_restriction_slots(minor.requirements.clone());
        assert_eq!(
            expanded.len(),
            7,
            "MATH minor: 2 calculus + lin alg/proofs + proof-based + 3 electives"
        );
    }

    #[test]
    fn degree_catalog_excludes_minors() {
        let catalog = major::degree_catalog();
        for school in &catalog {
            assert!(
                !school.majors.iter().any(|m| m.api_code == "EENT"),
                "minors should not appear in degree catalog"
            );
        }
    }

    #[test]
    fn eent_minor_resolves_six_cu() {
        let minor = major::resolve_minor("SEAS", "EENT", &["Standard".to_string()])
            .expect("EENT minor resolves");
        assert_eq!(minor.short_name, "EENT");

        let expanded = requirement::expand_restriction_slots(minor.requirements.clone());
        assert_eq!(
            expanded.len(),
            6,
            "EENT Standard: 5450 + 5460/5490 choice + 4 elective slots"
        );

        let fellows = major::resolve_minor("SEAS", "EENT", &["Fellows".to_string()])
            .expect("EENT Fellows resolves");
        let fellows_expanded = requirement::expand_restriction_slots(fellows.requirements.clone());
        assert_eq!(
            fellows_expanded.len(),
            6,
            "EENT Fellows: 5410 + 5430 + 4 elective slots"
        );
    }

    #[test]
    fn eent_single_half_cu_does_not_satisfy_electives() {
        let minor = major::resolve_minor("SEAS", "EENT", &["Standard".to_string()])
            .expect("EENT minor resolves");
        let cu_map = courses_data::cu_map().clone();
        let taken = vec![
            "EAS 5450".into(),
            "EAS 5460".into(),
            "MGMT 2670".into(),
        ];
        let validation =
            requirement::validate_courses_for_degree(minor.requirements.clone(), &taken, &cu_map);
        assert!(
            !validation
                .fulfilled
                .iter()
                .any(|m| m.requirement.get_category() == "EENT Electives"),
            "one 0.5 CU elective cannot satisfy a 1 CU elective slot"
        );
        let elective_fulfilled = validation
            .fulfilled
            .iter()
            .filter(|m| m.requirement.get_category() == "EENT Electives")
            .count();
        assert_eq!(
            elective_fulfilled, 0,
            "half-credit alone should not fulfill any elective slot"
        );
    }

    #[test]
    fn eent_half_cu_pairs_accumulate_toward_four_cu_electives() {
        let minor = major::resolve_minor("SEAS", "EENT", &["Standard".to_string()])
            .expect("EENT minor resolves");
        let cu_map = courses_data::cu_map().clone();
        let taken = vec![
            "EAS 5450".into(),
            "EAS 5460".into(),
            "MGMT 2670".into(),
            "MKTG 2270".into(),
            "FNCE 2500".into(),
            "NETS 1120".into(),
            "EAS 5070".into(),
        ];
        let validation =
            requirement::validate_courses_for_degree(minor.requirements.clone(), &taken, &cu_map);
        let elective_fulfilled: Vec<_> = validation
            .fulfilled
            .iter()
            .filter(|m| m.requirement.get_category() == "EENT Electives")
            .collect();
        assert_eq!(
            elective_fulfilled.len(),
            4,
            "four 1 CU elective slots should be fulfilled"
        );
        assert!(
            elective_fulfilled.iter().any(|m| {
                m.course_ids.contains(&"MGMT 2670".to_string())
                    && m.course_ids.contains(&"MKTG 2270".to_string())
            }),
            "paired 0.5 CU courses should count as 1 CU in one elective slot"
        );
        assert_eq!(
            validation.unfulfilled.len(),
            0,
            "EENT minor should be fully satisfied"
        );
    }

    #[test]
    fn math_minor_double_counts_with_cas_major() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![
                "MATH 1400".into(),
                "MATH 1410".into(),
                "MATH 1040".into(),
            ],
            degrees: vec![
                DegreeInput {
                    major: "ECON".into(),
                    school: "CAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "MATH".into(),
                    school: "CAS".into(),
                    kind: "minor".into(),
                    concentrations: vec![],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(false),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });

        assert_eq!(output.degree_results.len(), 2);
        let minor = output
            .degree_results
            .iter()
            .find(|r| r.kind == "minor")
            .expect("minor result");
        assert_eq!(minor.major, "MATH");
        assert!(
            minor
                .fulfilled_requirements
                .iter()
                .any(|m| m.course_ids.iter().any(|c| c == "MATH 1400")),
            "calculus should count on math minor when shared with CAS major plan"
        );
    }

    #[test]
    fn minor_double_counts_with_major_schedule() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![
                "MATH 1400".into(),
                "MATH 1410".into(),
                "EAS 5450".into(),
            ],
            degrees: vec![
                DegreeInput {
                    major: "CIS".into(),
                    school: "SEAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "EENT".into(),
                    school: "SEAS".into(),
                    kind: "minor".into(),
                    concentrations: vec!["Standard".into()],
                    concentration: Some("Standard".into()),
                },
            ],
            frozen: vec![],
            allow_summer: Some(false),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });

        assert_eq!(output.degree_results.len(), 2);
        let minor = output
            .degree_results
            .iter()
            .find(|r| r.kind == "minor")
            .expect("minor result");
        assert_eq!(minor.major, "EENT");
        assert!(
            minor
                .fulfilled_requirements
                .iter()
                .any(|m| m.course_ids.iter().any(|c| c == "EAS 5450")),
            "EENT core should count on minor even when shared with major plan"
        );
    }

    #[test]
    fn minor_blocks_grad_only_double_count() {
        let output = generate_schedule(ScheduleInput {
            taken: vec!["EAS 5450".into()],
            degrees: vec![
                DegreeInput {
                    major: "MS_CIS".into(),
                    school: "SEAS_MS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "EENT".into(),
                    school: "SEAS".into(),
                    kind: "minor".into(),
                    concentrations: vec!["Standard".into()],
                    concentration: Some("Standard".into()),
                },
            ],
            frozen: vec![],
            allow_summer: Some(false),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });

        let minor = output
            .degree_results
            .iter()
            .find(|r| r.kind == "minor")
            .expect("minor result");
        assert!(
            !minor
                .fulfilled_requirements
                .iter()
                .any(|m| m.course_ids.iter().any(|c| c == "EAS 5450")),
            "graduate-only overlap must not count toward the minor"
        );
    }

    #[test]
    fn minor_allows_undergrad_grad_and_minor_overlap() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![
                "MATH 1400".into(),
                "MATH 1410".into(),
                "EAS 5450".into(),
                "CIS 5190".into(),
            ],
            degrees: vec![
                DegreeInput {
                    major: "CIS".into(),
                    school: "SEAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "MS_CIS".into(),
                    school: "SEAS_MS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "EENT".into(),
                    school: "SEAS".into(),
                    kind: "minor".into(),
                    concentrations: vec!["Standard".into()],
                    concentration: Some("Standard".into()),
                },
            ],
            frozen: vec![],
            allow_summer: Some(false),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });

        let minor = output
            .degree_results
            .iter()
            .find(|r| r.kind == "minor")
            .expect("minor result");
        assert!(
            minor
                .fulfilled_requirements
                .iter()
                .any(|m| m.course_ids.iter().any(|c| c == "EAS 5450")),
            "undergrad + masters + minor may share EENT core courses"
        );
    }

    #[test]
    fn degree_results_preserve_payload_order_when_one_major_unresolved() {
        // Payload: valid, invalid, valid — results must stay aligned by index for the UI.
        let payload = ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "CIS".into(),
                    school: "SEAS".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "NOT_A_REAL_MAJOR".into(),
                    school: "SEAS".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(false),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        };
        let output = generate_schedule(payload);
        assert_eq!(output.degree_results.len(), 3);

        let majors: Vec<&str> = output
            .degree_results
            .iter()
            .map(|r| r.major.as_str())
            .collect();
        assert_eq!(
            majors,
            vec!["CIS", "NOT_A_REAL_MAJOR", "EE"],
            "degree_results must follow payload order; got {majors:?}"
        );
        assert!(output.degree_results[0].error.is_none());
        assert!(
            output.degree_results[1].error.as_deref().is_some_and(|e| e.contains("not implemented")),
            "middle entry should be the unresolved major error"
        );
        assert!(output.degree_results[2].error.is_none());

        // #region agent log
        {
            use std::io::Write;
            let log_path = "/Users/thoughtworks/Documents/2. Technology/Course Schedule Optimizer/penn-degree-planner/.cursor/debug-5cbabc.log";
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                let _ = writeln!(
                    f,
                    r#"{{"sessionId":"5cbabc","runId":"post-fix","hypothesisId":"H1","location":"test_suite.rs:degree_results_preserve_payload_order","message":"degree_results order after generate","data":{{"majors":{:?},"errors":{:?}}},"timestamp":{}}}"#,
                    majors,
                    output
                        .degree_results
                        .iter()
                        .map(|r| r.error.is_some())
                        .collect::<Vec<_>>(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0)
                );
            }
        }
        // #endregion
    }

    #[test]
    fn degree_catalog_excludes_unimplemented_majors() {
        let catalog = major::degree_catalog();
        let cas = catalog
            .iter()
            .find(|s| s.school_code == "CAS")
            .expect("CAS in catalog");
        assert!(
            !cas.majors.iter().any(|m| m.api_code == "BIOP"),
            "gen-ed-only placeholders should not appear in the UI catalog"
        );
        assert!(
            cas.majors.iter().any(|m| m.api_code == "BIOC"),
            "authored CAS Biochemistry should appear in the UI catalog"
        );
        assert!(
            cas.majors.iter().any(|m| m.api_code == "BIOL"),
            "authored CAS Biology should appear in the UI catalog"
        );
        assert!(
            cas.majors.iter().any(|m| m.api_code == "ECON"),
            "implemented CAS majors should remain selectable"
        );
        assert!(
            catalog
                .iter()
                .any(|s| s.school_code == "NURS")
                && catalog.iter().any(|s| {
                    s.school_code == "NURS"
                        && s.majors.iter().any(|m| m.api_code == "BSN")
                        && s.majors.iter().any(|m| m.api_code == "BSN_NOFL")
                        && s.majors.iter().any(|m| m.api_code == "NUTR_BSN")
                        && s.majors.iter().any(|m| m.api_code == "NUTR_BSN_NOFL")
                }),
            "implemented Nursing majors should appear in the UI catalog"
        );
        let seas_ms = catalog
            .iter()
            .find(|s| s.school_code == "SEAS_MS")
            .expect("SEAS_MS in catalog");
        assert!(
            seas_ms.majors.iter().any(|m| m.api_code == "MS_MEAM"),
            "implemented MS_MEAM should appear in the UI catalog"
        );
        assert!(
            !seas_ms.majors.iter().any(|m| m.api_code == "MS_MSE"),
            "placeholder grad programs should not appear in the UI catalog"
        );
    }

    #[test]
    fn major_is_implemented_inferred_from_requirements() {
        let biop = resolve_major("CAS", "BIOP", &[]).expect("BIOP resolves");
        assert!(
            !major::major_has_authored_requirements("CAS", &biop),
            "CAS placeholder with empty pool major CU should not count as implemented"
        );
        let bioc = resolve_major("CAS", "BIOC", &[]).expect("BIOC resolves");
        assert!(major::major_has_authored_requirements("CAS", &bioc));
        let biol = resolve_major("CAS", "BIOL", &[]).expect("BIOL resolves");
        assert!(major::major_has_authored_requirements("CAS", &biol));
        let econ = resolve_major("CAS", "ECON", &[]).expect("ECON resolves");
        assert!(major::major_has_authored_requirements("CAS", &econ));
        let meam = resolve_major("SEAS_MS", "MS_MEAM", &[]).expect("MS_MEAM resolves");
        assert!(major::major_is_implemented("SEAS_MS", "MS_MEAM"));
        assert!(major::major_has_authored_requirements("SEAS_MS", &meam));
        assert!(major::major_is_implemented("SEAS_MS", "MS_EE"));
        assert!(major::major_is_implemented("SEAS_MS", "MS_BE"));
    }

    #[test]
    fn restriction_description_omits_excluding() {
        let req = Requirement::Restriction {
            category: None,
            department: Some(vec!["PSYC".to_string()]),
            cu: None,
            level: Some(1000),
            max_level: Some(4999),
            attr: None,
            excluding: Some(vec!["PSYC 4997".to_string()]),
            number: 1,
            no_school: None,
        };
        let desc = req.create_requirement_description();
        assert!(!desc.to_lowercase().contains("excluding"));
        assert!(desc.contains("PSYC"));
    }

    #[test]
    fn dmd_major_resolves_with_thirty_seven_cu() {
        let dmd = resolve_major("SEAS", "DMD", &[]).expect("DMD");
        assert_eq!(dmd.short_name, "DMD");
        assert_eq!(dmd.name, "Digital Media Design");
        assert_eq!(dmd.requirements.len(), 35);
        assert!(dmd.concentrations.is_none());
        assert!(major::major_is_implemented("SEAS", "DMD"));
    }

    #[test]
    fn be_major_general_electives_use_course_pool() {
        use degree_planner::Requirement;

        let be = resolve_major("SEAS", "BE", &[]).expect("BE");
        assert_eq!(be.short_name, "BE");
        assert_eq!(be.requirements.len(), 33);
        let pool = be
            .requirements
            .iter()
            .find(|r| matches!(r, Requirement::CoursePool { category, .. } if category.as_deref() == Some("General Electives")))
            .expect("BE should have a General Electives CoursePool");
        if let Requirement::CoursePool {
            fixed_slots,
            flexible_slots,
            constraints,
            ..
        } = pool
        {
            assert!(fixed_slots.is_empty());
            assert_eq!(*flexible_slots, 7);
            assert_eq!(constraints.len(), 5);
            let units: i32 = constraints.iter().map(|c| c.count).sum();
            assert_eq!(units, 8, "8 coverage requirements on 7 pool courses");
        } else {
            panic!("expected CoursePool");
        }
    }

    #[test]
    fn be_pool_slots_count_only_valid_courses() {
        let be = resolve_major("SEAS", "BE", &[]).expect("BE");
        let cu_map = catalog_cu_map();
        let taken = vec!["EAS 2030".to_string()];
        let validation =
            validate_courses_for_degree(be.requirements.clone(), &taken, &cu_map);
        let pool = validation
            .pool_coverage_info
            .iter()
            .find(|p| p.category == "General Electives")
            .expect("General Electives pool");
        assert_eq!(
            pool.flexible_slots_filled, 1,
            "one taken course should fill one flex slot, got {}",
            pool.flexible_slots_filled
        );
        let cov_done = pool.constraints.iter().filter(|c| c.fulfilled).count();
        assert_eq!(
            cov_done, 2,
            "EAS 2030 should satisfy ethics + one distribution constraint, got {cov_done}"
        );
        assert!(
            pool.pool_courses.iter().all(|c| course::is_valid_course_code(c)),
            "pool_courses should not include requirement slot placeholders: {:?}",
            pool.pool_courses
        );
    }

    #[test]
    fn be_pool_schedule_flex_filled_matches_valid_courses_only() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "BE".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        let pool = output.degree_results[0]
            .pool_coverage_info
            .iter()
            .find(|p| p.category == "General Electives")
            .expect("General Electives pool");
        assert_eq!(
            pool.flexible_slots_filled as usize,
            pool.pool_courses.len(),
            "flex slots filled ({}) should match valid pool courses ({})",
            pool.flexible_slots_filled,
            pool.pool_courses.len()
        );
    }

    #[test]
    fn be_pool_schedule_uses_only_general_electives_labels() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "BE".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        for (slot, label) in &output.slot_labels {
            if slot.contains(":p") && slot.contains("29:") {
                assert!(
                    label.contains("General Electives"),
                    "BE pool flex slot {slot} should be General Electives, got {label}"
                );
            }
            if label.contains("Social Science") {
                panic!(
                    "pool constraint label on schedule: {slot} => {label}; all: {:?}",
                    output.slot_labels
                );
            }
        }
    }

    #[test]
    fn be_wh_dual_pool_schedule_avoids_constraint_slot_labels() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "BE".into(),
                    school: "SEAS".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL".into(),
                    school: "WH".into(),
                    kind: "major".to_string(),
                    concentrations: vec!["FNCE".into()],
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
        for (slot, label) in &output.slot_labels {
            if slot.contains(":p") && slot.contains("29:") {
                assert!(
                    label.contains("General Electives"),
                    "BE pool flex slot {slot} should be General Electives, got {label}"
                );
            }
            assert!(
                !label.contains("Social Science"),
                "pool constraint label on schedule: {slot} => {label}"
            );
        }
        for group in &output.overlap_schedule_groups {
            for m in &group.members {
                if m.degree_index == 0 && m.label.contains("Social Science") {
                    panic!(
                        "BE overlap should use pool category, not constraint name: {:?}",
                        group
                    );
                }
                if m.degree_index == 0 {
                    assert!(
                        m.label.contains("General Electives"),
                        "BE overlap member should be General Electives, got {}",
                        m.label
                    );
                }
            }
        }
    }

    #[test]
    fn ms_be_major_resolves_with_ten_cu() {
        let thesis = resolve_major("SEAS_MS", "MS_BE", &[]).expect("MS_BE thesis default");
        assert_eq!(thesis.short_name, "MS_BE");
        assert_eq!(thesis.name, "Bioengineering, MSE");
        assert_eq!(thesis.requirements.len(), 10);
        assert!(thesis.concentrations.is_some());
        assert!(
            thesis
                .requirements
                .iter()
                .filter(|r| matches!(
                    r,
                    Requirement::SingleCourse {
                        possibilities,
                        ..
                    } if possibilities == &["BE 9990".to_string()]
                ))
                .count() == 2,
            "thesis track should require 2 CU of BE 9990"
        );

        let non_thesis =
            resolve_major("SEAS_MS", "MS_BE", &["Non-thesis".into()]).expect("MS_BE non-thesis");
        assert_eq!(non_thesis.requirements.len(), 10);
        assert!(
            !non_thesis.requirements.iter().any(|r| matches!(
                r,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.contains(&"BE 9990".to_string())
            )),
            "non-thesis track should not require BE 9990"
        );
        assert_eq!(
            major::concentrations_for("SEAS_MS", "MS_BE"),
            vec!["Thesis", "Non-thesis"]
        );
    }

    #[test]
    fn ms_be_taken_thesis_fills_both_slots() {
        let major = resolve_major("SEAS_MS", "MS_BE", &[]).expect("MS_BE");
        let cu_map = courses_data::cu_map();
        assert!(
            (cu_map.get("BE 9990").copied().unwrap_or(0.0) - 2.0).abs() < 1e-6,
            "BE 9990 catalog CU should be 2.0"
        );
        let result = validate_courses_for_degree(
            major.requirements.clone(),
            &vec!["BE 9990".to_string()],
            cu_map,
        );
        let thesis_fulfilled = result
            .fulfilled
            .iter()
            .filter(|m| {
                matches!(
                    &m.requirement,
                    Requirement::SingleCourse {
                        possibilities,
                        ..
                    } if possibilities == &["BE 9990".to_string()]
                )
            })
            .count();
        let thesis_open = result
            .unfulfilled
            .iter()
            .filter(|m| {
                matches!(
                    &m.requirement,
                    Requirement::SingleCourse {
                        possibilities,
                        ..
                    } if possibilities == &["BE 9990".to_string()]
                )
            })
            .count();
        assert_eq!(thesis_fulfilled, 2, "one taken BE 9990 should fill both 1 CU thesis slots");
        assert_eq!(thesis_open, 0, "no open thesis slots after taking BE 9990");
    }

    #[test]
    fn ms_be_schedule_uses_two_thesis_placeholders() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "MS_BE".into(),
                school: "SEAS_MS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        let suggested = &output.degree_results[0].suggested_for_unfulfilled;
        let thesis_suggestions: Vec<_> = suggested
            .iter()
            .filter(|m| {
                matches!(
                    &m.requirement,
                    Requirement::SingleCourse {
                        category: Some(cat),
                        possibilities,
                        ..
                    } if cat == "Master's Thesis" && possibilities == &["BE 9990".to_string()]
                )
            })
            .collect();
        assert_eq!(thesis_suggestions.len(), 2, "two open thesis units");
        for mapped in &thesis_suggestions {
            assert_eq!(mapped.course_ids.len(), 1);
            let id = &mapped.course_ids[0];
            assert!(
                requirement::is_schedulable_requirement_slot_id(id),
                "duplicate sole-course thesis should schedule as req: placeholder, got {id}"
            );
            assert_ne!(id, "BE 9990", "must not emit a single concrete BE 9990 for both units");
        }
        assert_ne!(
            thesis_suggestions[0].course_ids[0], thesis_suggestions[1].course_ids[0],
            "placeholders must be distinct so units can sit in different semesters"
        );

        let schedule_ids: Vec<String> = output
            .schedule
            .iter()
            .flat_map(|p| {
                p.courses
                    .iter()
                    .cloned()
                    .chain(p.requirement_slots.iter().cloned())
            })
            .collect();
        let thesis_on_grid: Vec<_> = thesis_suggestions
            .iter()
            .map(|m| m.course_ids[0].as_str())
            .filter(|id| schedule_ids.iter().any(|s| s == id))
            .collect();
        assert_eq!(
            thesis_on_grid.len(),
            2,
            "both thesis placeholders should appear on the schedule grid"
        );

        let with_taken = generate_schedule(ScheduleInput {
            taken: vec!["BE 9990".into()],
            degrees: vec![DegreeInput {
                major: "MS_BE".into(),
                school: "SEAS_MS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(with_taken.error.is_none(), "{:?}", with_taken.error);
        let open_thesis = with_taken.degree_results[0]
            .suggested_for_unfulfilled
            .iter()
            .filter(|m| {
                matches!(
                    &m.requirement,
                    Requirement::SingleCourse {
                        possibilities,
                        ..
                    } if possibilities == &["BE 9990".to_string()]
                )
            })
            .count();
        assert_eq!(open_thesis, 0, "taken BE 9990 should clear thesis suggestions");
    }

    #[test]
    fn ms_be_non_thesis_schedule_has_no_be_9990() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "MS_BE".into(),
                school: "SEAS_MS".into(),
                kind: "major".to_string(),
                concentrations: vec!["Non-thesis".into()],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        assert!(
            !output.degree_results[0].suggested_for_unfulfilled.iter().any(|m| {
                matches!(
                    &m.requirement,
                    Requirement::SingleCourse { possibilities, .. }
                        if possibilities.contains(&"BE 9990".to_string())
                )
            }),
            "non-thesis track should not suggest BE 9990"
        );
    }

    #[test]
    fn single_sole_course_still_suggests_concrete_code() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "CIS".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        let cis_1200 = output.degree_results[0]
            .suggested_for_unfulfilled
            .iter()
            .find(|m| {
                matches!(
                    &m.requirement,
                    Requirement::SingleCourse {
                        possibilities,
                        ..
                    } if possibilities == &["CIS 1200".to_string()]
                )
            });
        let Some(mapped) = cis_1200 else {
            // CIS tree may nest CIS 1200; accept concrete appearance on the schedule.
            assert!(
                output
                    .schedule
                    .iter()
                    .any(|p| p.courses.iter().any(|c| c == "CIS 1200")),
                "sole-possibility SingleCourse should still schedule as concrete CIS 1200"
            );
            return;
        };
        assert_eq!(mapped.course_ids, vec!["CIS 1200".to_string()]);
    }

    #[test]
    fn ms_meam_major_resolves_with_ten_cu_and_concentrations() {
        let design = resolve_major("SEAS_MS", "MS_MEAM", &[]).expect("MS_MEAM design default");
        assert_eq!(design.short_name, "MS_MEAM");
        assert_eq!(
            design.name,
            "Mechanical Engineering and Applied Mechanics, MSE"
        );
        assert_eq!(design.requirements.len(), 10);
        assert!(design.concentrations.is_some());
        assert_eq!(design.concentrations.as_ref().unwrap().len(), 5);
        assert!(
            design.requirements.iter().any(|r| matches!(
                r,
                Requirement::SingleCourse {
                    category: Some(cat),
                    possibilities,
                    ..
                } if cat == "Concentration" && possibilities == &["MEAM 5140".to_string()]
            )),
            "Design and Manufacturing should require MEAM 5140"
        );
        assert_eq!(
            design
                .requirements
                .iter()
                .filter(|r| r.get_category() == "Concentration")
                .count(),
            3,
            "concentration should contribute 1 required + 2 core elective slots"
        );

        let heat = resolve_major(
            "SEAS_MS",
            "MS_MEAM",
            &["Heat Transfer, Fluid Mechanics, and Energy".into()],
        )
        .expect("MS_MEAM heat");
        assert!(
            heat.requirements.iter().any(|r| matches!(
                r,
                Requirement::SingleCourse {
                    category: Some(cat),
                    possibilities,
                    ..
                } if cat == "Concentration"
                    && possibilities.contains(&"MEAM 5360".to_string())
                    && possibilities.contains(&"MEAM 5700".to_string())
            )),
            "Heat concentration should allow MEAM 5360 or MEAM 5700 as required"
        );

        assert_eq!(
            major::concentrations_for("SEAS_MS", "MS_MEAM"),
            vec![
                "Design and Manufacturing",
                "Heat Transfer, Fluid Mechanics, and Energy",
                "Mechanics of Materials",
                "Mechatronic and Robotic Systems",
                "Micro/Nano Systems",
            ]
        );
    }

    #[test]
    fn chem_major_resolves() {
        let chem = resolve_major("CAS", "CHEM", &[]).expect("CHEM");
        assert_eq!(chem.short_name, "CHEM");
        assert_eq!(chem.name, "Chemistry");
        assert!(chem.concentrations.is_none());
    }

    #[test]
    fn phys_major_resolves_with_concentrations() {
        use degree_planner::penn_data::college_data::phys_concentration_names;

        assert_eq!(phys_concentration_names().len(), 6);
        let astro = resolve_major("CAS", "PHYS", &["Astrophysics".into()]).expect("PHYS");
        assert_eq!(astro.short_name, "PHYS");
        assert!(astro.concentrations.is_some());
        let bio = resolve_major("CAS", "PHYS", &["Biological Science".into()]).expect("PHYS bio");
        assert_eq!(bio.short_name, "PHYS");
        let chem = resolve_major("CAS", "PHYS", &["Chemical Principles".into()]).expect("PHYS chem");
        assert_eq!(chem.short_name, "PHYS");
        let comp = resolve_major("CAS", "PHYS", &["Computer Techniques".into()]).expect("PHYS comp");
        assert_eq!(comp.short_name, "PHYS");
        let theory = resolve_major(
            "CAS",
            "PHYS",
            &["Physical Theory and Experimental Technique".into()],
        )
        .expect("PHYS theory");
        assert_eq!(theory.short_name, "PHYS");
    }

    #[test]
    fn econ_gen_ed_marks_society_sector_completed_by_major() {
        use degree_planner::penn_data::college_data::{build_cas_gen_ed_info, cas_auto_completed_sectors_for, create_econ_major, SECTOR_SOCIETY};

        let major = create_econ_major();
        let cu_map = HashMap::from([("WRIT 0100".to_string(), 1.0)]);
        let taken = vec!["WRIT 0100".to_string()];
        let validation =
            validate_courses_for_degree(major.requirements, &taken, &cu_map);
        let pool = validation
            .pool_coverage_info
            .into_iter()
            .find(|p| p.category == "General Education")
            .expect("gen ed pool");
        let info = build_cas_gen_ed_info(&pool, &cas_auto_completed_sectors_for("ECON", None));

        assert_eq!(info.foundational_approaches.len(), 5);
        assert_eq!(info.sectors.len(), 7);
        let society = info
            .sectors
            .iter()
            .find(|s| s.attr == SECTOR_SOCIETY)
            .expect("society sector");
        assert!(society.fulfilled);
        assert!(society.fulfilled_by_major);
    }

    #[test]
    fn neur_gen_ed_marks_living_and_physical_world_completed_by_major() {
        use degree_planner::penn_data::college_data::{
            build_cas_gen_ed_info, cas_auto_completed_sectors_for, SECTOR_LIVING_WORLD,
            SECTOR_PHYSICAL_WORLD,
        };

        let major = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let cu_map = HashMap::from([("WRIT 0100".to_string(), 1.0)]);
        let taken = vec!["WRIT 0100".to_string()];
        let validation =
            validate_courses_for_degree(major.requirements, &taken, &cu_map);
        let pool = validation
            .pool_coverage_info
            .into_iter()
            .find(|p| p.category == "General Education")
            .expect("gen ed pool");
        let info = build_cas_gen_ed_info(&pool, &cas_auto_completed_sectors_for("NEUR", None));

        for attr in [SECTOR_LIVING_WORLD, SECTOR_PHYSICAL_WORLD] {
            let sector = info
                .sectors
                .iter()
                .find(|s| s.attr == attr)
                .unwrap_or_else(|| panic!("{attr} sector"));
            assert!(sector.fulfilled);
            assert!(sector.fulfilled_by_major);
        }
    }

    #[test]
    fn anth_medical_concentration_completes_hum_soc_sci_sector() {
        use degree_planner::penn_data::college_data::{cas_auto_completed_sectors_for, SECTOR_HUM_SOC_SCI};

        let sectors = cas_auto_completed_sectors_for(
            "ANTH",
            Some("Medical Anthropology & Global Health"),
        );
        assert_eq!(sectors, vec![SECTOR_HUM_SOC_SCI.to_string()]);
    }

    #[test]
    fn implemented_majors_resolve_with_requirements() {
        let cases = [
            ("CAS", "NEUR", vec![] as Vec<&str>),
            ("CAS", "ECON", vec![]),
            ("SEAS", "CIS", vec![]),
            ("WH", "WH_NOFL", vec!["FNCE"]),
            ("SEAS_MS", "MS_ROBO", vec![]),
            ("NURS", "BSN", vec![]),
            ("NURS", "NUTR_BSN", vec![]),
        ];
        for (school, major_code, concs) in cases {
            let conc_vec: Vec<String> = concs.into_iter().map(str::to_string).collect();
            let m = resolve_major(school, major_code, &conc_vec)
                .unwrap_or_else(|| panic!("{school}/{major_code} should resolve"));
            assert!(
                !m.requirements.is_empty(),
                "{school}/{major_code} needs requirements"
            );
        }
    }

    #[test]
    fn cas_placeholder_majors_are_valid_stubs() {
        let biop = resolve_major("CAS", "BIOP", &[]).expect("BIOP");
        assert_eq!(biop.short_name, "BIOP");
        assert!(biop.concentrations.is_none());
    }

    #[test]
    fn neur_includes_brain_behavior_and_abbe_pool() {
        let major = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let pool = major
            .requirements
            .iter()
            .find_map(|r| match r {
                Requirement::CoursePool { .. } => Some(r),
                _ => None,
            })
            .expect("gen ed pool");
        let Requirement::CoursePool { fixed_slots, .. } = pool else {
            panic!("expected pool");
        };
        assert!(
            fixed_slots
                .iter()
                .any(|r| r.get_category() == "Introduction to Brain & Behavior")
        );
        let abbe = fixed_slots
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    Requirement::Restriction {
                        attr: Some(attrs),
                        ..
                    } if attrs == &vec!["ABBE".to_string()]
                )
            })
            .count();
        assert_eq!(abbe, 3);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Course identity — only real Penn courses belong on a plan
// ═══════════════════════════════════════════════════════════════════════════════

mod course_identity {
    use super::*;

    #[test]
    fn valid_codes_match_dept_number_pattern() {
        assert!(course::is_valid_course_code("CIS 1200"));
        assert!(course::is_valid_course_code("MATH 1400"));
        assert!(!course::is_valid_course_code("CIS1200"));
        assert!(!course::is_valid_course_code("not a course"));
        assert!(!course::is_valid_course_code(""));
    }

    #[test]
    fn graduate_level_uses_course_number_threshold() {
        assert!(!course::is_graduate_level("CIS 1200"));
        assert!(course::is_graduate_level("CIS 5190"));
    }

    proptest! {
        #[test]
        fn invalid_strings_are_rejected(s in "\\PC*") {
            prop_assume!(!s.contains(' '));
            prop_assume!(!s.chars().any(|c| c.is_ascii_digit()));
            prop_assert!(!course::is_valid_course_code(&s));
        }

        #[test]
        fn synthetic_valid_codes_round_trip(dept in "[A-Z]{2,4}", num in 1000u32..9999) {
            let code = format!("{dept} {num}");
            prop_assume!(course::is_valid_course_code(&code));
            prop_assert_eq!(course::course_number(&code), Some(num as i32));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Requirement fulfillment — does the degree audit make sense?
// ═══════════════════════════════════════════════════════════════════════════════

mod requirement_fulfillment {
    use super::*;

    #[test]
    fn restriction_number_expands_to_individual_slots() {
        let cu_map = sample_cu_map();
        let expanded = expand_restriction_slots(vec![Requirement::Restriction {
            category: Some("Test elective".into()),
            department: Some(vec!["TEST".into()]),
            cu: None,
            level: None,
            max_level: None,
            attr: None,
            excluding: None,
            number: 4,
            no_school: None,
        }]);
        let explicit = vec![one_cu_restriction(); 4];
        let taken = vec![
            "TEST 1000".into(),
            "TEST 1001".into(),
            "TEST 1002".into(),
            "TEST 1003".into(),
        ];
        let a = validate_courses_for_degree(expanded, &taken, &cu_map);
        let b = validate_courses_for_degree(explicit, &taken, &cu_map);
        assert_eq!(a.fulfilled.len(), b.fulfilled.len());
        assert_eq!(a.unfulfilled.len(), b.unfulfilled.len());
    }

    #[test]
    fn half_cu_course_does_not_fill_one_cu_slot() {
        let attributes = attributes_data::create_attributes();
        let mut cu_map = HashMap::new();
        cu_map.insert("TEST 1000".into(), 0.5);
        let taken = vec!["TEST 1000".into()];
        assert!(
            one_cu_restriction()
                .fulfills_requirement(&taken, &attributes, &cu_map)
                .is_none()
        );
    }

    #[test]
    fn attribute_exclusion_blocks_even_when_attr_matches() {
        let attributes = attributes_data::create_attributes();
        assert!(!course_matches_restriction(
            "AFRC 0030",
            &None,
            &None,
            &None,
            &Some(vec!["AIRE".into()]),
            &Some(vec!["AUFS".into()]),
            &None,
            &attributes,
        ));
        assert!(course_matches_restriction(
            "BEPP 2010",
            &None,
            &None,
            &None,
            &Some(vec!["AIRE".into()]),
            &Some(vec!["AUFS".into()]),
            &None,
            &attributes,
        ));
    }

    #[test]
    fn nurs_exploration_excludes_nurs_courses() {
        let attributes = attributes_data::create_attributes();
        let no_school = Some("NURS".to_string());
        assert!(!course_matches_restriction(
            "NURS 1030",
            &None,
            &None,
            &None,
            &None,
            &None,
            &no_school,
            &attributes,
        ));
        assert!(course_matches_restriction(
            "ECON 0100",
            &None,
            &None,
            &None,
            &None,
            &None,
            &no_school,
            &attributes,
        ));
    }

    #[test]
    fn taken_courses_partition_into_fulfilled_and_open() {
        let cu_map = catalog_cu_map();
        let major = resolve_major("SEAS", "CIS", &[]).expect("CIS");
        let taken = vec!["CIS 1100".into(), "CIS 1200".into()];
        let v = validate_courses_for_degree(major.requirements.clone(), &taken, &cu_map);
        assert!(!v.fulfilled.is_empty());
        let fulfilled_courses: HashSet<_> = v
            .fulfilled
            .iter()
            .flat_map(|m| m.course_ids.iter())
            .collect();
        for c in &taken {
            assert!(fulfilled_courses.contains(c));
        }
    }

    #[test]
    fn requirement_slot_ids_are_distinguishable_from_courses() {
        assert!(is_requirement_slot_id("req:0"));
        assert!(!is_requirement_slot_id("CIS 1200"));
        assert!(is_schedulable_requirement_slot_id("req:1:f0"));
        assert!(!is_schedulable_requirement_slot_id("req:1:c0"));
        assert!(is_pool_constraint_slot_id("req:1:c0"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Pool constraints & concentrations — flexible buckets behave correctly
// ═══════════════════════════════════════════════════════════════════════════════

mod pools_and_concentrations {
    use super::*;

    #[test]
    fn wh_fl_mt_pool_blocks_cc_ssh_double_count_in_same_group() {
        use degree_planner::penn_data::wharton_data::create_wh_fl_mt_major;

        let major = create_wh_fl_mt_major(vec!["FNCE".into()]);
        let pool_req = major
            .requirements
            .iter()
            .find(|r| matches!(r, Requirement::CoursePool { .. }))
            .expect("FL M&T LAS pool");
        let Requirement::CoursePool { constraints, .. } = pool_req else {
            panic!("expected CoursePool");
        };

        let mut attributes = attributes_data::create_attributes();
        for attr in ["WUCN", "WUHM"] {
            attributes
                .entry(attr.to_string())
                .or_default()
                .push("ANTH 0001".into());
        }
        let cu_map = HashMap::from([("ANTH 0001".into(), 1.0)]);
        let pool = vec!["ANTH 0001".into()];

        let evaluations = evaluate_pool_constraints(&pool, constraints, &attributes, &cu_map);
        let mt_las_fulfilled = evaluations
            .iter()
            .filter(|e| e.consumption_group == "wh:mt_las" && e.fulfilled)
            .count();
        assert_eq!(
            mt_las_fulfilled, 1,
            "CC and SSH share wh:mt_las — one course covers at most one slot"
        );
    }

    #[test]
    fn wh_nofl_mt_has_no_course_pool() {
        use degree_planner::penn_data::wharton_data::create_wh_nofl_mt_major;

        let major = create_wh_nofl_mt_major(vec!["STAT".into()]);
        assert!(
            !major
                .requirements
                .iter()
                .any(|r| matches!(r, Requirement::CoursePool { .. })),
            "NOFL M&T uses standalone LAS requirements"
        );
    }

    #[test]
    fn wh_concentration_names_are_short_codes_and_exclude_oidd() {
        use degree_planner::penn_data::wharton_data::{
            concentration_names, create_wh_concentrations, resolve_wh_concentration_key,
        };

        let catalog = create_wh_concentrations();
        assert!(!catalog.contains_key("OIDD"));
        assert!(catalog.contains_key("MAOM"));
        assert_eq!(catalog.len(), 13);

        let names = concentration_names();
        assert!(names.contains(&"FNCE".to_string()));
        assert!(names.contains(&"STAT".to_string()));
        assert!(names.contains(&"MAOM".to_string()));
        assert!(names.contains(&"HCMG".to_string()));
        assert!(names.contains(&"ODDP".to_string()));
        assert!(names.contains(&"ODGN".to_string()));
        assert!(names.contains(&"ODIS".to_string()));
        assert!(names.contains(&"ODOM".to_string()));
        assert!(!names.iter().any(|n| n == "OIDD"));

        assert_eq!(
            resolve_wh_concentration_key("Marketing & Operations Management"),
            Some("MAOM".to_string())
        );
        assert_eq!(
            resolve_wh_concentration_key("Health Care Management"),
            Some("HCMG".to_string())
        );
        assert_eq!(
            resolve_wh_concentration_key("OIDD: Decision Processes"),
            Some("ODDP".to_string())
        );
        assert_eq!(
            resolve_wh_concentration_key("OIDD: General"),
            Some("ODGN".to_string())
        );
        assert_eq!(
            resolve_wh_concentration_key("OIDD: Information Systems"),
            Some("ODIS".to_string())
        );
        assert_eq!(
            resolve_wh_concentration_key("OIDD: Operations Management"),
            Some("ODOM".to_string())
        );
        assert_eq!(resolve_wh_concentration_key("OIDD"), None);
    }

    #[test]
    fn wh_oidd_information_systems_concentration_requires_four_electives() {
        use degree_planner::Requirement;
        use degree_planner::penn_data::wharton_data::create_wh_concentrations;

        let catalog = create_wh_concentrations();
        let is = catalog.get("ODIS").expect("ODIS concentration");
        assert_eq!(is.len(), 4);
        for slot in is {
            assert!(matches!(
                slot,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.len() == 7
            ));
        }
    }

    #[test]
    fn wh_oidd_operations_management_concentration_requires_core_and_three_electives() {
        use degree_planner::Requirement;
        use degree_planner::penn_data::wharton_data::create_wh_concentrations;

        let catalog = create_wh_concentrations();
        let om = catalog.get("ODOM").expect("ODOM concentration");
        assert_eq!(om.len(), 4);
        assert!(matches!(
            &om[0],
            Requirement::SingleCourse { possibilities, .. }
                if possibilities == &vec!["OIDD 2200".to_string(), "OIDD 2210".to_string()]
        ));
        for slot in &om[1..] {
            assert!(matches!(
                slot,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.len() == 6
            ));
        }
    }

    #[test]
    fn wh_oidd_general_concentration_requires_four_wuod_courses() {
        use degree_planner::Requirement;
        use degree_planner::penn_data::wharton_data::create_wh_concentrations;

        let catalog = create_wh_concentrations();
        let general = catalog.get("ODGN").expect("ODGN concentration");
        assert_eq!(general.len(), 1);
        assert!(matches!(
            &general[0],
            Requirement::Restriction {
                number: 4,
                attr: Some(attrs),
                ..
            } if attrs == &vec!["WUOD".to_string()]
        ));
    }

    #[test]
    fn wh_oidd_decision_processes_concentration_requires_core_and_two_electives() {
        use degree_planner::Requirement;
        use degree_planner::penn_data::wharton_data::create_wh_concentrations;

        let catalog = create_wh_concentrations();
        let dp = catalog.get("ODDP").expect("ODDP concentration");
        assert_eq!(dp.len(), 4);
        assert!(matches!(
            &dp[0],
            Requirement::SingleCourse { possibilities, .. }
                if possibilities == &vec!["OIDD 2900".to_string()]
        ));
        assert!(matches!(
            &dp[1],
            Requirement::SingleCourse { possibilities, .. }
                if possibilities == &vec!["OIDD 2910".to_string()]
        ));
        for slot in &dp[2..] {
            assert!(matches!(
                slot,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.len() == 16
            ));
        }
    }

    #[test]
    fn wh_hcmg_concentration_requires_1010_and_three_electives() {
        use degree_planner::Requirement;
        use degree_planner::penn_data::wharton_data::create_wh_concentrations;

        let catalog = create_wh_concentrations();
        let hc = catalog.get("HCMG").expect("HCMG concentration");
        assert_eq!(hc.len(), 4);
        assert!(matches!(
            &hc[0],
            Requirement::SingleCourse { possibilities, .. }
                if possibilities == &vec!["HCMG 1010".to_string()]
        ));
        for slot in &hc[1..] {
            assert!(matches!(
                slot,
                Requirement::Restriction {
                    department,
                    level: Some(2000),
                    max_level: Some(4000),
                    excluding,
                    ..
                } if department.as_ref().is_some_and(|d| d == &vec!["HCMG".to_string()])
                    && excluding.as_ref().is_some_and(|e| e.contains(&"HCMG 1010".to_string()))
            ));
        }
    }

    #[test]
    fn concentration_tracker_reflects_wh_mt_progress() {
        let major = resolve_major("WH", "WH_NOFL_MT", &["FNCE".into()]).expect("WH_NOFL_MT");
        let cu_map = catalog_cu_map();
        let taken = vec![
            "FNCE 2310".into(),
            "FNCE 2030".into(),
            "FNCE 2050".into(),
            "FNCE 2070".into(),
        ];
        let validation = validate_courses_for_degree(major.requirements.clone(), &taken, &cu_map);
        let info = extract_concentration_info(
            &major.requirements,
            &major.concentrations,
            &["FNCE".into()],
            &taken,
            &cu_map,
            Some(&validation),
        );
        assert!(!info.is_empty());
        let fnce = info
            .iter()
            .find(|c| c.name == "FNCE")
            .expect("FNCE concentration tracker");
        assert!(fnce.requirements_fulfilled > 0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Cross-degree sharing — double-counting stays within Penn rules
// ═══════════════════════════════════════════════════════════════════════════════

mod cross_degree_sharing {
    use super::*;

    #[test]
    fn business_breadth_slot_label_matches_scoped_id() {
        use degree_planner::penn_data::wharton_data;
        let major = wharton_data::create_wh_nofl_major(vec!["FNCE".into()]);
        let validation = validate_courses_for_degree(
            major.requirements.clone(),
            &vec![],
            &catalog_cu_map(),
        );
        let bb = validation
            .unfulfilled
            .iter()
            .find(|m| {
                m.requirement
                    .get_category()
                    .to_lowercase()
                    .contains("business breadth")
            })
            .expect("WH should have unfulfilled business breadth");
        let instance = bb.instance_id.as_deref().expect("instance id");
        let slot_id = requirement::business_breadth_slot_id(Some(instance));
        let label = bb.requirement.slot_label_for_id(&slot_id);
        assert!(
            bb.requirement.matches_slot_id(&slot_id),
            "matches_slot_id failed for {slot_id} category={}",
            bb.requirement.get_category()
        );
        assert_ne!(label, "Open requirement", "slot_id={slot_id} instance={instance}");
        assert!(
            label.contains("WH Business Breadth"),
            "got {label} for {slot_id}"
        );
    }

    #[test]
    fn wh_business_breadth_schedule_slots_are_labeled() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "WH_NOFL".into(),
                school: "WH".into(),
                kind: "major".to_string(),
                concentrations: vec!["FNCE".into()],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        let bb_slots: Vec<_> = output
            .slot_labels
            .iter()
            .filter(|(slot, _)| slot.contains("BB:Business_Breadth"))
            .collect();
        assert!(
            !bb_slots.is_empty(),
            "expected business breadth slots on schedule; slots: {:?}",
            output.slot_labels
        );
        for (slot, label) in bb_slots {
            assert_ne!(
                label.as_str(),
                "Open requirement",
                "BB slot {slot} should not be open requirement"
            );
            assert_eq!(
                label.as_str(),
                "1 WH Business Breadth",
                "BB slot {slot} got unexpected label"
            );
        }
    }

    #[test]
    fn course_cannot_count_toward_three_degrees() {
        let schools = vec!["SEAS".into(), "WH".into(), "SEAS_MS".into()];
        let majors = vec!["CIS".into(), "WH_FL".into(), "MS_ROBO".into()];
        let mut state = CrossDegreeState::new(schools, majors);
        let cu = sample_cu_map();
        state.register_claim("CIS 1200", 0, &cu);
        state.register_claim("CIS 1200", 1, &cu);
        assert!(matches!(
            state.can_claim("CIS 1200", 2, &cu),
            Err(CrossDegreeViolationKind::TooManyDegrees)
        ));
    }

    #[test]
    fn graduate_degrees_cannot_share_courses() {
        let schools = vec!["SEAS_MS".into(), "SEAS_MS".into()];
        let majors = vec!["MS_ROBO".into(), "MS_EE".into()];
        let mut state = CrossDegreeState::new(schools, majors);
        let cu = sample_cu_map();
        state.register_claim("CIS 5190", 0, &cu);
        assert!(matches!(
            state.can_claim("CIS 5190", 1, &cu),
            Err(CrossDegreeViolationKind::GradGradOverlap)
        ));
    }

    #[test]
    fn undergrad_grad_shared_cu_capped_at_three() {
        let schools = vec!["SEAS".into(), "SEAS_MS".into()];
        let majors = vec!["CIS".into(), "MS_ROBO".into()];
        let mut state = CrossDegreeState::new(schools, majors);
        let cu = sample_cu_map();
        for c in ["CIS 5190", "CIS 5200", "CIS 5210", "MATH 1400"] {
            state.register_claim(c, 0, &cu);
            state.register_claim(c, 1, &cu);
        }
        enforce_claim_rules(&mut state, &cu);
        let schools = vec!["SEAS".into(), "SEAS_MS".into()];
        let shared: f64 = state
            .claims
            .iter()
            .filter(|(course, idx)| crosses_undergrad_grad(course, idx, &schools))
            .map(|(course, _)| cu.get(course.as_str()).copied().unwrap_or(1.0))
            .sum();
        assert!(shared <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS);
    }

    #[test]
    fn grad_school_code_is_recognized() {
        assert!(is_graduate_degree("SEAS_MS"));
        assert!(!is_graduate_degree("SEAS"));
        assert!(!is_graduate_degree("CAS"));
    }

    #[test]
    fn overlap_plan_applies_to_undergrad_plus_grad() {
        use degree_planner::overlap_planner::is_overlap_schedule_group_id;
        use degree_planner::scheduler::{dual_undergrad_only, generate_schedule, DegreeInput, ScheduleInput};

        let schools = vec!["SEAS".into(), "SEAS_MS".into()];
        assert!(!dual_undergrad_only(&schools));
        assert!(!cross_degree_optimizer_applicable(&schools));
        assert!(overlap_plan_applicable(&schools));

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "MS_EE".into(),
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
        assert!(output.error.is_none());
        assert!(
            output.overlap_plan.is_some(),
            "EE + MS_EE should still discover grad↔undergrad overlaps"
        );
        assert!(output.cross_degree_summary.is_some());
    }

    #[test]
    fn undergrad_grad_cap_is_global_across_all_undergrad_degrees() {
        let schools = vec!["SEAS".into(), "WH".into(), "SEAS_MS".into()];
        let majors = vec!["EE".into(), "WH_NOFL_MT".into(), "MS_ROBO".into()];
        let mut state = CrossDegreeState::new(schools.clone(), majors);
        let cu = sample_cu_map();
        // EE↔MS and WH↔MS each add 1 CU — total 4 CU exceeds global 3 CU cap
        for c in ["CIS 5190", "BEPP 2500", "FNCE 1010", "MGMT 2370"] {
            if c.starts_with("CIS") {
                state.register_claim(c, 0, &cu);
                state.register_claim(c, 2, &cu);
            } else {
                state.register_claim(c, 1, &cu);
                state.register_claim(c, 2, &cu);
            }
        }
        enforce_claim_rules(&mut state, &cu);
        let shared: f64 = state
            .claims
            .iter()
            .filter(|(course, idx)| crosses_undergrad_grad(course, idx, &schools))
            .map(|(course, _)| cu.get(course.as_str()).copied().unwrap_or(1.0))
            .sum();
        assert!(
            shared <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS,
            "total undergrad↔masters CU should be capped at 3, got {shared}"
        );
    }

    #[test]
    fn schedule_undergrad_grad_overlap_respects_three_cu_cap() {
        use degree_planner::cross_degree::is_graduate_degree;
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

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

        let schools: Vec<String> = output
            .degree_results
            .iter()
            .map(|r| r.school.clone())
            .collect();
        let cu_map = catalog_cu_map();

        let summary = output
            .cross_degree_summary
            .as_ref()
            .expect("cross degree summary");
        let shared_cu: f64 = summary
            .course_allocations
            .iter()
            .filter(|(course, allocs)| {
                let idx: HashSet<_> = allocs.iter().map(|a| a.degree_index).collect();
                crosses_undergrad_grad(course, &idx, &schools)
            })
            .map(|(course, _)| cu_map.get(course.as_str()).copied().unwrap_or(1.0))
            .sum();
        assert!(
            shared_cu <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS,
            "scheduled undergrad↔masters shared CU {shared_cu} exceeds {UNDERGRAD_GRAD_CU_LIMIT}"
        );
        assert!(
            summary.violations.is_empty(),
            "unexpected cross-degree violations: {:?}",
            summary.violations
        );

        let ug_ms_overlap_groups = output
            .overlap_schedule_groups
            .iter()
            .filter(|g| {
                let has_ms = g.members.iter().any(|m| is_graduate_degree(&m.school));
                let has_ug = g.members.iter().any(|m| !is_graduate_degree(&m.school));
                has_ms && has_ug
            })
            .count();
        let ug_ms_pairs = output
            .overlap_plan
            .as_ref()
            .map(|plan| {
                plan.pairs
                    .iter()
                    .filter(|pair| {
                        let schools: HashSet<_> =
                            pair.slots.iter().map(|s| s.school.as_str()).collect();
                        schools.contains("SEAS_MS")
                            && schools.iter().any(|s| *s != "SEAS_MS")
                    })
                    .count()
            })
            .unwrap_or(0);
        assert!(
            ug_ms_pairs <= UNDERGRAD_GRAD_CU_LIMIT as usize,
            "too many undergrad↔masters overlap pairs: {ug_ms_pairs}"
        );
        assert!(
            ug_ms_overlap_groups <= UNDERGRAD_GRAD_CU_LIMIT as usize,
            "too many undergrad↔masters overlap blocks on schedule: {ug_ms_overlap_groups} (pairs: {ug_ms_pairs})"
        );
    }

    #[test]
    fn schedule_undergrad_grad_cap_with_taken_overlapping_courses() {
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![
                "CIS 5190".into(),
                "CIS 5200".into(),
                "CIS 5210".into(),
                "ESE 3010".into(),
            ],
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
        });
        assert!(output.error.is_none(), "{:?}", output.error);

        let schools: Vec<String> = output
            .degree_results
            .iter()
            .map(|r| r.school.clone())
            .collect();
        let cu_map = catalog_cu_map();
        let summary = output.cross_degree_summary.as_ref().unwrap();
        let shared_cu: f64 = summary
            .course_allocations
            .iter()
            .filter(|(course, allocs)| {
                let idx: HashSet<_> = allocs.iter().map(|a| a.degree_index).collect();
                crosses_undergrad_grad(course, &idx, &schools)
            })
            .map(|(course, _)| cu_map.get(course.as_str()).copied().unwrap_or(1.0))
            .sum();
        assert!(
            shared_cu <= UNDERGRAD_GRAD_CU_LIMIT + CU_EPS,
            "taken courses shared undergrad↔masters CU {shared_cu} exceeds cap"
        );
        assert!(
            summary.violations.is_empty(),
            "violations: {:?}",
            summary.violations
        );

        let ug_ms_in_results: Vec<String> = output.degree_results[0]
            .fulfilled_requirements
            .iter()
            .flat_map(|m| m.course_ids.iter().cloned())
            .filter(|c| {
                course::is_valid_course_code(c)
                    && output.degree_results[1]
                        .fulfilled_requirements
                        .iter()
                        .any(|m| m.course_ids.iter().any(|id| id == c))
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        assert!(
            ug_ms_in_results.len() <= UNDERGRAD_GRAD_CU_LIMIT as usize,
            "MS degree results still show {} shared courses: {:?}",
            ug_ms_in_results.len(),
            ug_ms_in_results
        );
    }

    #[test]
    fn ee_wh_nofl_mt_with_ms_robo_still_surfaces_undergrad_overlaps() {
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

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

        assert!(output.error.is_none());
        assert!(
            !cross_degree_optimizer_applicable(&["SEAS".into(), "WH".into(), "SEAS_MS".into()]),
            "grad mix disables full optimizer"
        );
        assert!(overlap_plan_applicable(&[
            "SEAS".into(),
            "WH".into(),
            "SEAS_MS".into()
        ]));

        let plan = output.overlap_plan.as_ref().expect("overlap plan");
        assert!(
            plan.pairs.iter().any(|pair| {
                pair.slots.iter().any(|s| s.school == "SEAS_MS")
                    && pair.slots.iter().any(|s| s.school == "SEAS" || s.school == "WH")
            }),
            "expected at least one grad↔undergrad overlap pair; pairs: {:?}",
            plan.pairs
        );
        let suggested: Vec<&String> = plan
            .opportunities
            .iter()
            .flat_map(|o| o.suggested_courses.iter())
            .collect();
        assert!(
            suggested.iter().any(|c| *c == "BEPP 2500"),
            "expected BEPP 2500 EE+WH overlap; opportunities: {:?}",
            plan.opportunities
        );
        assert!(
            suggested.iter().any(|c| *c == "ESE 3010"),
            "expected ESE 3010 EE+WH overlap; opportunities: {:?}",
            plan.opportunities
        );

        let scheduled_courses: Vec<&str> = output
            .schedule
            .iter()
            .flat_map(|sem| sem.courses.iter().map(String::as_str))
            .collect();
        assert!(
            scheduled_courses.contains(&"BEPP 2500"),
            "BEPP 2500 should appear as shared undergrad course; courses: {:?}",
            scheduled_courses
        );
        assert!(
            scheduled_courses.contains(&"MATH 1400"),
            "MATH 1400 should appear as shared undergrad course; courses: {:?}",
            scheduled_courses
        );

        let group_explanations: Vec<&str> = output
            .overlap_schedule_groups
            .iter()
            .map(|g| g.explanation.as_str())
            .collect();
        assert!(
            group_explanations.iter().any(|e| {
                e.contains("EE") && e.contains("WH_NOFL_MT") && e.contains("General Electives")
            }),
            "EE+WH general elective overlap should appear; groups: {:?}",
            group_explanations
        );
        assert!(
            !group_explanations.iter().any(|e| e.contains("BEPP 2500")),
            "BEPP 2500 should be a course card, not dashed overlap block; groups: {:?}",
            group_explanations
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Overlap discovery — dual degrees surface real double-count opportunities
// ═══════════════════════════════════════════════════════════════════════════════

mod overlap {
    use super::*;

    #[test]
    fn neur_wh_produces_cross_degree_overlap_hints() {
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH");
        let cu_map = catalog_cu_map();
        let empty: Vec<String> = vec![];
        let per_degree = vec![
            validate_courses_for_degree(neur.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
        ];
        let schools = vec!["CAS".into(), "WH".into()];
        let majors = vec!["NEUR".into(), "WH_NOFL".into()];
        let cross = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &[&neur, &wh],
            &schools,
            &majors,
            &HashSet::new(),
            &cross,
            &cu_map,
            None,
        );
        assert!(!plan.opportunities.is_empty());
        assert!(!plan.hints_by_slot.is_empty());
        assert!(!plan.pairs.is_empty());
    }

    #[test]
    fn overlap_pairs_always_span_exactly_two_degrees() {
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH");
        let econ = resolve_major("CAS", "ECON", &[]).expect("ECON");
        let cu_map = catalog_cu_map();
        let empty: Vec<String> = vec![];
        let per_degree = vec![
            validate_courses_for_degree(neur.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(econ.requirements.clone(), &empty, &cu_map),
        ];
        let schools = vec!["CAS".into(), "WH".into(), "CAS".into()];
        let majors = vec!["NEUR".into(), "WH_NOFL".into(), "ECON".into()];
        let cross = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &[&neur, &wh, &econ],
            &schools,
            &majors,
            &HashSet::new(),
            &cross,
            &cu_map,
            None,
        );
        for opp in &plan.opportunities {
            let degrees: HashSet<_> = opp.slots.iter().map(|s| s.degree_index).collect();
            assert_eq!(degrees.len(), 2, "overlap must pair two degrees");
        }
    }

    #[test]
    fn ee_wh_fl_mt_surfaces_shared_fundamentals_overlap() {
        let ee = resolve_major("SEAS", "EE", &[]).expect("EE");
        let wh = resolve_major("WH", "WH_FL_MT", &["FNCE".into()]).expect("WH_FL_MT");
        let cu_map = catalog_cu_map();
        let empty: Vec<String> = vec![];
        let per_degree = vec![
            validate_courses_for_degree(ee.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
        ];
        let schools = vec!["SEAS".into(), "WH".into()];
        let majors = vec!["EE".into(), "WH_FL_MT".into()];
        let cross = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &[&ee, &wh],
            &schools,
            &majors,
            &HashSet::new(),
            &cross,
            &cu_map,
            None,
        );

        let all_suggested: Vec<&String> = plan
            .opportunities
            .iter()
            .flat_map(|o| o.suggested_courses.iter())
            .collect();
        assert!(
            all_suggested.iter().any(|c| *c == "ESE 3010" || *c == "STAT 4300"),
            "EE + WH_FL_MT should surface shared math/stats fundamentals; opportunities: {:?}",
            plan.opportunities
        );
        assert!(
            !plan.pairs.is_empty(),
            "EE + WH_FL_MT should produce at least one overlap pair"
        );

        let fundamentals_overlap = plan.opportunities.iter().any(|o| {
            o.suggested_courses
                .iter()
                .any(|c| *c == "ESE 3010" || *c == "STAT 4300")
                && o.explanation.contains("Fundamentals")
        });
        assert!(
            fundamentals_overlap,
            "expected a fundamentals stats overlap opportunity; opportunities: {:?}",
            plan.opportunities
        );
    }

    #[test]
    fn overlap_group_ids_are_stable_and_recognizable() {
        let slots = vec![
            OverlapSlotRef {
                degree_index: 1,
                slot_key: "3:p0".into(),
                label: "Elective".into(),
                school: "WH".into(),
                major: "WH_NOFL".into(),
            },
            OverlapSlotRef {
                degree_index: 0,
                slot_key: "2:c1".into(),
                label: "Sector".into(),
                school: "CAS".into(),
                major: "NEUR".into(),
            },
        ];
        let id = overlap_group_schedule_id(&slots);
        assert!(id.starts_with("req:overlap:"));
        assert!(is_overlap_schedule_group_id(&id));
    }

    #[test]
    fn ee_robotics_wh_nofl_mt_surfaces_key_overlaps_on_schedule() {
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

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
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });

        let plan = output.overlap_plan.as_ref().expect("overlap plan");
        let suggested: Vec<&String> = plan
            .opportunities
            .iter()
            .flat_map(|o| o.suggested_courses.iter())
            .collect();
        assert!(
            suggested.iter().any(|c| *c == "ESE 3010"),
            "expected ESE 3010 overlap opportunity; got {:?}",
            plan.opportunities
        );
        assert!(
            suggested.iter().any(|c| *c == "MGMT 2370"),
            "expected MGMT 2370 overlap opportunity; got {:?}",
            plan.opportunities
        );
        assert!(
            plan.opportunities.iter().any(|o| {
                o.explanation.contains("Humanities")
                    || o.explanation.contains("Social Science")
            }),
            "expected humanities/social science overlap; got {:?}",
            plan.opportunities
        );

        let group_explanations: Vec<&str> = output
            .overlap_schedule_groups
            .iter()
            .map(|g| g.explanation.as_str())
            .collect();
        assert!(
            group_explanations.iter().any(|e| {
                e.contains("EE") && e.contains("WH_NOFL_MT") && e.contains("General Electives")
            }),
            "schedule should group EE+WH general elective overlap; groups: {:?}",
            group_explanations
        );

        let scheduled_courses: Vec<&str> = output
            .schedule
            .iter()
            .flat_map(|sem| sem.courses.iter().map(String::as_str))
            .collect();
        assert!(
            scheduled_courses.contains(&"MATH 1400"),
            "MATH 1400 should appear as a shared course card; courses: {:?}",
            scheduled_courses
        );
        assert!(
            scheduled_courses.contains(&"ESE 3010"),
            "ESE 3010 should appear as a shared course card; courses: {:?}",
            scheduled_courses
        );
        assert!(
            !group_explanations.iter().any(|e| e.contains("MATH 1400")),
            "MATH 1400 should not be a dashed overlap block; groups: {:?}",
            group_explanations
        );
        assert!(
            !group_explanations.iter().any(|e| {
                e.contains("Fundamentals") && e.contains("Math and Natural Science")
            }),
            "ESE 3010 fundamentals overlap should be a course card, not dashed block; groups: {:?}",
            group_explanations
        );
        assert!(
            scheduled_courses.contains(&"MGMT 2370"),
            "MGMT 2370 should appear as a shared course card (WH SingleCourse + EE option); courses: {:?}",
            scheduled_courses
        );
        assert!(
            !group_explanations.iter().any(|e| e.contains("MGMT 2370")),
            "MGMT 2370 should not be a dashed overlap block; groups: {:?}",
            group_explanations
        );
        assert!(
            scheduled_courses.contains(&"BEPP 2500"),
            "BEPP 2500 should appear as a shared course card (WH Fundamentals + EE/CIS pool); courses: {:?}",
            scheduled_courses
        );
        assert!(
            scheduled_courses.contains(&"FNCE 1010"),
            "FNCE 1010 should appear as a shared course card (WH Fundamentals + EE/CIS pool); courses: {:?}",
            scheduled_courses
        );
        assert!(
            !group_explanations.iter().any(|e| e.contains("BEPP 2500")),
            "BEPP 2500 should not be a dashed overlap block; groups: {:?}",
            group_explanations
        );
        assert!(
            !group_explanations.iter().any(|e| e.contains("FNCE 1010")),
            "FNCE 1010 should not be a dashed overlap block; groups: {:?}",
            group_explanations
        );
    }

    #[test]
    fn ee_wh_schedules_one_shared_stats_fundamental_not_both() {
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL_MT".into(),
                    school: "WH".into(),
                    kind: "major".to_string(),
                    concentrations: vec!["FNCE".into()],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        let scheduled: Vec<&str> = output
            .schedule
            .iter()
            .flat_map(|sem| sem.courses.iter().map(String::as_str))
            .collect();
        let has_ese = scheduled.contains(&"ESE 3010");
        let has_stat = scheduled.contains(&"STAT 4300");
        assert!(
            has_ese || has_stat,
            "EE+WH should schedule ESE 3010 or STAT 4300 as shared fundamentals; got {:?}",
            scheduled
        );
        assert!(
            !(has_ese && has_stat),
            "ESE 3010 and STAT 4300 should not both appear when one satisfies EE Math + WH Fundamentals; got {:?}",
            scheduled
        );
    }

    #[test]
    fn ee_wh_mgmt2370_fills_professional_electives_not_ese4000() {
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

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
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        let scheduled: Vec<&str> = output
            .schedule
            .iter()
            .flat_map(|sem| sem.courses.iter().map(String::as_str))
            .collect();
        assert!(
            scheduled.contains(&"MGMT 2370"),
            "MGMT 2370 should be the shared course for EE Professional Electives + WH M&T Soph; got {:?}",
            scheduled
        );
        assert!(
            !scheduled.contains(&"ESE 4000"),
            "ESE 4000 should not appear separately when MGMT 2370 satisfies the same EE Professional Electives slot; got {:?}",
            scheduled
        );
        let plan = output.overlap_plan.as_ref().expect("overlap plan");
        assert!(
            plan.pairs.iter().any(|p| {
                p.explanation.contains("Professional Electives") && p.explanation.contains("M&T Soph")
            }),
            "overlap pair should link EE Professional Electives to WH M&T Soph; pairs: {:?}",
            plan.pairs
        );
    }

    fn assert_shared_course_on_schedule(
        output: &scheduler::ScheduleOutput,
        course: &str,
        label: &str,
    ) {
        let scheduled_courses: Vec<&str> = output
            .schedule
            .iter()
            .flat_map(|sem| sem.courses.iter().map(String::as_str))
            .collect();
        let group_explanations: Vec<&str> = output
            .overlap_schedule_groups
            .iter()
            .map(|g| g.explanation.as_str())
            .collect();
        let plan = output.overlap_plan.as_ref().expect("overlap plan");
        let in_overlap_plan = plan
            .opportunities
            .iter()
            .flat_map(|o| o.suggested_courses.iter())
            .any(|c| c == course);
        assert!(
            in_overlap_plan,
            "{label}: expected {course} overlap opportunity; got {:?}",
            plan.opportunities
        );
        assert!(
            scheduled_courses.contains(&course),
            "{label}: {course} should appear as a shared course card; courses: {:?}",
            scheduled_courses
        );
        assert!(
            !group_explanations.iter().any(|e| e.contains(course)),
            "{label}: {course} should not be a dashed overlap block; groups: {:?}",
            group_explanations
        );
    }

    #[test]
    fn cis_wh_nofl_schedules_bepp2500_and_fnce1010_as_shared_course_cards() {
        let output = generate_schedule(dual_degree_input("SEAS", "CIS", "WH", "WH_NOFL"));
        assert_shared_course_on_schedule(&output, "BEPP 2500", "CIS+WH_NOFL");
        assert_shared_course_on_schedule(&output, "FNCE 1010", "CIS+WH_NOFL");
    }

    #[test]
    fn ee_wh_nofl_mt_schedules_bepp2500_and_fnce1010_as_shared_course_cards() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL_MT".into(),
                    school: "WH".into(),
                    kind: "major".to_string(),
                    concentrations: vec!["FNCE".into()],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert_shared_course_on_schedule(&output, "BEPP 2500", "EE+WH_NOFL_MT");
        assert_shared_course_on_schedule(&output, "FNCE 1010", "EE+WH_NOFL_MT");
    }

    #[test]
    fn taken_bepp2500_and_fnce1010_fulfill_wh_fundamentals_and_engineering_pools() {
        let cu_map = catalog_cu_map();
        let taken = vec!["BEPP 2500".to_string(), "FNCE 1010".to_string()];
        let cis = resolve_major("SEAS", "CIS", &[]).expect("CIS");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH_NOFL");
        let mut per_degree = vec![
            validate_courses_for_degree(cis.requirements.clone(), &taken, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &taken, &cu_map),
        ];
        resolve_cross_degree_conflicts(
            &mut per_degree,
            &["SEAS".into(), "WH".into()],
            &["CIS".into(), "WH_NOFL".into()],
            &cu_map,
            None,
            Some(&taken),
            None,
        );
        for course in ["BEPP 2500", "FNCE 1010"] {
            assert!(
                per_degree[1]
                    .fulfilled
                    .iter()
                    .any(|m| requirement_explicitly_lists_course(&m.requirement, course)),
                "WH fundamentals should be fulfilled by taken {course}"
            );
            assert!(
                per_degree[0]
                    .fulfilled
                    .iter()
                    .any(|m| requirement_accepts_shared_course(&m.requirement, course)),
                "CIS should accept taken {course} toward a technical elective pool"
            );
        }
    }

    #[test]
    fn taken_mgmt2370_fulfills_wh_single_course_and_ee_professional_elective() {
        let cu_map = catalog_cu_map();
        let taken = vec!["MGMT 2370".to_string()];
        let ee = resolve_major("SEAS", "EE", &[]).expect("EE");
        let wh = resolve_major("WH", "WH_NOFL_MT", &["FNCE".into()]).expect("WH_NOFL_MT");
        let mut per_degree = vec![
            validate_courses_for_degree(ee.requirements.clone(), &taken, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &taken, &cu_map),
        ];
        resolve_cross_degree_conflicts(
            &mut per_degree,
            &["SEAS".into(), "WH".into()],
            &["EE".into(), "WH_NOFL_MT".into()],
            &cu_map,
            None,
            Some(&taken),
            None,
        );
        assert!(
            per_degree[0]
                .fulfilled
                .iter()
                .any(|m| {
                    requirement_explicitly_lists_course(&m.requirement, "MGMT 2370")
                        || requirement_accepts_shared_course(&m.requirement, "MGMT 2370")
                }),
            "EE should accept taken MGMT 2370 toward professional electives"
        );
        assert!(
            per_degree[1]
                .fulfilled
                .iter()
                .any(|m| requirement_explicitly_lists_course(&m.requirement, "MGMT 2370")),
            "WH M&T soph SingleCourse should be fulfilled by taken MGMT 2370"
        );
    }

    #[test]
    fn requirement_accepts_shared_course_for_explicit_or_pool() {
        let wh_single = Requirement::SingleCourse {
            category: Some("M&T Soph Course".into()),
            possibilities: vec!["MGMT 2370".into()],
        };
        let ee_anyof = Requirement::AnyOf {
            category: Some("Professional Electives".into()),
            possibilities: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec![
                        "ESE 4000".into(),
                        "MGMT 2370".into(),
                        "OIDD 2360".into(),
                    ],
                },
                Requirement::Restriction {
                    category: None,
                    department: None,
                    cu: None,
                    level: None,
                    max_level: None,
                    attr: Some(vec!["EUNG".into()]),
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
            ],
        };
        let ee_restriction_only = Requirement::Restriction {
            category: Some("Professional Electives".into()),
            department: None,
            cu: None,
            level: None,
            max_level: None,
            attr: Some(vec!["EUNG".into(), "EUMA".into(), "EUNS".into()]),
            excluding: None,
            number: 1,
            no_school: None,
        };

        assert!(requirement_explicitly_lists_course(&wh_single, "MGMT 2370"));
        assert!(requirement_accepts_shared_course(&wh_single, "MGMT 2370"));
        assert!(requirement_accepts_shared_course(&ee_anyof, "MGMT 2370"));
        assert!(!requirement_explicitly_lists_course(&ee_restriction_only, "MGMT 2370"));
        assert!(requirement_accepts_shared_course(&ee_restriction_only, "ESE 4000"));
        assert!(!requirement_accepts_shared_course(&ee_restriction_only, "NOT A COURSE"));
    }

    #[test]
    fn cis_wh_overlap_plan_includes_bepp2500_and_fnce1010() {
        let cis = resolve_major("SEAS", "CIS", &[]).expect("CIS");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH");
        let cu_map = catalog_cu_map();
        let empty: Vec<String> = vec![];
        let per_degree = vec![
            validate_courses_for_degree(cis.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
        ];
        let schools = vec!["SEAS".into(), "WH".into()];
        let majors = vec!["CIS".into(), "WH_NOFL".into()];
        let cross = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &[&cis, &wh],
            &schools,
            &majors,
            &HashSet::new(),
            &cross,
            &cu_map,
            None,
        );
        for course in ["BEPP 2500", "FNCE 1010"] {
            assert!(
                plan.opportunities
                    .iter()
                    .flat_map(|o| o.suggested_courses.iter())
                    .any(|c| c == course),
                "CIS+WH should surface {course} overlap; opportunities: {:?}",
                plan.opportunities
            );
        }
    }

    #[test]
    fn ee_wh_overlap_plan_includes_bepp2500_and_fnce1010() {
        let ee = resolve_major("SEAS", "EE", &[]).expect("EE");
        let wh = resolve_major("WH", "WH_NOFL_MT", &["FNCE".into()]).expect("WH_NOFL_MT");
        let cu_map = catalog_cu_map();
        let empty: Vec<String> = vec![];
        let per_degree = vec![
            validate_courses_for_degree(ee.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
        ];
        let schools = vec!["SEAS".into(), "WH".into()];
        let majors = vec!["EE".into(), "WH_NOFL_MT".into()];
        let cross = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &[&ee, &wh],
            &schools,
            &majors,
            &HashSet::new(),
            &cross,
            &cu_map,
            None,
        );
        for course in ["BEPP 2500", "FNCE 1010"] {
            assert!(
                plan.opportunities
                    .iter()
                    .flat_map(|o| o.suggested_courses.iter())
                    .any(|c| c == course),
                "EE+WH should surface {course} overlap; opportunities: {:?}",
                plan.opportunities
            );
        }
    }

    #[test]
    fn cas_wh_overlap_plan_pairs_writ_requirements() {
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH");
        let cu_map = catalog_cu_map();
        let empty: Vec<String> = vec![];
        let per_degree = vec![
            validate_courses_for_degree(neur.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
        ];
        let schools = vec!["CAS".into(), "WH".into()];
        let majors = vec!["NEUR".into(), "WH_NOFL".into()];
        let cross = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &[&neur, &wh],
            &schools,
            &majors,
            &HashSet::new(),
            &cross,
            &cu_map,
            None,
        );
        assert!(
            overlap_plan_has_writ_opportunity(&plan),
            "CAS + WH should pair WRIT department requirements; opportunities: {:?}",
            plan.opportunities
        );
        assert!(
            plan.pairs.iter().any(|pair| {
                pair.slots.iter().any(|s| is_writ_slot_label(&s.label))
                    && pair.slots.iter().map(|s| s.degree_index).collect::<HashSet<_>>().len() == 2
            }),
            "expected a cross-degree WRIT pair; pairs: {:?}",
            plan.pairs
        );
    }

    #[test]
    fn neur_wh_unrestricted_elective_overlap_pairs() {
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH");
        let cu_map = catalog_cu_map();
        let empty: Vec<String> = vec![];
        let per_degree = vec![
            validate_courses_for_degree(neur.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
        ];
        let schools = vec!["CAS".into(), "WH".into()];
        let majors = vec!["NEUR".into(), "WH_NOFL".into()];
        let cross = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &[&neur, &wh],
            &schools,
            &majors,
            &HashSet::new(),
            &cross,
            &cu_map,
            None,
        );
        assert!(
            plan.pairs.iter().any(|pair| {
                pair.slots.iter().any(|s| s.label == "Unrestricted Electives")
                    && pair
                        .slots
                        .iter()
                        .map(|s| s.degree_index)
                        .collect::<HashSet<_>>()
                        .len()
                        == 2
            }),
            "WH Unrestricted Electives should pair cross-degree; pairs: {:?}",
            plan.pairs.iter().map(|p| p.slots.iter().map(|s| &s.label).collect::<Vec<_>>()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn neur_wh_cas_gened_wh_las_pool_overlap_pairs() {
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH");
        let cu_map = catalog_cu_map();
        let empty: Vec<String> = vec![];
        let per_degree = vec![
            validate_courses_for_degree(neur.requirements.clone(), &empty, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
        ];
        let schools = vec!["CAS".into(), "WH".into()];
        let majors = vec!["NEUR".into(), "WH_NOFL".into()];
        let cross = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &[&neur, &wh],
            &schools,
            &majors,
            &HashSet::new(),
            &cross,
            &cu_map,
            None,
        );

        fn cas_gened_side(label: &str) -> bool {
            label.contains("General Education")
                || label.starts_with("Foundational Approaches")
                || label.starts_with("Sectors of Knowledge")
        }

        fn wh_las_side(label: &str) -> bool {
            label.contains("Liberal Arts and Sciences")
                || label.contains("Humanities (WUHM)")
                || label.contains("Natural Science & Math")
                || label.contains("Social Science (WUSS)")
        }

        let overlaps_las = |labels: &[&str]| {
            labels.iter().any(|l| cas_gened_side(l)) && labels.iter().any(|l| wh_las_side(l))
        };

        assert!(
            plan.opportunities
                .iter()
                .any(|opp| overlaps_las(&opp.slots.iter().map(|s| s.label.as_str()).collect::<Vec<_>>()))
                || plan.pairs.iter().any(|pair| {
                    overlaps_las(&pair.slots.iter().map(|s| s.label.as_str()).collect::<Vec<_>>())
                }),
            "CAS gen-ed should overlap WH LAS pool or SSH constraints; pairs: {:?}",
            plan.pairs.iter().map(|p| p.slots.iter().map(|s| &s.label).collect::<Vec<_>>()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn neur_wh_unrestricted_elective_overlap_on_schedule() {
        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL"));
        assert!(output.error.is_none(), "{:?}", output.error);
        assert!(
            output.overlap_schedule_groups.iter().any(|g| {
                g.members
                    .iter()
                    .any(|m| m.label == "Unrestricted Electives")
            }),
            "expected WH Unrestricted Electives overlap group on schedule; groups: {:?}",
            output
                .overlap_schedule_groups
                .iter()
                .map(|g| g.members.iter().map(|m| &m.label).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn neur_wh_chem_1011_overlaps_neur_major_with_wh_wunm() {
        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL"));
        assert!(output.error.is_none(), "{:?}", output.error);
        let summary = output
            .cross_degree_summary
            .as_ref()
            .expect("cross degree summary");
        let allocs = summary
            .course_allocations
            .get("CHEM 1011")
            .expect("CHEM 1011 should appear on schedule");
        assert_eq!(allocs.len(), 2);
        assert_eq!(allocs[0].major, "NEUR");
        assert_eq!(allocs[1].major, "WH_NOFL");

        let plan = output.overlap_plan.as_ref().expect("overlap plan");
        assert!(
            plan.pairs.iter().any(|pair| {
                pair.slots.iter().any(|s| s.label == "Introductory Chemistry")
                    && pair
                        .slots
                        .iter()
                        .any(|s| s.label.contains("Natural Science & Math"))
            }),
            "CHEM 1011 should overlap NEUR intro chem with WH WUNM; pairs: {:?}",
            plan.pairs.iter().map(|p| p.slots.iter().map(|s| &s.label).collect::<Vec<_>>()).collect::<Vec<_>>()
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Scheduling & CU policy — generated plans respect workload limits
// ═══════════════════════════════════════════════════════════════════════════════

mod scheduling {
    use super::*;

    #[test]
    fn scheduled_courses_have_degree_stripe_mapping_undergrad_plus_grad() {
        let cases = [
            (
                "EE+MS_ROBO (Data Science conc)",
                dual_degree_input_with_conc(
                    "SEAS",
                    "EE",
                    Some("Data Science"),
                    "SEAS_MS",
                    "MS_ROBO",
                    None,
                ),
            ),
            (
                "CIS+MS_ROBO",
                dual_degree_input("SEAS", "CIS", "SEAS_MS", "MS_ROBO"),
            ),
            (
                "NEUR+MS_ROBO",
                dual_degree_input("CAS", "NEUR", "SEAS_MS", "MS_ROBO"),
            ),
            (
                "EE+MS_EE",
                dual_degree_input("SEAS", "EE", "SEAS_MS", "MS_EE"),
            ),
            (
                "MS_ROBO+MS_EE",
                dual_degree_input("SEAS_MS", "MS_ROBO", "SEAS_MS", "MS_EE"),
            ),
        ];
        for (label, input) in cases {
            assert_scheduled_courses_have_stripe_mapping(&generate_schedule(input), label);
        }
    }

    #[test]
    fn scheduled_courses_have_degree_stripe_mapping_dual_undergrad() {
        for (label, input) in implemented_dual_undergrad_pairs() {
            assert_scheduled_courses_have_stripe_mapping(&generate_schedule(input), &label);
        }
        assert_scheduled_courses_have_stripe_mapping(
            &generate_schedule(dual_degree_input("CAS", "ECON", "CAS", "CIS")),
            "CAS ECON+CIS",
        );
        assert_scheduled_courses_have_stripe_mapping(
            &generate_schedule(dual_degree_input("SEAS", "EE", "CAS", "ECON")),
            "SEAS EE+CAS ECON",
        );
    }

    #[test]
    fn year_one_fall_is_always_five_point_five() {
        let schools = vec!["SEAS".into(), "WH".into()];
        assert_eq!(default_semester_cu_limit(&schools, 1, "Fall"), 5.5);
        assert_eq!(default_semester_cu_limit(&schools, 1, "Spring"), 6.5);
    }

    #[test]
    fn dual_non_cas_undergrad_gets_six_point_five_and_five_years() {
        let schools = vec!["CAS".into(), "WH".into()];
        assert_eq!(default_semester_cu_limit(&schools, 2, "Fall"), 6.5);
        assert_eq!(undergrad_schedule_years(&schools), 5);
    }

    #[test]
    fn dual_cas_stays_four_years_at_five_point_five() {
        let schools = vec!["CAS".into(), "CAS".into()];
        assert_eq!(default_semester_cu_limit(&schools, 2, "Fall"), 5.5);
        assert_eq!(undergrad_schedule_years(&schools), 4);
    }

    #[test]
    fn single_degree_defaults_to_four_years_five_point_five() {
        let schools = vec!["SEAS".into()];
        assert_eq!(default_semester_cu_limit(&schools, 3, "Spring"), 5.5);
        assert_eq!(undergrad_schedule_years(&schools), 4);
    }

    #[test]
    fn single_undergrad_plus_grad_stays_five_point_five() {
        let schools = vec!["SEAS".into(), "SEAS_MS".into()];
        assert_eq!(default_semester_cu_limit(&schools, 2, "Fall"), 5.5);
        assert_eq!(undergrad_schedule_years(&schools), 4);
        assert!(!cross_degree::has_dual_undergrad(&schools));
    }

    #[test]
    fn dual_undergrad_plus_grad_gets_six_point_five_except_y1f() {
        let schools = vec!["SEAS".into(), "WH".into(), "SEAS_MS".into()];
        assert!(!scheduler::dual_undergrad_only(&schools));
        assert!(cross_degree::has_dual_undergrad(&schools));
        assert_eq!(default_semester_cu_limit(&schools, 1, "Fall"), 5.5);
        assert_eq!(default_semester_cu_limit(&schools, 1, "Spring"), 6.5);
        assert_eq!(default_semester_cu_limit(&schools, 2, "Fall"), 6.5);
        assert_eq!(undergrad_schedule_years(&schools), 5);
    }

    #[test]
    fn dual_cas_undergrad_plus_grad_stays_five_point_five() {
        let schools = vec!["CAS".into(), "CAS".into(), "SEAS_MS".into()];
        assert!(cross_degree::has_dual_undergrad(&schools));
        assert_eq!(default_semester_cu_limit(&schools, 2, "Fall"), 5.5);
        assert_eq!(undergrad_schedule_years(&schools), 4);
    }

    #[test]
    fn summer_cap_is_two_cu() {
        assert_eq!(default_semester_cu_limit(&vec!["SEAS".into()], 2, "Summer"), 2.0);
    }

    fn assert_dual_schedule(output: &scheduler::ScheduleOutput, label: &str) {
        assert!(output.error.is_none(), "{label}: pipeline error");
        assert_eq!(output.degree_results.len(), 2);
        assert_schedule_respects_cu_limits(output, label);
        let max_year = output.schedule.iter().map(|p| p.year).max().unwrap_or(0);
        assert!(
            max_year <= 6,
            "{label}: schedule extends past year 6 (got {max_year})"
        );
        let pairs = output
            .overlap_plan
            .as_ref()
            .map(|p| p.pairs.len())
            .unwrap_or(0);
        assert!(
            !output.overlap_schedule_groups.is_empty() || pairs > 0,
            "{label}: expected overlap hints"
        );
    }

    #[test]
    fn neur_wh_dual_degree_generates_valid_schedule() {
        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL"));
        assert_dual_schedule(&output, "NEUR + WH_NOFL");
    }

    #[test]
    fn cis_wh_dual_degree_generates_valid_schedule() {
        let output = generate_schedule(dual_degree_input("SEAS", "CIS", "WH", "WH_NOFL"));
        assert_dual_schedule(&output, "CIS + WH_NOFL");
    }

    #[test]
    fn ee_wh_fl_mt_dual_degree_generates_valid_schedule() {
        let output = generate_schedule(dual_degree_input("SEAS", "EE", "WH", "WH_FL_MT"));
        assert_dual_schedule(&output, "EE + WH_FL_MT");
        let overlap = output.overlap_plan.as_ref().expect("overlap plan");
        let shared_stats = overlap
            .opportunities
            .iter()
            .flat_map(|o| o.suggested_courses.iter())
            .any(|c| c == "ESE 3010" || c == "STAT 4300");
        assert!(shared_stats, "EE + WH_FL_MT schedule should surface shared stats overlap");
    }

    #[test]
    fn single_cis_schedule_has_no_cross_degree_summary() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "CIS".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.cross_degree_summary.is_none());
        assert_schedule_respects_cu_limits(&output, "single CIS");
    }

    #[test]
    fn nurs_bsn_generates_valid_schedule() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "BSN".into(),
                school: "NURS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        assert_schedule_respects_cu_limits(&output, "NURS BSN");
        assert_eq!(undergrad_schedule_years(&vec!["NURS".into()]), 4);
    }

    #[test]
    fn nutr_bsn_generates_valid_schedule() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "NUTR_BSN".into(),
                school: "NURS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        assert_schedule_respects_cu_limits(&output, "NURS NUTR_BSN");
    }

    #[test]
    fn schedule_items_are_courses_or_requirement_slots() {
        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL"));
        for plan in &output.schedule {
            for c in &plan.courses {
                assert!(
                    course::is_valid_course_code(c),
                    "invalid course on schedule: {c}"
                );
            }
            for s in &plan.requirement_slots {
                assert!(
                    is_requirement_slot_id(s),
                    "invalid slot id on schedule: {s}"
                );
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7b. Dual-degree properties — shared requirements (e.g. WRIT) appear once
// ═══════════════════════════════════════════════════════════════════════════════

mod dual_degree_properties {
    use super::*;

    #[test]
    fn cas_and_wh_majors_both_include_writ_requirement() {
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH_NOFL");
        assert!(major_has_writ_requirement(&neur), "CAS majors include WRIT");
        assert!(major_has_writ_requirement(&wh), "Wharton majors include WRIT");
    }

    #[test]
    fn seas_undergrad_majors_do_not_model_writ_yet() {
        let cis = resolve_major("SEAS", "CIS", &[]).expect("CIS");
        assert!(
            !major_has_writ_requirement(&cis),
            "SEAS data has no explicit WRIT slot yet — test documents current catalog"
        );
    }

    #[test]
    fn neur_wh_cas_gened_wh_overlap_uses_gen_ed_label_and_caps_flex_slots() {
        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL"));
        assert!(output.error.is_none(), "pipeline error: {:?}", output.error);

        let cas_gened_overlaps = output
            .overlap_schedule_groups
            .iter()
            .filter(|g| {
                g.members.iter().any(|m| {
                    m.school == "CAS"
                        && output
                            .slot_labels
                            .get(&g.group_id)
                            .is_some_and(|l| l.contains("General Education"))
                })
            })
            .count();
        assert!(
            cas_gened_overlaps > 0,
            "expected CAS gen-ed / WH LAS overlap groups"
        );

        for group in &output.overlap_schedule_groups {
            for member in &group.members {
                if member.school != "CAS" {
                    continue;
                }
                if member
                    .label
                    .contains("Foundational Approaches")
                    || member.label.contains("Sectors of Knowledge")
                {
                    panic!(
                        "CAS overlap member should use gen-ed pool label, got: {}",
                        member.label
                    );
                }
            }
        }

        let mut gen_ed_flex = 0usize;
        let mut gen_ed_overlap = 0usize;
        for plan in &output.schedule {
            for slot in &plan.requirement_slots {
                if output
                    .overlap_schedule_groups
                    .iter()
                    .any(|g| &g.group_id == slot)
                {
                    if output
                        .slot_labels
                        .get(slot)
                        .is_some_and(|l| l.contains("General Education"))
                    {
                        gen_ed_overlap += 1;
                    }
                } else if output
                    .slot_labels
                    .get(slot)
                    .is_some_and(|l| l == "General Education")
                {
                    gen_ed_flex += 1;
                }
            }
        }
        assert!(
            gen_ed_flex + gen_ed_overlap <= 12,
            "expected at most 12 gen-ed schedule items (flex + WH overlaps), got flex={} overlap={}",
            gen_ed_flex,
            gen_ed_overlap
        );
    }

    #[test]
    fn neur_wh_discover_writ_overlap_and_schedule_once() {
        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL"));
        assert!(output.error.is_none(), "pipeline error: {:?}", output.error);

        let plan = output.overlap_plan.as_ref().expect("overlap plan");
        assert!(
            overlap_plan_has_writ_opportunity(plan),
            "NEUR + WH should surface WRIT overlap; opportunities: {:?}",
            plan.opportunities
        );

        let writ_units = writ_cu_units_on_schedule(&output);
        assert_eq!(
            writ_units, 1.0,
            "shared WRIT should occupy exactly 1 CU on the schedule; got {writ_units}"
        );
    }

    #[test]
    fn econ_wh_writ_overlap_appears_once() {
        let output = generate_schedule(dual_degree_input("CAS", "ECON", "WH", "WH_FL"));
        let plan = output.overlap_plan.as_ref().expect("overlap plan");
        assert!(overlap_plan_has_writ_opportunity(plan));
        assert_eq!(writ_cu_units_on_schedule(&output), 1.0);
    }

    #[test]
    fn cas_single_econ_gen_ed_capped_at_twelve_with_unrestricted_electives() {
        use degree_planner::penn_data::college_data::{
            cas_auto_completed_sectors_for, cas_major_pool_major_cu, cas_open_gen_ed_slot_count,
            cas_shared_gened_flex_slots, cas_shared_unrestricted_elective_count, create_econ_major,
            CAS_DEGREE_CU, CAS_UNRESTRICTED_ELECTIVES_CATEGORY,
        };
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let major = create_econ_major();
        let major_cu = cas_major_pool_major_cu(&major);
        let autos = cas_auto_completed_sectors_for("ECON", None);
        let open = cas_open_gen_ed_slot_count(&autos);
        let gen_ed_flex = cas_shared_gened_flex_slots(major_cu, open);
        let unrestricted = cas_shared_unrestricted_elective_count(major_cu, open);

        assert!(gen_ed_flex <= 12, "gen-ed flex should cap at 12, got {gen_ed_flex}");
        assert!(
            unrestricted > 0,
            "expected unrestricted electives beyond gen-ed slots"
        );
        assert_eq!(
            1 + major_cu + gen_ed_flex + unrestricted,
            CAS_DEGREE_CU,
            "CAS degree should total 36 CU"
        );

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "ECON".into(),
                school: "CAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);

        let gen_ed_slots: usize = output
            .schedule
            .iter()
            .flat_map(|p| p.requirement_slots.iter())
            .filter(|s| {
                output
                    .slot_labels
                    .get(s.as_str())
                    .is_some_and(|l| l == "General Education")
            })
            .count();
        assert!(
            gen_ed_slots <= 12,
            "single CAS ECON should schedule at most 12 gen-ed slots, got {gen_ed_slots}"
        );
        assert_eq!(
            gen_ed_slots as i32, gen_ed_flex,
            "scheduled gen-ed flex should match open coverage needs"
        );

        let unrest_rows = output.degree_results[0]
            .unfulfilled_requirements
            .iter()
            .chain(output.degree_results[0].fulfilled_requirements.iter())
            .filter(|m| m.requirement.get_category() == CAS_UNRESTRICTED_ELECTIVES_CATEGORY)
            .count();
        assert_eq!(unrest_rows as i32, unrestricted);
    }

    #[test]
    fn cas_anch_non_gened_courses_allocate_to_unrestricted_not_pool_flex() {

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "ANCH".into(),
                school: "CAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![
                FrozenCourse {
                    course_id: "AFRC 1100".into(),
                    year: 1,
                    semester: "Fall".into(),
                },
                FrozenCourse {
                    course_id: "AFRC 0527".into(),
                    year: 1,
                    semester: "Spring".into(),
                },
            ],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);

        let dr = &output.degree_results[0];
        let gen_ed_flex: Vec<_> = dr
            .fulfilled_requirements
            .iter()
            .filter(|m| m.instance_id.as_deref().is_some_and(|id| id.starts_with("1:p")))
            .collect();
        assert!(
            gen_ed_flex.is_empty(),
            "non-gen-ed courses should not fill pool flex slots: {:?}",
            gen_ed_flex
                .iter()
                .map(|m| (&m.instance_id, &m.course_ids))
                .collect::<Vec<_>>()
        );

        let unrestricted: Vec<_> = dr
            .fulfilled_requirements
            .iter()
            .filter(|m| {
                m.requirement.get_category() == "Unrestricted Electives"
                    && m.course_ids.iter().any(|c| c.starts_with("AFRC"))
            })
            .collect();
        assert_eq!(
            unrestricted.len(),
            2,
            "expected both AFRC courses as unrestricted electives, got {:?}",
            unrestricted
                .iter()
                .map(|m| (&m.instance_id, &m.course_ids))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn cas_anch_redundant_gened_course_goes_unrestricted_after_sector_covered() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "ANCH".into(),
                school: "CAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![
                FrozenCourse {
                    course_id: "ANTH 0040".into(),
                    year: 1,
                    semester: "Fall".into(),
                },
                FrozenCourse {
                    course_id: "ANTH 1238".into(),
                    year: 1,
                    semester: "Spring".into(),
                },
            ],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);

        let dr = &output.degree_results[0];
        let pool_flex_with_anth: Vec<_> = dr
            .fulfilled_requirements
            .iter()
            .filter(|m| {
                m.instance_id.as_deref().is_some_and(|id| id.starts_with("1:p"))
                    && m.course_ids.iter().any(|c| c == "ANTH 0040")
            })
            .collect();
        assert_eq!(
            pool_flex_with_anth.len(),
            1,
            "ANTH 0040 should fill one unmet gen-ed constraint in the pool"
        );

        let anth_in_pool: Vec<_> = dr
            .fulfilled_requirements
            .iter()
            .filter(|m| {
                m.instance_id.as_deref().is_some_and(|id| id.starts_with("1:p"))
                    && m.course_ids.iter().any(|c| c == "ANTH 1238")
            })
            .collect();
        assert!(
            anth_in_pool.is_empty(),
            "second Hum/Soc Sci course should not enter pool when that sector is already covered: {:?}",
            anth_in_pool
        );

        let anth_unrestricted = dr.fulfilled_requirements.iter().any(|m| {
            m.requirement.get_category() == "Unrestricted Electives"
                && m.course_ids.iter().any(|c| c == "ANTH 1238")
        });
        assert!(
            anth_unrestricted,
            "redundant sector course should count as unrestricted elective"
        );
    }

    #[test]
    fn seas_ee_cas_econ_gen_ed_slots_capped_at_twelve() {
        let output = generate_schedule(dual_degree_input("SEAS", "EE", "CAS", "ECON"));
        assert!(output.error.is_none(), "{:?}", output.error);

        let mut sched_slots = Vec::new();
        for p in &output.schedule {
            sched_slots.extend(p.requirement_slots.iter().cloned());
        }
        let gen_ed_slots: Vec<_> = sched_slots
            .iter()
            .filter(|s| {
                output
                    .slot_labels
                    .get(s.as_str())
                    .is_some_and(|l| l == "General Education")
            })
            .collect();

        let gen_ed_overlaps = output
            .overlap_schedule_groups
            .iter()
            .filter(|g| {
                output
                    .slot_labels
                    .get(&g.group_id)
                    .is_some_and(|l| l.contains("General Education"))
            })
            .count();

        assert!(
            gen_ed_slots.len() + gen_ed_overlaps <= 12,
            "expected at most 12 CAS gen-ed schedule items for SEAS+ECON dual degree, got flex={} overlap={}: flex={:?}",
            gen_ed_slots.len(),
            gen_ed_overlaps,
            gen_ed_slots
        );
    }

    #[test]
    fn cas_econ_cis_gen_ed_slots_scheduled_once() {
        let output = generate_schedule(dual_degree_input("CAS", "ECON", "CAS", "CIS"));
        let mut sched_slots = Vec::new();
        for p in &output.schedule {
            sched_slots.extend(p.requirement_slots.iter().cloned());
        }
        let gen_ed_slots: Vec<_> = sched_slots
            .iter()
            .filter(|s| {
                output
                    .slot_labels
                    .get(s.as_str())
                    .is_some_and(|l| l.contains("General Education"))
            })
            .collect();
        let open_from_api: Vec<_> = output
            .degree_results
            .iter()
            .flat_map(|r| r.suggested_for_unfulfilled.iter())
            .flat_map(|m| m.course_ids.iter())
            .filter(|id| is_schedulable_requirement_slot_id(id))
            .filter(|id| {
                output
                    .slot_labels
                    .get(id.as_str())
                    .is_some_and(|l| l.contains("General Education"))
            })
            .collect();
        eprintln!(
            "gen_ed schedule slots={} api open={}",
            gen_ed_slots.len(),
            open_from_api.len()
        );
        for r in &output.degree_results {
            let n = r
                .suggested_for_unfulfilled
                .iter()
                .filter(|m| {
                    m.course_ids.iter().any(|id| {
                        output
                            .slot_labels
                            .get(id)
                            .is_some_and(|l| l.contains("General Education"))
                    })
                })
                .count();
            eprintln!(
                "{} {} suggested gen-ed mapped rows={}",
                r.school, r.major, n
            );
        }
        if let Some(pool) = output.degree_results[0]
            .pool_coverage_info
            .iter()
            .find(|p| p.category == "General Education")
        {
            eprintln!(
                "primary pool flex total={} filled={}",
                pool.flexible_slots_total, pool.flexible_slots_filled
            );
        }
        let expected_flex = {
            use degree_planner::penn_data::college_data::{
                cas_college_auto_completed_sectors, cas_effective_combined_major_cu,
                cas_open_gen_ed_slot_count, cas_shared_gened_flex_slots,
            };
            let econ = resolve_major("CAS", "ECON", &[]).expect("ECON");
            let cis = resolve_major("CAS", "CIS", &[]).expect("CIS");
            let majors = vec![&econ, &cis];
            let combined = cas_effective_combined_major_cu(&majors, 0);
            let open = cas_open_gen_ed_slot_count(&cas_college_auto_completed_sectors(&majors));
            cas_shared_gened_flex_slots(combined, open) as usize
        };
        assert_eq!(
            gen_ed_slots.len(),
            expected_flex,
            "expected {expected_flex} shared gen-ed flex slots on schedule, got {}: {:?}",
            gen_ed_slots.len(),
            gen_ed_slots
        );
    }

    #[test]
    fn cas_double_major_major_course_not_absorbed_as_unrestricted() {
        // CIS listed first → primary for college-wide residual slots. SOCI 1000 fills Society
        // gen-ed; ECON 0200 must stay on the ECON major and must NOT also fill unrestricted.
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "CIS".into(),
                    school: "CAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "ECON".into(),
                    school: "CAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
            ],
            frozen: vec![
                FrozenCourse {
                    course_id: "SOCI 1000".into(),
                    year: 1,
                    semester: "Fall".into(),
                },
                FrozenCourse {
                    course_id: "ECON 0200".into(),
                    year: 1,
                    semester: "Spring".into(),
                },
            ],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);

        let econ = output
            .degree_results
            .iter()
            .find(|r| r.major == "ECON")
            .expect("ECON");
        assert!(
            econ.fulfilled_requirements.iter().any(|m| {
                m.requirement.get_category() == "Introductory Economics"
                    && m.course_ids.iter().any(|c| c == "ECON 0200")
            }),
            "ECON 0200 should fulfill Introductory Economics"
        );

        for dr in &output.degree_results {
            let unrestricted_econ: Vec<_> = dr
                .fulfilled_requirements
                .iter()
                .filter(|m| {
                    m.requirement.get_category() == "Unrestricted Electives"
                        && m.course_ids.iter().any(|c| c == "ECON 0200")
                })
                .collect();
            assert!(
                unrestricted_econ.is_empty(),
                "{} {} must not place ECON 0200 in Unrestricted Electives: {:?}",
                dr.school,
                dr.major,
                unrestricted_econ
                    .iter()
                    .map(|m| &m.course_ids)
                    .collect::<Vec<_>>()
            );

            let flex_econ = dr.fulfilled_requirements.iter().any(|m| {
                m.instance_id.as_deref().is_some_and(|id| id.contains(":p"))
                    && m.course_ids.iter().any(|c| c == "ECON 0200")
            });
            assert!(
                !flex_econ,
                "{} {} must not absorb ECON 0200 into gen-ed flex when it is an ECON major course",
                dr.school,
                dr.major
            );
        }
    }

    #[test]
    fn cas_cis_no_within_major_double_count_of_core_as_elective() {
        // CIS 1600 fulfills Core; the CIS Elective restriction must not reclaim it.
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "CIS".into(),
                    school: "CAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "ECON".into(),
                    school: "CAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
            ],
            frozen: vec![FrozenCourse {
                course_id: "CIS 1600".into(),
                year: 1,
                semester: "Fall".into(),
            }],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);

        let cis = output
            .degree_results
            .iter()
            .find(|r| r.major == "CIS")
            .expect("CIS");
        let cis1600_rows: Vec<_> = cis
            .fulfilled_requirements
            .iter()
            .filter(|m| m.course_ids.iter().any(|c| c == "CIS 1600"))
            .collect();
        assert_eq!(
            cis1600_rows.len(),
            1,
            "CIS 1600 must fulfill exactly one CIS major slot, got {:?}",
            cis1600_rows
                .iter()
                .map(|m| (m.requirement.get_category(), &m.instance_id, &m.course_ids))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            cis1600_rows[0].requirement.get_category(),
            "Core Courses",
            "CIS 1600 should count as Core, not CIS Elective"
        );
        assert!(
            !cis.fulfilled_requirements.iter().any(|m| {
                m.requirement.get_category() == "CIS Elective"
                    && m.course_ids.iter().any(|c| c == "CIS 1600")
            }),
            "CIS 1600 must not also fulfill CIS Elective"
        );
    }

    #[test]
    fn cas_double_major_fnce_unrestricted_primary_only() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "CIS".into(),
                    school: "CAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "ECON".into(),
                    school: "CAS".into(),
                    kind: "major".into(),
                    concentrations: vec![],
                    concentration: None,
                },
            ],
            frozen: vec![FrozenCourse {
                course_id: "FNCE 1010".into(),
                year: 2,
                semester: "Fall".into(),
            }],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);

        let primary = &output.degree_results[0];
        assert_eq!(primary.major, "CIS");
        assert!(
            primary.fulfilled_requirements.iter().any(|m| {
                m.requirement.get_category() == "Unrestricted Electives"
                    && m.course_ids.iter().any(|c| c == "FNCE 1010")
            }),
            "FNCE 1010 should fulfill primary Unrestricted Electives"
        );

        let secondary = &output.degree_results[1];
        assert!(
            !secondary.fulfilled_requirements.iter().any(|m| {
                m.course_ids.iter().any(|c| c == "FNCE 1010")
            }),
            "secondary CAS major must not also claim FNCE 1010 after college-wide reconcile"
        );

        let allocs = output
            .cross_degree_summary
            .as_ref()
            .and_then(|s| s.course_allocations.get("FNCE 1010"))
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            allocs.len(),
            1,
            "FNCE 1010 should allocate to exactly one degree, got {:?}",
            allocs
        );
        assert_eq!(allocs[0].degree_index, 0);
        assert_eq!(allocs[0].major, "CIS");
    }

    #[test]
    fn cas_double_major_unrestricted_computed_from_combined_major_cu() {
        use degree_planner::penn_data::college_data::{
            cas_college_auto_completed_sectors, cas_effective_combined_major_cu,
            cas_major_pool_major_cu, cas_open_gen_ed_slot_count, cas_shared_gened_flex_slots,
            cas_shared_unrestricted_elective_count, CAS_DEGREE_CU,
            CAS_UNRESTRICTED_ELECTIVES_CATEGORY,
        };

        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let econ = resolve_major("CAS", "ECON", &[]).expect("ECON");
        let cas_majors = vec![&neur, &econ];
        let combined = cas_effective_combined_major_cu(&cas_majors, 0);
        let open = cas_open_gen_ed_slot_count(&cas_college_auto_completed_sectors(&cas_majors));
        let shared_unrestricted = cas_shared_unrestricted_elective_count(combined, open);
        let gen_ed_flex = cas_shared_gened_flex_slots(combined, open);
        assert_eq!(
            1 + combined + gen_ed_flex + shared_unrestricted,
            CAS_DEGREE_CU,
            "shared CAS degree components should total {CAS_DEGREE_CU} CU"
        );

        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "CAS", "ECON"));
        assert!(output.error.is_none(), "{:?}", output.error);
        assert_eq!(output.degree_results.len(), 2);

        let primary = &output.degree_results[0];
        let secondary = &output.degree_results[1];
        let primary_unrestricted: Vec<_> = primary
            .unfulfilled_requirements
            .iter()
            .chain(primary.fulfilled_requirements.iter())
            .filter(|m| m.requirement.get_category() == CAS_UNRESTRICTED_ELECTIVES_CATEGORY)
            .collect();
        let secondary_unrestricted: Vec<_> = secondary
            .unfulfilled_requirements
            .iter()
            .chain(secondary.fulfilled_requirements.iter())
            .filter(|m| m.requirement.get_category() == CAS_UNRESTRICTED_ELECTIVES_CATEGORY)
            .collect();

        assert_eq!(
            primary_unrestricted.len() as i32,
            shared_unrestricted,
            "primary CAS degree should expose exactly {shared_unrestricted} unrestricted electives, got {}",
            primary_unrestricted.len()
        );
        assert!(
            secondary_unrestricted.is_empty(),
            "secondary CAS major should not list college-wide unrestricted electives"
        );

        let nominal = cas_major_pool_major_cu(&neur) + cas_major_pool_major_cu(&econ);
        assert!(
            nominal > combined || shared_unrestricted == 0,
            "combined major CU should not exceed nominal without overlap savings"
        );
    }

    #[test]
    fn cas_cas_dual_writ_appears_once_without_college_overlap_optimizer() {
        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "CAS", "ECON"));
        assert_eq!(writ_cu_units_on_schedule(&output), 1.0);
        if let Some(plan) = &output.overlap_plan {
            assert!(
                !overlap_plan_has_writ_opportunity(plan),
                "CAS double major must not pair Writing Seminar via overlap optimizer"
            );
            for opp in &plan.opportunities {
                for slot in &opp.slots {
                    assert!(
                        college_data::is_cas_major_overlap_slot_key(&slot.slot_key),
                        "CAS double major overlap must be major-only, got slot {:?}",
                        slot
                    );
                }
            }
        }
    }

    #[test]
    fn cas_cas_overlap_plan_excludes_writing_and_gen_ed() {
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let econ = resolve_major("CAS", "ECON", &[]).expect("ECON");
        let taken: HashSet<String> = HashSet::new();
        let schools = vec!["CAS".into(), "CAS".into()];
        let majors = vec!["NEUR".into(), "ECON".into()];
        let cu = catalog_cu_map();
        let per_degree = vec![
            validate_courses_for_degree(neur.requirements.clone(), &vec![], &cu),
            validate_courses_for_degree(econ.requirements.clone(), &vec![], &cu),
        ];
        let major_refs = vec![&neur, &econ];
        let state = CrossDegreeState::new(schools.clone(), majors.clone());
        let plan = compute_overlap_plan(
            &per_degree,
            &major_refs,
            &schools,
            &majors,
            &taken,
            &state,
            &cu,
            None,
        );
        assert!(
            !overlap_plan_has_writ_opportunity(&plan),
            "CAS double major should not WRIT-overlap; opportunities: {:?}",
            plan.opportunities
        );
        for opp in &plan.opportunities {
            for slot in &opp.slots {
                assert!(
                    slot.slot_key.starts_with("1:f"),
                    "expected major slot only, got {:?}",
                    slot
                );
            }
        }
    }

    #[test]
    fn cas_writing_fulfilled_by_taken_writ_course() {
        let cu_map = catalog_cu_map();
        let taken = vec!["WRIT 0100".to_string()];
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let validation = validate_courses_for_degree(neur.requirements.clone(), &taken, &cu_map);
        assert!(
            validation
                .fulfilled
                .iter()
                .any(|m| requirement_tree_has_writ_department(&m.requirement)),
            "CAS writing should be fulfilled before gen-ed pool absorbs WRIT courses"
        );
    }

    #[test]
    fn cas_writing_course_never_counts_elsewhere() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "ECON".into(),
                school: "CAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![FrozenCourse {
                course_id: "WRIT 0100".into(),
                year: 1,
                semester: "Fall".into(),
            }],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        let dr = &output.degree_results[0];
        let writ_rows: Vec<_> = dr
            .fulfilled_requirements
            .iter()
            .filter(|m| m.course_ids.iter().any(|c| c == "WRIT 0100"))
            .collect();
        assert_eq!(writ_rows.len(), 1, "WRIT should appear on exactly one fulfilled row");
        assert_eq!(
            writ_rows[0].instance_id.as_deref(),
            Some("0"),
            "WRIT must only fulfill Writing Seminar"
        );
        assert!(
            !dr.fulfilled_requirements.iter().any(|m| {
                m.instance_id.as_deref() != Some("0")
                    && m.course_ids.iter().any(|c| c == "WRIT 0100")
            }),
            "WRIT must not appear on gen-ed, major, or unrestricted rows"
        );
    }

    #[test]
    fn cas_sector_major_double_count_at_most_one() {
        use degree_planner::penn_data::attributes_data;
        use degree_planner::penn_data::college_data::{cas_pool_constraints, SECTOR_SOCIETY};
        use degree_planner::requirement::evaluate_cas_pool_constraints;
        use std::collections::HashSet;

        let cu = catalog_cu_map();
        let attrs = attributes_data::attributes();
        // Society auto-completed → AUHS / AUPW remain. Both courses carry those attrs.
        let constraints = cas_pool_constraints(&[SECTOR_SOCIETY.to_string()]);
        let major_only = vec!["ECON 0620".to_string(), "CHEM 1012".to_string()];
        let major_set: HashSet<String> = major_only.iter().cloned().collect();

        let evals = evaluate_cas_pool_constraints(
            &major_only,
            &major_only,
            &major_set,
            &constraints,
            &attrs,
            &cu,
        );
        let sector_hits: Vec<_> = evals
            .iter()
            .filter(|e| e.consumption_group.starts_with("cas:sector") && e.fulfilled)
            .collect();
        assert_eq!(
            sector_hits.len(),
            1,
            "at most one major course may double-count toward a sector; got {:?}",
            sector_hits
                .iter()
                .map(|e| (&e.label, &e.course_ids))
                .collect::<Vec<_>>()
        );

        // Treated as non-major → both sectors can be covered.
        let empty_majors = HashSet::new();
        let both = evaluate_cas_pool_constraints(
            &major_only,
            &major_only,
            &empty_majors,
            &constraints,
            &attrs,
            &cu,
        );
        let both_hits = both
            .iter()
            .filter(|e| e.consumption_group.starts_with("cas:sector") && e.fulfilled)
            .count();
        assert_eq!(
            both_hits, 2,
            "without major tagging, both sector courses should count"
        );
    }

    #[test]
    fn orphan_taken_course_absent_from_degree_allocations() {
        // EE has a single Free Elective. Two non-major courses → one fills it, one is orphan.
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "EE".into(),
                school: "SEAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![
                FrozenCourse {
                    course_id: "FNCE 1010".into(),
                    year: 1,
                    semester: "Fall".into(),
                },
                FrozenCourse {
                    course_id: "MKTG 1010".into(),
                    year: 1,
                    semester: "Spring".into(),
                },
            ],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        let dr = &output.degree_results[0];
        let claimed: std::collections::HashSet<&str> = dr
            .fulfilled_requirements
            .iter()
            .chain(dr.unfulfilled_requirements.iter().filter(|m| m.partial))
            .flat_map(|m| m.course_ids.iter().map(|c| c.as_str()))
            .collect();
        let orphans: Vec<_> = ["FNCE 1010", "MKTG 1010"]
            .into_iter()
            .filter(|c| !claimed.contains(*c))
            .collect();
        assert_eq!(
            orphans.len(),
            1,
            "exactly one of the two free courses should be unclaimed, got claimed={claimed:?}"
        );
        let orphan = orphans[0];
        let allocs = output
            .cross_degree_summary
            .as_ref()
            .and_then(|s| s.course_allocations.get(orphan))
            .cloned()
            .unwrap_or_default();
        assert!(
            allocs.is_empty(),
            "orphan {orphan} must not appear in course_allocations: {:?}",
            allocs
        );
    }

    #[test]
    fn taken_writ_fulfills_both_degrees_before_scheduling() {
        let cu_map = catalog_cu_map();
        let taken = vec!["WRIT 0100".to_string()];
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH");
        let mut per_degree = vec![
            validate_courses_for_degree(neur.requirements.clone(), &taken, &cu_map),
            validate_courses_for_degree(wh.requirements.clone(), &taken, &cu_map),
        ];
        let schools = vec!["CAS".into(), "WH".into()];
        let majors = vec!["NEUR".into(), "WH_NOFL".into()];
        resolve_cross_degree_conflicts(
            &mut per_degree,
            &schools,
            &majors,
            &cu_map,
            None,
            Some(&taken),
            None,
        );
        for (idx, validation) in per_degree.iter().enumerate() {
            let fulfilled = validation
                .fulfilled
                .iter()
                .any(|m| requirement_tree_has_writ_department(&m.requirement));
            let unfulfilled = validation
                .unfulfilled
                .iter()
                .any(|m| requirement_tree_has_writ_department(&m.requirement));
            assert!(fulfilled, "degree {idx} should fulfill WRIT when taken");
            assert!(!unfulfilled, "degree {idx} should not leave WRIT open when taken");
        }
    }

    #[test]
    fn taken_writ_course_suppresses_duplicate_writ_slots() {
        let output = generate_schedule(ScheduleInput {
            taken: vec!["WRIT 0100".into()],
            degrees: vec![
                DegreeInput {
                    major: "NEUR".into(),
                    school: "CAS".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL".into(),
                    school: "WH".into(),
                    kind: "major".to_string(),
                    concentrations: vec![],
                    concentration: Some("FNCE".into()),
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        let writ_units = writ_cu_units_on_schedule(&output);
        let open_writ_slots: Vec<_> = output
            .schedule
            .iter()
            .flat_map(|p| p.requirement_slots.iter())
            .filter(|s| {
                output
                    .slot_labels
                    .get(s.as_str())
                    .is_some_and(|l| is_writ_slot_label(l))
                    || output.overlap_schedule_groups.iter().any(|g| {
                        g.group_id == **s
                            && (is_writ_slot_label(&g.explanation)
                                || g.members.iter().any(|m| is_writ_slot_label(&m.label)))
                    })
            })
            .collect();
        assert!(
            open_writ_slots.is_empty(),
            "fulfilled WRIT should not leave open WRIT placeholders; got slots {:?}, writ_units={writ_units}",
            open_writ_slots
        );
    }

    #[test]
    fn all_implemented_dual_pairs_generate_valid_schedules() {
        for (label, input) in implemented_dual_undergrad_pairs() {
            let output = generate_schedule(input);
            assert!(
                output.error.is_none(),
                "{label}: pipeline error: {:?}",
                output.error
            );
            assert_eq!(output.degree_results.len(), 2, "{label}");
            assert_schedule_respects_cu_limits(&output, &label);
        }
    }

    #[test]
    fn dual_pairs_with_writ_on_both_degrees_schedule_writ_once() {
        for (label, input) in implemented_dual_undergrad_pairs() {
            let both_have_writ = input.degrees.iter().all(|d| degree_input_has_writ(d));
            if !both_have_writ {
                continue;
            }
            let all_cas = input.degrees.iter().all(|d| d.school == "CAS");
            let output = generate_schedule(input);
            assert!(
                output.error.is_none(),
                "{label}: pipeline error: {:?}",
                output.error
            );
            let writ_units = writ_cu_units_on_schedule(&output);
            assert_eq!(
                writ_units, 1.0,
                "{label}: expected exactly one WRIT CU on schedule, got {writ_units}"
            );
            if let Some(plan) = &output.overlap_plan {
                if all_cas {
                    assert!(
                        !overlap_plan_has_writ_opportunity(plan),
                        "{label}: CAS double major must not WRIT-overlap"
                    );
                } else {
                    assert!(
                        overlap_plan_has_writ_opportunity(plan),
                        "{label}: expected WRIT overlap opportunity"
                    );
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(12))]

        #[test]
        fn wh_variant_with_cas_major_schedules_single_writ(
            cas_major in prop_oneof!["NEUR", "ECON", "CIS"],
            wh_major in prop_oneof!["WH_NOFL", "WH_FL", "WH_NOFL_MT", "WH_FL_MT"],
        ) {
            let output = generate_schedule(dual_degree_input("CAS", &cas_major, "WH", &wh_major));
            prop_assume!(output.error.is_none());
            let writ_units = writ_cu_units_on_schedule(&output);
            prop_assert!(
                (writ_units - 1.0).abs() < CU_EPS,
                "expected 1 WRIT CU for {cas_major}+{wh_major}, got {writ_units}"
            );
            let plan = output.overlap_plan.as_ref().expect("overlap plan");
            prop_assert!(
                overlap_plan_has_writ_opportunity(plan),
                "expected WRIT overlap for {cas_major}+{wh_major}"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Semester templates — placement hints follow academic ordering
// ═══════════════════════════════════════════════════════════════════════════════

mod schedule_templates {
    use super::*;

    #[test]
    fn scheduled_requirements_receive_sequential_indices() {
        let (reqs, hints) = scheduled(vec![
            (
                Y1F,
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["CIS 1100".into()],
                },
            ),
            (
                Y1S,
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["CIS 1200".into()],
                },
            ),
        ]);
        assert_eq!(reqs.len(), 2);
        assert_eq!(
            hints.get("0").map(|h| (h.year, h.semester.as_str(), h.mode)),
            Some((1, "Fall", ScheduleHintMode::Flexible))
        );
        assert_eq!(
            hints.get("1").map(|h| (h.year, h.semester.as_str(), h.mode)),
            Some((1, "Spring", ScheduleHintMode::Flexible))
        );
    }

    #[test]
    fn fixed_hints_only_allow_exact_semester() {
        let hint = ScheduleHint::fixed(Y4F);
        let semesters = placement_semesters(&hint, 4);
        assert_eq!(semesters, vec![(4, "Fall".to_string())]);
    }

    #[test]
    fn flexible_hints_allow_backfill_before_target() {
        let hint = ScheduleHint::flexible(Y3F);
        let semesters = placement_semesters(&hint, 4);
        assert!(semesters.iter().any(|(y, s)| *y == 1 && s == "Fall"));
        assert!(semesters.iter().any(|(y, s)| *y == 3 && s == "Fall"));
        let first = semesters.first().unwrap();
        let last_target = semesters
            .iter()
            .find(|(y, s)| *y == 3 && s == "Fall")
            .unwrap();
        assert!(
            semester_order(first.0, &first.1) < semester_order(last_target.0, &last_target.1)
        );
    }

    fn assert_course_in_semester(
        output: &scheduler::ScheduleOutput,
        course: &str,
        year: i32,
        semester: &str,
    ) {
        let found = output.schedule.iter().any(|plan| {
            plan.year == year
                && plan.semester == semester
                && plan.courses.iter().any(|c| c == course)
        });
        assert!(
            found,
            "{course} should be scheduled in year {year} {semester}; got {:?}",
            output
                .schedule
                .iter()
                .map(|p| (p.year, p.semester.as_str(), p.courses.clone()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn seas_senior_design_courses_are_fixed_y4f_and_y4s() {
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "CIS".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert_course_in_semester(&output, "CIS 4000", 4, "Fall");
        assert_course_in_semester(&output, "CIS 4010", 4, "Spring");
    }

    #[test]
    fn ee_robotics_wh_places_fixed_courses_in_mandatory_semesters() {
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

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
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert_course_in_semester(&output, "WH 1010", 1, "Fall");
        assert_course_in_semester(&output, "OIDD 2340", 1, "Fall");
        assert_course_in_semester(&output, "MGMT 2370", 2, "Spring");
        assert_course_in_semester(&output, "ESE 4500", 4, "Fall");
        assert_course_in_semester(&output, "ESE 4510", 4, "Spring");
    }

    #[test]
    fn half_credit_wh_mt_named_courses_fulfill_their_slots() {
        let major = resolve_major("WH", "WH_NOFL_MT", &["STAT".into()]).expect("WH_NOFL_MT");
        let cu_map = catalog_cu_map();
        for (course, category) in [
            ("WH 1010", "Leadership Journey"),
            ("WH 2010", "Leadership Journey"),
            ("MGMT 3010", "Leadership Journey"),
            ("OIDD 2340", "M&T Freshman Course"),
        ] {
            assert!(
                (cu_map.get(course).copied().unwrap_or(0.0) - 0.5).abs() < 1e-6,
                "{course} should be 0.5 CU"
            );
            let result = validate_courses_for_degree(
                major.requirements.clone(),
                &vec![course.to_string()],
                cu_map,
            );
            assert!(
                result.fulfilled.iter().any(|m| {
                    m.requirement.get_category() == category && m.course_ids.iter().any(|c| c == course)
                }),
                "{course} should fulfill {category}; fulfilled={:?} unfulfilled={:?}",
                result
                    .fulfilled
                    .iter()
                    .map(|m| (m.requirement.get_category(), m.course_ids.clone()))
                    .collect::<Vec<_>>(),
                result
                    .unfulfilled
                    .iter()
                    .filter(|m| m.requirement.get_category() == category)
                    .map(|m| m.course_ids.clone())
                    .collect::<Vec<_>>(),
            );
        }
    }

    #[test]
    fn frozen_wh_mt_leadership_and_freshman_count_on_requirements() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "WH_NOFL_MT".into(),
                school: "WH".into(),
                kind: "major".to_string(),
                concentrations: vec!["STAT".into()],
                concentration: None,
            }],
            frozen: vec![
                FrozenCourse {
                    course_id: "WH 1010".into(),
                    year: 1,
                    semester: "Fall".into(),
                },
                FrozenCourse {
                    course_id: "WH 2010".into(),
                    year: 2,
                    semester: "Fall".into(),
                },
                FrozenCourse {
                    course_id: "MGMT 3010".into(),
                    year: 3,
                    semester: "Fall".into(),
                },
                FrozenCourse {
                    course_id: "OIDD 2340".into(),
                    year: 1,
                    semester: "Fall".into(),
                },
            ],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{:?}", output.error);
        let wh = output
            .degree_results
            .iter()
            .find(|r| r.major == "WH_NOFL_MT")
            .expect("WH_NOFL_MT result");
        let fulfilled_cats: Vec<(String, Vec<String>)> = wh
            .fulfilled_requirements
            .iter()
            .map(|m| (m.requirement.get_category(), m.course_ids.clone()))
            .collect();
        for (course, category) in [
            ("WH 1010", "Leadership Journey"),
            ("WH 2010", "Leadership Journey"),
            ("MGMT 3010", "Leadership Journey"),
            ("OIDD 2340", "M&T Freshman Course"),
        ] {
            assert!(
                wh.fulfilled_requirements.iter().any(|m| {
                    m.requirement.get_category() == category
                        && m.course_ids.iter().any(|c| c == course)
                }),
                "{course} should appear on {category} in the requirements panel; fulfilled={fulfilled_cats:?}"
            );
        }
        let leadership_filled = wh
            .fulfilled_requirements
            .iter()
            .filter(|m| m.requirement.get_category() == "Leadership Journey")
            .count();
        assert_eq!(
            leadership_filled, 3,
            "all three Leadership Journey slots should be fulfilled; {fulfilled_cats:?}"
        );
    }

    #[test]
    fn frozen_ese_4210_counts_toward_ee_robotics_tracker() {
        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "EE".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec!["Robotics".into()],
                concentration: None,
            }],
            frozen: vec![FrozenCourse {
                course_id: "ESE 4210".into(),
                year: 3,
                semester: "Spring".into(),
            }],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        let ee = output
            .degree_results
            .iter()
            .find(|r| r.major == "EE")
            .expect("EE result");
        let robotics = ee
            .concentration_info
            .iter()
            .find(|ci| ci.name == "Robotics")
            .expect("Robotics tracker");
        assert!(
            robotics.requirement_fulfilled.first().copied().unwrap_or(false),
            "frozen ESE 4210 should count toward Robotics; matched={:?}",
            robotics.matched_courses
        );
    }

    #[test]
    fn wh_nofl_places_wh1010_in_y1_fall() {
        use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "WH_NOFL".into(),
                school: "WH".into(),
                kind: "major".to_string(),
                concentrations: vec!["FNCE".into()],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert_course_in_semester(&output, "WH 1010", 1, "Fall");
    }

    #[test]
    fn wh_fl_mt_fixed_hints_include_wh1010_and_oidd2340() {
        use degree_planner::major::resolve_major;
        use degree_planner::schedule_template::ScheduleHintMode;

        let major = resolve_major("WH", "WH_FL_MT", &["FNCE".into()]).expect("WH_FL_MT");
        for (course, year, semester) in [
            ("WH 1010", 1, "Fall"),
            ("OIDD 2340", 1, "Fall"),
            ("MGMT 2370", 2, "Spring"),
        ] {
            let hint = major
                .schedule_hints
                .get(course)
                .unwrap_or_else(|| panic!("missing fixed hint for {course}"));
            assert_eq!(hint.mode, ScheduleHintMode::Fixed, "{course} should be rigid");
            assert_eq!(hint.year, year, "{course} year");
            assert_eq!(hint.semester, semester, "{course} semester");
        }
    }

    #[test]
    fn later_semesters_extends_beyond_four_years() {
        let seq = later_semesters((4, "Spring"), 6);
        assert!(seq.iter().any(|(y, s)| *y == 5 && s == "Fall"));
        assert!(seq.iter().any(|(y, s)| *y == 6 && s == "Spring"));
    }

    #[test]
    fn semester_order_is_monotonic_within_year() {
        assert!(semester_order(2, "Fall") < semester_order(2, "Spring"));
        assert!(semester_order(2, "Spring") < semester_order(2, "Summer"));
        assert!(semester_order(2, "Summer") < semester_order(3, "Fall"));
    }

    proptest! {
        #[test]
        fn later_semesters_never_go_backwards(
            year in 1i32..8,
            max_year in 1i32..10,
        ) {
            prop_assume!(max_year >= year);
            let mut prev = semester_order(year, "Fall");
            for (y, s) in later_semesters((year, "Spring"), max_year) {
                let ord = semester_order(y, &s);
                prop_assert!(ord >= prev, "semester sequence went backwards");
                prev = ord;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. Property-based invariants — rules that must hold for all inputs
// ═══════════════════════════════════════════════════════════════════════════════

mod property_invariants {
    use super::*;

    #[test]
    fn valid_catalog_courses_have_non_negative_cu() {
        for c in courses_data::courses() {
            if course::is_valid_course_code(&c.course_code) {
                assert!(
                    c.cu >= 0.0,
                    "{} has negative CU ({})",
                    c.course_code,
                    c.cu
                );
            }
        }
    }

    proptest! {
        #[test]
        fn cu_limit_never_below_one(
            schools in prop::collection::vec("(CAS|SEAS|WH)".prop_map(String::from), 1..3),
            year in 1i32..6,
            semester in prop_oneof!["Fall", "Spring", "Summer"],
        ) {
            let limit = default_semester_cu_limit(&schools, year, &semester);
            prop_assert!(limit >= 1.0);
            prop_assert!(limit <= 10.0);
        }

        #[test]
        fn dual_cas_never_gets_six_point_five(
            year in 2i32..5,
            semester in prop_oneof!["Fall", "Spring"],
        ) {
            let schools = vec!["CAS".into(), "CAS".into()];
            prop_assert_eq!(default_semester_cu_limit(&schools, year, &semester), 5.5);
        }
    }

    #[test]
    fn ee_chem_gen_ed_backfills_y4s_before_y5() {
        let output = generate_schedule(dual_degree_input("SEAS", "EE", "CAS", "CHEM"));
        assert!(output.error.is_none(), "{:?}", output.error);

        let gen_ed_in_y5: Vec<_> = output
            .schedule
            .iter()
            .filter(|p| p.year >= 5)
            .flat_map(|p| {
                p.requirement_slots
                    .iter()
                    .map(|slot| (p.year, p.semester.as_str(), slot.clone()))
            })
            .filter(|(_, _, slot)| {
                output
                    .slot_labels
                    .get(slot.as_str())
                    .is_some_and(|l| l == "General Education")
            })
            .collect();

        let y4s = output
            .schedule
            .iter()
            .find(|p| p.year == 4 && p.semester == "Spring")
            .expect("Y4 Spring");
        let limit = default_semester_cu_limit(
            &["SEAS".into(), "CAS".into()],
            4,
            "Spring",
        );
        let y4s_has_spare = y4s.total_cu + 1.0 <= limit + CU_EPS;

        assert!(
            gen_ed_in_y5.is_empty() || !y4s_has_spare,
            "gen-ed slots should backfill Y4 Spring before year 5 when space remains \
             (Y4S={:.1}/{:.1}, Y5 gen-eds={:?})",
            y4s.total_cu,
            limit,
            gen_ed_in_y5
        );
    }

    #[test]
    fn generated_dual_schedules_always_respect_cu_limits() {
        for (label, input) in implemented_dual_undergrad_pairs() {
            let output = generate_schedule(input);
            assert_schedule_respects_cu_limits(&output, &label);
        }
    }
}
