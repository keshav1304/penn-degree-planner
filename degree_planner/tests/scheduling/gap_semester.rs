use degree_planner::scheduler::{
    DegreeInput, FrozenCourse, ScheduleInput, ScheduleOutput, SemesterPlan, generate_schedule,
};

fn cis_input(gap_semesters: Vec<&str>, frozen: Vec<FrozenCourse>) -> ScheduleInput {
    ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "CIS".into(),
            school: "SEAS".into(),
            kind: "major".to_string(),
            concentrations: vec![],
            concentration: None,
        }],
        frozen,
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: gap_semesters.into_iter().map(str::to_string).collect(),
        anon_session_id: None,
    }
}

fn term<'a>(output: &'a ScheduleOutput, year: i32, semester: &str) -> Option<&'a SemesterPlan> {
    output
        .schedule
        .iter()
        .find(|p| p.year == year && p.semester == semester)
}

fn term_item_count(plan: &SemesterPlan) -> usize {
    plan.courses.len() + plan.requirement_slots.len()
}

fn schedule_item_count(output: &ScheduleOutput) -> usize {
    output.schedule.iter().map(term_item_count).sum()
}

#[test]
fn gap_semester_is_not_auto_filled() {
    let output = generate_schedule(cis_input(vec!["2-Spring"], vec![]));
    assert!(output.error.is_none(), "{:?}", output.error);
    let plan = term(&output, 2, "Spring").expect("Y2 Spring exists");
    assert_eq!(
        term_item_count(plan),
        0,
        "gapped Y2 Spring should have no auto-placed courses or slots, got {:?}",
        plan
    );
}

#[test]
fn frozen_pin_stays_in_gap_semester() {
    let output = generate_schedule(cis_input(
        vec!["2-Spring"],
        vec![FrozenCourse {
            course_id: "CIS 1200".into(),
            year: 2,
            semester: "Spring".into(),
        }],
    ));
    assert!(output.error.is_none(), "{:?}", output.error);
    let plan = term(&output, 2, "Spring").expect("Y2 Spring exists");
    assert_eq!(plan.courses, vec!["CIS 1200".to_string()]);
    assert!(
        plan.requirement_slots.is_empty(),
        "gap should not auto-fill slots around a pin, got {:?}",
        plan.requirement_slots
    );
}

#[test]
fn enough_gaps_expand_past_year_four() {
    let output = generate_schedule(cis_input(
        vec![
            "1-Fall", "1-Spring", "2-Fall", "2-Spring", "3-Fall", "3-Spring",
        ],
        vec![],
    ));
    assert!(output.error.is_none(), "{:?}", output.error);
    assert!(
        output
            .schedule
            .iter()
            .any(|p| p.year >= 5 && term_item_count(p) > 0),
        "gapping years 1–3 should overflow into year 5+, schedule years: {:?}",
        output
            .schedule
            .iter()
            .map(|p| (p.year, p.semester.as_str(), term_item_count(p)))
            .collect::<Vec<_>>()
    );
}

#[test]
fn fixed_hint_does_not_dump_into_gap() {
    let baseline = generate_schedule(cis_input(vec![], vec![]));
    let gapped = generate_schedule(cis_input(vec!["4-Fall"], vec![]));
    assert!(gapped.error.is_none(), "{:?}", gapped.error);

    let y4f = term(&gapped, 4, "Fall").expect("Y4 Fall exists");
    assert_eq!(
        term_item_count(y4f),
        0,
        "gapped Y4 Fall must stay empty even for Fixed senior-design hints, got {:?}",
        y4f
    );

    if gapped
        .schedule
        .iter()
        .any(|p| p.courses.iter().any(|c| c == "CIS 4000"))
    {
        let placed = gapped
            .schedule
            .iter()
            .find(|p| p.courses.iter().any(|c| c == "CIS 4000"))
            .unwrap();
        assert!(
            !(placed.year == 4 && placed.semester == "Fall"),
            "CIS 4000 must not be forced into gapped Y4 Fall"
        );
    }

    assert_eq!(
        schedule_item_count(&baseline),
        schedule_item_count(&gapped),
        "gapping a term should relocate items, not drop them"
    );
}
