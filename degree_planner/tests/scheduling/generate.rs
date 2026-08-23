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
    assert_eq!(
        default_semester_cu_limit(&vec!["SEAS".into()], 2, "Summer"),
        2.0
    );
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

fn scheduled_slot_ids(output: &scheduler::ScheduleOutput) -> HashSet<&str> {
    output
        .schedule
        .iter()
        .flat_map(|p| p.requirement_slots.iter().map(String::as_str))
        .collect()
}

#[test]
fn biol_wh_nofl_finishes_in_four_years_without_duplicate_placeholders() {
    let output = generate_schedule(dual_degree_input("CAS", "BIOL", "WH", "WH_NOFL"));
    assert_dual_schedule(&output, "BIOL + WH_NOFL");
    assert!(
        occupied_schedule_max_year(&output) <= 4,
        "BIOL+WH_NOFL should finish in 4 years with summer; occupied max year={}, total_cu={:.1}, schedule={:?}",
        occupied_schedule_max_year(&output),
        output.schedule.iter().map(|p| p.total_cu).sum::<f64>(),
        output
            .schedule
            .iter()
            .filter(|p| !p.courses.is_empty() || !p.requirement_slots.is_empty())
            .map(|p| format!(
                "Y{} {} cu={:.1} courses={:?} slots={:?}",
                p.year, p.semester, p.total_cu, p.courses, p.requirement_slots
            ))
            .collect::<Vec<_>>()
    );

    let courses: HashSet<&str> = output
        .schedule
        .iter()
        .flat_map(|p| p.courses.iter().map(String::as_str))
        .collect();
    let on_grid = scheduled_slot_ids(&output);
    for slot in &on_grid {
        if let Some(rest) = slot.strip_prefix("req:") {
            if let Some((_, fp)) = rest.split_once(":S:") {
                let first = fp.split('/').next().unwrap_or("").replace('_', " ");
                assert!(
                    !courses.contains(first.as_str()),
                    "BIOL+WH_NOFL: {first} is already a course card; leftover placeholder {slot}"
                );
            }
        }
        let label = output
            .slot_labels
            .get(*slot)
            .map(String::as_str)
            .unwrap_or("");
        assert!(
            !label.eq_ignore_ascii_case("One of the following options"),
            "BIOL+WH_NOFL: schedule slot {slot} should use the category name, got {label:?}"
        );
    }

    let additional_biology_slots: Vec<_> = on_grid
        .iter()
        .copied()
        .filter(|id| {
            id.contains("Additional_Biology")
                || output
                    .slot_labels
                    .get(*id)
                    .is_some_and(|l| l.contains("Additional Biology"))
        })
        .collect();
    assert!(
        !additional_biology_slots.is_empty(),
        "BIOL+WH_NOFL should still schedule Additional Biology slots"
    );
    for slot in additional_biology_slots {
        let label = output
            .slot_labels
            .get(slot)
            .map(String::as_str)
            .unwrap_or("");
        assert!(
            label.contains("Additional Biology"),
            "BIOL+WH_NOFL: Additional Biology slot {slot} should be labeled with the category, got {label:?}"
        );
        assert!(
            !label.contains("BIOL 2000"),
            "BIOL+WH_NOFL: Additional Biology slot {slot} should not expand to the child restriction, got {label:?}"
        );
    }
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
    assert!(
        shared_stats,
        "EE + WH_FL_MT schedule should surface shared stats overlap"
    );
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
    assert!(
        output.overlap_plan.is_none(),
        "single-degree generate must skip overlap discovery"
    );
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
