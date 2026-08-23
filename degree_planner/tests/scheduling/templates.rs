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
        hints
            .get("0")
            .map(|h| (h.year, h.semester.as_str(), h.mode)),
        Some((1, "Fall", ScheduleHintMode::Flexible))
    );
    assert_eq!(
        hints
            .get("1")
            .map(|h| (h.year, h.semester.as_str(), h.mode)),
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
    assert!(semester_order(first.0, &first.1) < semester_order(last_target.0, &last_target.1));
}

fn assert_course_in_semester(
    output: &scheduler::ScheduleOutput,
    course: &str,
    year: i32,
    semester: &str,
) {
    let found = output.schedule.iter().any(|plan| {
        plan.year == year && plan.semester == semester && plan.courses.iter().any(|c| c == course)
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
fn ee_robotics_wh_places_fixed_courses_in_mandatory_semesters() {
    use degree_planner::scheduler::{DegreeInput, ScheduleInput, generate_schedule};

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
                m.requirement.get_category() == category && m.course_ids.iter().any(|c| c == course)
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
        robotics
            .requirement_fulfilled
            .first()
            .copied()
            .unwrap_or(false),
        "frozen ESE 4210 should count toward Robotics; matched={:?}",
        robotics.matched_courses
    );
}

#[test]
fn wh_nofl_places_wh1010_in_y1_fall() {
    use degree_planner::scheduler::{DegreeInput, ScheduleInput, generate_schedule};

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
        assert_eq!(
            hint.mode,
            ScheduleHintMode::Fixed,
            "{course} should be rigid"
        );
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
