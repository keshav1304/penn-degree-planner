//! Behavioral tests for also_offered_as equivalence and mutually_exclusive warnings.
//! Asserts public API outcomes without inspecting internal canonical/graph state.

use std::collections::HashSet;

use degree_planner::cross_degree::{CrossDegreeState, CrossDegreeViolationKind};
use degree_planner::major::resolve_major;
use degree_planner::penn_data::courses_data;
use degree_planner::requirement::{self, Requirement};
use degree_planner::scheduler::{DegreeInput, FrozenCourse, ScheduleInput, generate_schedule};

fn major_input(school: &str, major: &str) -> DegreeInput {
    DegreeInput {
        major: major.to_string(),
        school: school.to_string(),
        kind: "major".to_string(),
        concentrations: vec![],
        concentration: None,
    }
}

fn schedule_codes(output: &degree_planner::scheduler::ScheduleOutput) -> HashSet<String> {
    output
        .schedule
        .iter()
        .flat_map(|p| p.courses.iter().cloned())
        .collect()
}

fn mutex_violation_messages(output: &degree_planner::scheduler::ScheduleOutput) -> Vec<String> {
    output
        .cross_degree_summary
        .as_ref()
        .map(|s| {
            s.violations
                .iter()
                .filter(|v| v.kind == CrossDegreeViolationKind::MutuallyExclusive)
                .map(|v| v.message.clone())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn taken_alias_fulfills_listed_possibility() {
    // ACCT 2110 is also offered as BEPP 2110 — either spelling should fill a SingleCourse list.
    let req = Requirement::SingleCourse {
        category: Some("Test".into()),
        possibilities: vec!["BEPP 2110".into()],
    };
    let cu_map = courses_data::cu_map();
    let validation = requirement::validate_courses_for_degree(
        vec![req.clone()],
        &vec!["ACCT 2110".into()],
        cu_map,
    );
    assert!(
        validation.unfulfilled.is_empty(),
        "ACCT 2110 should fulfill BEPP 2110 listing: {:?}",
        validation.unfulfilled
    );
    assert!(
        validation.fulfilled.iter().any(|m| m
            .course_ids
            .iter()
            .any(|c| c == "BEPP 2110" || c == "ACCT 2110")),
        "fulfilled course_ids should record the match: {:?}",
        validation.fulfilled
    );

    let reverse = Requirement::SingleCourse {
        category: Some("Test".into()),
        possibilities: vec!["ACCT 2110".into()],
    };
    let validation_rev =
        requirement::validate_courses_for_degree(vec![reverse], &vec!["BEPP 2110".into()], cu_map);
    assert!(
        validation_rev.unfulfilled.is_empty(),
        "BEPP 2110 should fulfill ACCT 2110 listing"
    );
}

#[test]
fn alias_pair_in_taken_does_not_double_count_as_two_courses_for_ug_ms_budget() {
    // Claiming under two spellings of the same also-offered course must charge UG↔MS budget once.
    let schools = vec!["SEAS".into(), "SEAS_MS".into()];
    let mut state = CrossDegreeState::new(schools, vec!["CIS".into(), "MS_ROBO".into()]);
    let cu = courses_data::cu_map();

    state.register_claim("ACCT 2110", 0, cu);
    assert!(
        state.can_claim("BEPP 2110", 1, cu).is_ok(),
        "alias spelling should resolve to the same claim key"
    );
    state.register_claim("BEPP 2110", 1, cu);

    let expected_cu = cu
        .get("ACCT 2110")
        .or_else(|| cu.get("BEPP 2110"))
        .copied()
        .unwrap_or(1.0);
    assert!(
        (state.undergrad_grad_cu_used - expected_cu).abs() < 0.001,
        "alias pair must count once toward UG↔MS budget (used={}, expected={expected_cu})",
        state.undergrad_grad_cu_used
    );
    assert_eq!(
        state.claims.len(),
        1,
        "two spellings should collapse to one claim entry: {:?}",
        state.claims
    );

    // Schedule sanitize: both spellings in taken must not become two grid tiles.
    let output = generate_schedule(ScheduleInput {
        taken: vec!["ACCT 2110".into(), "BEPP 2110".into()],
        degrees: vec![major_input("SEAS", "CIS")],
        frozen: vec![
            FrozenCourse {
                course_id: "ACCT 2110".into(),
                year: 1,
                semester: "Fall".into(),
            },
            FrozenCourse {
                course_id: "BEPP 2110".into(),
                year: 1,
                semester: "Spring".into(),
            },
        ],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);
    let codes = schedule_codes(&output);
    assert!(
        !(codes.contains("ACCT 2110") && codes.contains("BEPP 2110")),
        "schedule must not list both aliases: {codes:?}"
    );
}

#[test]
fn mutex_warns_only_when_both_on_schedule() {
    let both = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![major_input("SEAS", "CIS")],
        frozen: vec![
            FrozenCourse {
                course_id: "CIS 4190".into(),
                year: 1,
                semester: "Fall".into(),
            },
            FrozenCourse {
                course_id: "CIS 5190".into(),
                year: 1,
                semester: "Spring".into(),
            },
        ],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    let codes = schedule_codes(&both);
    assert!(codes.contains("CIS 4190"), "{codes:?}");
    assert!(codes.contains("CIS 5190"), "{codes:?}");
    let msgs = mutex_violation_messages(&both);
    assert!(
        !msgs.is_empty(),
        "expected MutuallyExclusive when both on schedule"
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("CIS 4190") && m.contains("CIS 5190")),
        "message should name both courses: {msgs:?}"
    );
    // Not stripped
    assert!(codes.contains("CIS 4190") && codes.contains("CIS 5190"));

    let only_one = generate_schedule(ScheduleInput {
        taken: vec!["CIS 5190".into()], // taken but not placed on a semester grid row via frozen
        degrees: vec![major_input("SEAS", "CIS")],
        frozen: vec![FrozenCourse {
            course_id: "CIS 4190".into(),
            year: 1,
            semester: "Fall".into(),
        }],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    let only_codes = schedule_codes(&only_one);
    assert!(
        only_codes.contains("CIS 4190"),
        "4190 should be on schedule: {only_codes:?}"
    );
    // 5190 may appear if auto-placed; if it does, mutex would correctly warn.
    // Control: when 5190 is only in taken and NOT on schedule courses, no mutex.
    if !only_codes.contains("CIS 5190") {
        assert!(
            mutex_violation_messages(&only_one).is_empty(),
            "no mutex warning when partner is not on the schedule"
        );
    }
}

#[test]
fn also_offered_pair_in_taken_emits_same_course_warning() {
    let output = generate_schedule(ScheduleInput {
        taken: vec!["STAT 4760".into(), "MKTG 4760".into()],
        degrees: vec![major_input("SEAS", "CIS")],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);
    let msgs: Vec<_> = output
        .cross_degree_summary
        .as_ref()
        .map(|s| {
            s.violations
                .iter()
                .filter(|v| v.kind == CrossDegreeViolationKind::AlsoOfferedSameCourse)
                .map(|v| v.message.clone())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        msgs.iter().any(|m| m.contains("STAT 4760")
            && m.contains("MKTG 4760")
            && m.contains("same course")),
        "expected also-offered same-course warning: {msgs:?}"
    );
}

#[test]
fn missing_prereq_warns_without_adding_courses() {
    // Use Wharton so the CIS auto-placer does not fill CIS 1210/2620 as major courses.
    let output = generate_schedule(ScheduleInput {
        taken: vec!["CIS 3200".into()],
        degrees: vec![major_input("WH", "WH_NOFL")],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);
    let msgs: Vec<_> = output
        .cross_degree_summary
        .as_ref()
        .map(|s| {
            s.violations
                .iter()
                .filter(|v| v.kind == CrossDegreeViolationKind::MissingPrerequisite)
                .map(|v| v.message.clone())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        msgs.iter()
            .any(|m| m.contains("CIS 3200") && m.contains("CIS 1210") && m.contains("CIS 2620")),
        "expected missing-prereq warning: {msgs:?}"
    );
    let codes = schedule_codes(&output);
    assert!(
        !codes.contains("CIS 1210") && !codes.contains("CIS 2620"),
        "missing prereqs must not be auto-added for the warning: {codes:?}"
    );

    let satisfied = generate_schedule(ScheduleInput {
        taken: vec!["CIS 3200".into(), "CIS 1210".into(), "CIS 2620".into()],
        degrees: vec![major_input("WH", "WH_NOFL")],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    let sat_msgs: Vec<_> = satisfied
        .cross_degree_summary
        .as_ref()
        .map(|s| {
            s.violations
                .iter()
                .filter(|v| {
                    v.kind == CrossDegreeViolationKind::MissingPrerequisite
                        && v.message.contains("CIS 3200")
                })
                .map(|v| v.message.clone())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        sat_msgs.is_empty(),
        "CIS 3200 should not warn when prereqs are taken: {sat_msgs:?}"
    );
}

#[test]
fn suggest_avoids_mutex_partner_when_other_taken() {
    let output = generate_schedule(ScheduleInput {
        taken: vec!["CIS 4190".into()],
        degrees: vec![major_input("SEAS", "CIS")],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    for result in &output.degree_results {
        for mapped in &result.suggested_for_unfulfilled {
            assert!(
                !mapped.course_ids.iter().any(|c| c == "CIS 5190"),
                "should not suggest CIS 5190 when CIS 4190 is taken: {:?}",
                mapped.course_ids
            );
        }
    }
    if let Some(plan) = &output.overlap_plan {
        for opp in &plan.opportunities {
            assert!(
                !opp.suggested_courses.iter().any(|c| c == "CIS 5190"),
                "overlap should not suggest CIS 5190: {:?}",
                opp.suggested_courses
            );
        }
    }

    // Sanity: CIS major still resolves
    assert!(resolve_major("SEAS", "CIS", &[]).is_some());
}
