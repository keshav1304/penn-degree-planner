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

use crate::course;
use crate::cross_degree::{
    self, crosses_undergrad_grad, cross_degree_optimizer_applicable, enforce_claim_rules,
    is_graduate_degree,
    CrossDegreeState, CrossDegreeViolationKind, UNDERGRAD_GRAD_CU_LIMIT,
};
use crate::major::{self, resolve_major};
use crate::overlap_planner::{
    self, compute_overlap_plan, is_overlap_schedule_group_id, overlap_group_schedule_id,
    OverlapSlotRef,
};
use crate::penn_data::{attributes_data, college_data, courses_data};
use crate::requirement::{
    self, course_matches_restriction, evaluate_pool_constraints, expand_restriction_slots,
    extract_concentration_info, is_pool_constraint_slot_id, is_requirement_slot_id,
    is_schedulable_requirement_slot_id, validate_courses_for_degree,
    resolve_cross_degree_conflicts, requirement_accepts_shared_course,
    requirement_explicitly_lists_course,
    Requirement,
};
use crate::schedule_template::{
    later_semesters, placement_semesters, scheduled, semester_order, ScheduleHint,
    ScheduleHintMode, Y1F, Y1S, Y2F, Y2S, Y3F, Y4F,
};
use crate::scheduler::{
    self, default_semester_cu_limit, generate_schedule, undergrad_schedule_years, CU_EPS,
    DegreeInput, ScheduleInput,
};

// ─── Shared helpers ───────────────────────────────────────────────────────────

fn catalog_cu_map() -> HashMap<String, f64> {
    courses_data::all_courses()
        .iter()
        .map(|c| (c.course_code.clone(), c.cu))
        .collect()
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
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: major2.to_string(),
                school: school2.to_string(),
                concentrations: vec![],
                concentration: wh_conc,
            },
        ],
        frozen: vec![],
        allow_summer: Some(true),
        semester_cu_limits: None,
    }
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
    fn cas_lists_all_college_majors() {
        assert_eq!(college_data::CAS_DEGREE_CATALOG.len(), 56);
        assert!(college_data::cas_catalog_entry("BIOL").is_some());
        assert!(college_data::cas_catalog_entry("NOT_A_MAJOR").is_none());
    }

    #[test]
    fn econ_gen_ed_marks_society_sector_completed_by_major() {
        use crate::penn_data::college_data::{build_cas_gen_ed_info, cas_auto_completed_sectors_for, create_econ_major, SECTOR_SOCIETY};

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
        use crate::penn_data::college_data::{
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
        use crate::penn_data::college_data::{cas_auto_completed_sectors_for, SECTOR_HUM_SOC_SCI};

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
        let biol = resolve_major("CAS", "BIOL", &[]).expect("BIOL");
        assert_eq!(biol.short_name, "BIOL");
        assert!(biol.concentrations.is_none());
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
            &Some(vec!["AIRE".into()]),
            &Some(vec!["AUFS".into()]),
            &None,
            &attributes,
        ));
        assert!(course_matches_restriction(
            "BEPP 2010",
            &None,
            &None,
            &Some(vec!["AIRE".into()]),
            &Some(vec!["AUFS".into()]),
            &None,
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
        use crate::penn_data::wharton_data::create_wh_fl_mt_major;

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
        use crate::penn_data::wharton_data::create_wh_nofl_mt_major;

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
            .find(|c| c.name.contains("FNCE") || c.name.contains("Finance"))
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
    fn overlap_optimizer_skipped_for_undergrad_plus_grad() {
        use crate::overlap_planner::is_overlap_schedule_group_id;
        use crate::scheduler::{dual_undergrad_only, generate_schedule, DegreeInput, ScheduleInput};

        let schools = vec!["SEAS".into(), "SEAS_MS".into()];
        assert!(!dual_undergrad_only(&schools));
        assert!(!cross_degree_optimizer_applicable(&schools));

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "MS_EE".into(),
                    school: "SEAS_MS".into(),
                    concentrations: vec![],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
        });
        assert!(output.error.is_none());
        assert!(output.overlap_plan.is_none());
        assert!(output.overlap_schedule_groups.is_empty());
        assert!(output.cross_degree_summary.is_some());
        let has_overlap_blocks = output.schedule.iter().any(|plan| {
            plan.requirement_slots
                .iter()
                .any(|slot| is_overlap_schedule_group_id(slot))
        });
        assert!(
            !has_overlap_blocks,
            "grad mixes must not produce paired overlap requirement blocks"
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
        use crate::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    concentrations: vec!["Robotics".into()],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL_MT".into(),
                    school: "WH".into(),
                    concentrations: vec!["FNCE".into()],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
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
                (e.contains("Humanities") || e.contains("Social Science"))
                    && e.contains("General Electives")
            }),
            "schedule should group humanities/social science overlap; groups: {:?}",
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
        use crate::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL_MT".into(),
                    school: "WH".into(),
                    concentrations: vec!["FNCE".into()],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
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
        use crate::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    concentrations: vec!["Robotics".into()],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL_MT".into(),
                    school: "WH".into(),
                    concentrations: vec!["FNCE".into()],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
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
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL_MT".into(),
                    school: "WH".into(),
                    concentrations: vec!["FNCE".into()],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
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
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Scheduling & CU policy — generated plans respect workload limits
// ═══════════════════════════════════════════════════════════════════════════════

mod scheduling {
    use super::*;

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
    fn undergrad_plus_grad_stays_four_years_not_dual_undergrad_optimizer() {
        let schools = vec!["SEAS".into(), "SEAS_MS".into()];
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
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
        });
        assert!(output.cross_degree_summary.is_none());
        assert_schedule_respects_cu_limits(&output, "single CIS");
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
                            .is_some_and(|l| l.contains("1 CU from General Education"))
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
                        .is_some_and(|l| l.contains("1 CU from General Education"))
                    {
                        gen_ed_overlap += 1;
                    }
                } else if output
                    .slot_labels
                    .get(slot)
                    .is_some_and(|l| l == "1 CU from General Education")
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
        assert_eq!(
            gen_ed_slots.len(),
            12,
            "expected 12 shared gen-ed flex slots on schedule, got {}: {:?}",
            gen_ed_slots.len(),
            gen_ed_slots
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
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL".into(),
                    school: "WH".into(),
                    concentrations: vec![],
                    concentration: Some("FNCE".into()),
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
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
        use crate::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: "CIS".into(),
                school: "SEAS".into(),
                concentrations: vec![],
                concentration: None,
            }],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
        });
        assert_course_in_semester(&output, "CIS 4000", 4, "Fall");
        assert_course_in_semester(&output, "CIS 4010", 4, "Spring");
    }

    #[test]
    fn ee_robotics_wh_places_fixed_courses_in_mandatory_semesters() {
        use crate::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: "EE".into(),
                    school: "SEAS".into(),
                    concentrations: vec!["Robotics".into()],
                    concentration: None,
                },
                DegreeInput {
                    major: "WH_NOFL_MT".into(),
                    school: "WH".into(),
                    concentrations: vec!["FNCE".into()],
                    concentration: None,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
        });
        assert_course_in_semester(&output, "MGMT 2370", 2, "Spring");
        assert_course_in_semester(&output, "ESE 4500", 4, "Fall");
        assert_course_in_semester(&output, "ESE 4510", 4, "Spring");
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
        for c in courses_data::all_courses() {
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
    fn generated_dual_schedules_always_respect_cu_limits() {
        for (label, input) in implemented_dual_undergrad_pairs() {
            let output = generate_schedule(input);
            assert_schedule_respects_cu_limits(&output, &label);
        }
    }
}

