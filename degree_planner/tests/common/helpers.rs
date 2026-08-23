pub use std::collections::{HashMap, HashSet};

pub use proptest::prelude::*;

pub use degree_planner::course;
pub use degree_planner::cross_degree::{
    self, CrossDegreeState, CrossDegreeViolationKind, UNDERGRAD_GRAD_CU_LIMIT,
    cross_degree_optimizer_applicable, crosses_undergrad_grad, enforce_claim_rules,
    is_graduate_degree, overlap_plan_applicable,
};
pub use degree_planner::major::{self, resolve_major};
pub use degree_planner::overlap_planner::{
    self, OverlapSlotRef, compute_overlap_plan, is_overlap_schedule_group_id,
    overlap_group_schedule_id,
};
pub use degree_planner::penn_data::{attributes_data, college_data, courses_data};
pub use degree_planner::requirement::{
    self, Requirement, course_matches_restriction, evaluate_pool_constraints,
    expand_restriction_slots, extract_concentration_info, is_pool_constraint_slot_id,
    is_requirement_slot_id, is_schedulable_requirement_slot_id, requirement_accepts_shared_course,
    requirement_explicitly_lists_course, resolve_cross_degree_conflicts,
    validate_courses_for_degree,
};
pub use degree_planner::schedule_template::{
    ScheduleHint, ScheduleHintMode, Y1F, Y1S, Y2F, Y2S, Y3F, Y4F, later_semesters,
    placement_semesters, scheduled, semester_order,
};
pub use degree_planner::scheduler::{
    self, CU_EPS, DegreeInput, FrozenCourse, ScheduleInput, default_semester_cu_limit,
    generate_schedule, undergrad_schedule_years,
};

// ─── Shared helpers ───────────────────────────────────────────────────────────

pub fn catalog_cu_map() -> &'static HashMap<String, f64> {
    courses_data::cu_map()
}

pub fn sample_cu_map() -> HashMap<String, f64> {
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

pub fn one_cu_restriction() -> Requirement {
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

pub fn dual_degree_input(
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

pub fn dual_degree_input_with_conc(
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
pub fn course_has_stripe_mapping(output: &scheduler::ScheduleOutput, course: &str) -> bool {
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
            || result
                .concentration_info
                .iter()
                .any(|ci| !ci.is_core && ci.matched_courses.iter().flatten().any(|id| id == course))
    })
}

pub fn assert_scheduled_courses_have_stripe_mapping(
    output: &scheduler::ScheduleOutput,
    label: &str,
) {
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
pub fn requirement_tree_has_writ_department(req: &Requirement) -> bool {
    match req {
        Requirement::Restriction {
            department,
            number,
            cu,
            ..
        } => {
            department
                .as_ref()
                .is_some_and(|d| d.iter().any(|x| x == "WRIT"))
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
        Requirement::CoursePool { fixed_slots, .. } => {
            fixed_slots.iter().any(requirement_tree_has_writ_department)
        }
        _ => false,
    }
}

pub fn major_has_writ_requirement(major: &major::Major) -> bool {
    major
        .requirements
        .iter()
        .any(requirement_tree_has_writ_department)
}

pub fn degree_input_has_writ(degree: &DegreeInput) -> bool {
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

pub fn is_writ_slot_label(label: &str) -> bool {
    let lower = label.to_lowercase();
    lower.contains("writ") || lower.contains("writing sem") || lower.contains("writing seminar")
}

/// Count how many 1-CU WRIT units appear on the generated schedule grid.
pub fn writ_cu_units_on_schedule(output: &scheduler::ScheduleOutput) -> f64 {
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
                        || group.members.iter().any(|m| {
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
            let label = output
                .slot_labels
                .get(slot)
                .map(String::as_str)
                .unwrap_or("");
            if is_writ_slot_label(label) {
                units += 1.0;
            }
        }
    }

    units
}

pub fn overlap_plan_has_writ_opportunity(plan: &overlap_planner::OverlapPlan) -> bool {
    plan.opportunities.iter().any(|opp| {
        opp.slots.iter().any(|s| is_writ_slot_label(&s.label))
            || is_writ_slot_label(&opp.explanation)
            || opp.suggested_courses.iter().any(|c| c.starts_with("WRIT "))
    })
}

pub fn schedule_input(
    label: &str,
    school1: &str,
    major1: &str,
    school2: &str,
    major2: &str,
) -> (String, ScheduleInput) {
    (
        label.to_string(),
        dual_degree_input(school1, major1, school2, major2),
    )
}

pub fn implemented_dual_undergrad_pairs() -> Vec<(String, ScheduleInput)> {
    let cas_majors = ["NEUR", "ECON", "CIS"];
    let seas_majors = [
        ("SEAS", "CIS"),
        ("SEAS", "EE"),
        ("SEAS", "MSE"),
        ("SEAS", "AI"),
        ("SEAS", "CMPE"),
    ];
    let wh_majors = ["WH_NOFL", "WH_FL", "WH_NOFL_MT", "WH_FL_MT"];
    let mut pairs = Vec::new();

    for cas in cas_majors {
        for wh in &wh_majors {
            pairs.push(schedule_input(&format!("{cas}+{wh}"), "CAS", cas, "WH", wh));
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

pub fn assert_schedule_respects_cu_limits(output: &scheduler::ScheduleOutput, label: &str) {
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

pub fn occupied_schedule_max_year(output: &scheduler::ScheduleOutput) -> i32 {
    output
        .schedule
        .iter()
        .filter(|p| !p.courses.is_empty() || !p.requirement_slots.is_empty())
        .map(|p| p.year)
        .max()
        .unwrap_or(0)
}

pub fn assert_no_generic_anyof_grid_labels(output: &scheduler::ScheduleOutput, label: &str) {
    for plan in &output.schedule {
        for slot in &plan.requirement_slots {
            let text = output
                .slot_labels
                .get(slot)
                .map(String::as_str)
                .unwrap_or("");
            assert!(
                !text.eq_ignore_ascii_case("One of the following options"),
                "{label}: slot {slot} should use a category name, got {text:?}"
            );
        }
    }
}

pub fn assert_no_named_course_plus_option_placeholder(
    output: &scheduler::ScheduleOutput,
    label: &str,
) {
    let courses: HashSet<&str> = output
        .schedule
        .iter()
        .flat_map(|p| p.courses.iter().map(String::as_str))
        .collect();
    for plan in &output.schedule {
        for slot in &plan.requirement_slots {
            let Some(rest) = slot.strip_prefix("req:") else {
                continue;
            };
            let Some((_, fp)) = rest.split_once(":S:") else {
                continue;
            };
            let first = fp.split('/').next().unwrap_or("").replace('_', " ");
            assert!(
                !courses.contains(first.as_str()),
                "{label}: {first} is already a course card; leftover placeholder {slot}"
            );
        }
    }
}

pub fn assert_healthy_dual_degree_schedule(
    output: &scheduler::ScheduleOutput,
    label: &str,
    max_occupied_year: i32,
) {
    assert!(
        output.error.is_none(),
        "{label}: pipeline error: {:?}",
        output.error
    );
    assert_eq!(output.degree_results.len(), 2, "{label}");
    for result in &output.degree_results {
        assert!(
            result.error.is_none(),
            "{label}: {} {} error: {:?}",
            result.school,
            result.major,
            result.error
        );
    }
    assert_schedule_respects_cu_limits(output, label);
    let occupied = occupied_schedule_max_year(output);
    assert!(
        occupied <= max_occupied_year,
        "{label}: occupied max year {occupied} (limit {max_occupied_year}); total_cu={:.1}",
        output.schedule.iter().map(|p| p.total_cu).sum::<f64>()
    );
    let pairs = output
        .overlap_plan
        .as_ref()
        .map(|p| p.pairs.len())
        .unwrap_or(0);
    assert!(
        !output.overlap_schedule_groups.is_empty() || pairs > 0,
        "{label}: expected overlap pairs or schedule groups"
    );
    assert_no_generic_anyof_grid_labels(output, label);
    assert_no_named_course_plus_option_placeholder(output, label);
}
