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
    self, crosses_undergrad_grad, enforce_claim_rules, is_graduate_degree,
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
    Requirement,
};
use crate::schedule_template::{
    later_semesters, scheduled, semester_order, Y1F, Y1S, Y2F,
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
        let info = build_cas_gen_ed_info(&pool, &cas_auto_completed_sectors_for("ECON"));

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

        let fundamentals_pair = plan.pairs.iter().any(|p| {
            p.explanation.contains("Fundamentals")
                && p.explanation.contains("Math and Natural Science")
        });
        assert!(
            fundamentals_pair,
            "expected a fundamentals ↔ math/science overlap pair; pairs: {:?}",
            plan.pairs
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
        assert_eq!(hints.get("0"), Some(&(1, "Fall".to_string())));
        assert_eq!(hints.get("1"), Some(&(1, "Spring".to_string())));
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
        for (label, input) in [
            ("NEUR+WH", dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL")),
            ("CIS+WH", dual_degree_input("SEAS", "CIS", "WH", "WH_NOFL")),
        ] {
            let output = generate_schedule(input);
            assert_schedule_respects_cu_limits(&output, label);
        }
    }
}
