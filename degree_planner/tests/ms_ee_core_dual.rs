use std::collections::HashSet;

use degree_planner::course;
use degree_planner::major::resolve_major;
use degree_planner::requirement::{self, Requirement};
use degree_planner::scheduler::{generate_schedule, DegreeInput, ScheduleInput};

const CORE: &str = "Electrical Engineering Core";

fn ms_ee_core_count(list: &[requirement::MappedRequirement]) -> usize {
    list.iter()
        .filter(|m| m.requirement.get_category() == CORE)
        .count()
}

#[test]
fn ms_ee_alone_has_five_core_slots() {
    let major = resolve_major("SEAS_MS", "MS_EE", &[]).expect("MS_EE");
    let core_in_major = major
        .requirements
        .iter()
        .filter(|r| r.get_category() == CORE)
        .count();
    assert_eq!(core_in_major, 5, "major definition should have 5 core singles");
}

#[test]
fn format_schedule_restriction_label_levels() {
    use degree_planner::requirement::format_schedule_restriction_description;

    assert_eq!(
        format_schedule_restriction_description(
            &Some(vec!["ESE".to_string()]),
            &None,
            &Some(5000),
            &None,
            &None,
            &1,
            &None,
        ),
        "ESE min level 5000"
    );
    assert_eq!(
        format_schedule_restriction_description(
            &None,
            &None,
            &None,
            &Some(3000),
            &Some(vec!["EUHS".to_string(), "EUSS".to_string()]),
            &1,
            &None,
        ),
        "EUHS/EUSS max level 3000"
    );
}

#[test]
fn format_schedule_single_course_label_truncates_after_two() {
    let label = requirement::format_schedule_single_course_label(&[
        "ESE 5090".into(),
        "ESE 5100".into(),
        "ESE 5130".into(),
        "ESE 5210".into(),
        "ESE 5230".into(),
    ]);
    assert_eq!(label, "ESE 5090/ESE 5100 (+3)");
}

#[test]
fn normalize_single_course_schedule_ids() {
    let multi = Requirement::SingleCourse {
        category: Some(CORE.to_string()),
        possibilities: vec!["ESE 5090".into(), "ESE 5100".into()],
    };
    let mut mapped = requirement::MappedRequirement {
        requirement: multi,
        course_ids: vec!["ESE 5090".into()],
        instance_id: Some("3".to_string()),
        attribute_fulfillment: None,
        partial: false,
        committed_anyof_branch: None,
    };
    requirement::normalize_suggested_schedule_ids(&mut mapped);
    assert_eq!(mapped.course_ids.len(), 1);
    assert!(requirement::is_requirement_slot_id(&mapped.course_ids[0]));
    assert!(mapped.course_ids[0].starts_with("req:3:S:"));

    let single = Requirement::SingleCourse {
        category: Some("Engineering".to_string()),
        possibilities: vec!["CIS 1100".into()],
    };
    let mut one = requirement::MappedRequirement {
        requirement: single,
        course_ids: vec!["req:placeholder".into()],
        instance_id: Some("0".to_string()),
        attribute_fulfillment: None,
        partial: false,
        committed_anyof_branch: None,
    };
    requirement::normalize_suggested_schedule_ids(&mut one);
    assert_eq!(one.course_ids, vec!["CIS 1100".to_string()]);
}

#[test]
fn ee_plus_ms_ee_schedules_five_distinct_ms_core_placeholders() {
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
    });
    assert!(output.error.is_none(), "{:?}", output.error);

    let ms = output
        .degree_results
        .iter()
        .find(|d| d.major == "MS_EE")
        .expect("MS_EE result");

    assert_eq!(ms_ee_core_count(&ms.suggested_for_unfulfilled), 5);

    let core_suggestions: Vec<_> = ms
        .suggested_for_unfulfilled
        .iter()
        .filter(|m| m.requirement.get_category() == CORE)
        .collect();

    for mapped in &core_suggestions {
        assert_eq!(mapped.course_ids.len(), 1);
        let id = &mapped.course_ids[0];
        assert!(
            requirement::is_requirement_slot_id(id),
            "multi-option core should schedule as dashed placeholder, got {id}"
        );
        assert!(
            !course::is_valid_course_code(id),
            "multi-option core should not reuse the same course id on the grid"
        );
    }

    let unique_ids: HashSet<_> = core_suggestions
        .iter()
        .map(|m| m.course_ids[0].clone())
        .collect();
    assert_eq!(
        unique_ids.len(),
        5,
        "each MS EE core slot needs a distinct schedule id"
    );

    let sample_label = output
        .slot_labels
        .get(core_suggestions[0].course_ids[0].as_str())
        .map(String::as_str)
        .unwrap_or("");
    assert!(
        sample_label.starts_with("ESE 5090/ESE 5100 (+"),
        "unexpected schedule label: {sample_label}"
    );
}
