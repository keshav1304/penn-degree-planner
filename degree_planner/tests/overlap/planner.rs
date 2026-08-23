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
    common::assert_overlap_plan_accuracy(
        &plan,
        &per_degree,
        &schools,
        &majors,
        &HashSet::new(),
        "NEUR+WH",
    );
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
    common::assert_overlap_plan_accuracy(
        &plan,
        &per_degree,
        &schools,
        &majors,
        &HashSet::new(),
        "NEUR+WH+ECON",
    );
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
        all_suggested
            .iter()
            .any(|c| *c == "ESE 3010" || *c == "STAT 4300"),
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
    common::assert_overlap_plan_accuracy(
        &plan,
        &per_degree,
        &schools,
        &majors,
        &HashSet::new(),
        "EE+WH_FL_MT",
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
    let reversed = overlap_group_schedule_id(&[slots[1].clone(), slots[0].clone()]);
    assert_eq!(id, reversed, "overlap group ids must be order-independent");
    assert_eq!(id, "req:overlap:0@2:c1+1@3:p0");
    assert!(id.starts_with("req:overlap:"));
    assert!(is_overlap_schedule_group_id(&id));
}

#[test]
fn ee_robotics_wh_nofl_mt_surfaces_key_overlaps_on_schedule() {
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
            o.explanation.contains("Humanities") || o.explanation.contains("Social Science")
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
        !group_explanations
            .iter()
            .any(|e| { e.contains("Fundamentals") && e.contains("Math and Natural Science") }),
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
    use degree_planner::scheduler::{DegreeInput, ScheduleInput, generate_schedule};

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

fn assert_shared_course_on_schedule(output: &scheduler::ScheduleOutput, course: &str, label: &str) {
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

fn option_list_placeholder_on_grid(output: &scheduler::ScheduleOutput, course: &str) -> bool {
    let slug = course.replace(' ', "_");
    output.schedule.iter().any(|sem| {
        sem.requirement_slots.iter().any(|slot| {
            slot.contains(&format!("S:{slug}"))
                || slot.contains(&format!("{slug}/"))
                || slot.contains(&format!("/{slug}"))
        })
    })
}

#[test]
fn biol_wh_nofl_math1400_is_one_shared_course_not_course_plus_placeholder() {
    let output = generate_schedule(dual_degree_input("CAS", "BIOL", "WH", "WH_NOFL"));
    assert_shared_course_on_schedule(&output, "MATH 1400", "BIOL+WH_NOFL");
    assert!(
        !option_list_placeholder_on_grid(&output, "MATH 1400"),
        "MATH 1400 already fills Biology allied science + Wharton math; the MATH 1400/MATH 1070 dashed block must not also sit on the grid. slots={:?}",
        output
            .schedule
            .iter()
            .flat_map(|p| p.requirement_slots.iter())
            .collect::<Vec<_>>()
    );
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
        per_degree[0].fulfilled.iter().any(|m| {
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
                possibilities: vec!["ESE 4000".into(), "MGMT 2370".into(), "OIDD 2360".into()],
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
    assert!(!requirement_explicitly_lists_course(
        &ee_restriction_only,
        "MGMT 2370"
    ));
    assert!(requirement_accepts_shared_course(
        &ee_restriction_only,
        "ESE 4000"
    ));
    assert!(!requirement_accepts_shared_course(
        &ee_restriction_only,
        "NOT A COURSE"
    ));
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
    common::assert_overlap_plan_accuracy(
        &plan,
        &per_degree,
        &schools,
        &majors,
        &HashSet::new(),
        "CIS+WH",
    );
}

#[test]
fn cis_wh_math1400_pairs_named_math_not_free_elective() {
    let cis = resolve_major("SEAS", "CIS", &[]).expect("CIS");
    let wh = resolve_major("WH", "WH_FL_MT", &["FNCE".into(), "STAT".into()]).expect("WH_FL_MT");
    let cu_map = catalog_cu_map();
    let empty: Vec<String> = vec![];
    let per_degree = vec![
        validate_courses_for_degree(cis.requirements.clone(), &empty, &cu_map),
        validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
    ];
    let schools = vec!["SEAS".into(), "WH".into()];
    let majors = vec!["CIS".into(), "WH_FL_MT".into()];
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

    let pair_labels = |pair: &degree_planner::overlap_planner::OverlapPair| -> Vec<String> {
        pair.slots.iter().map(|s| s.label.clone()).collect()
    };

    assert!(
        plan.pairs.iter().any(|pair| {
            let labels = pair_labels(pair);
            labels.iter().any(|l| l.contains("Math and Natural Science"))
                && labels.iter().any(|l| l.contains("First-Year Foundations"))
        }),
        "MATH 1400 should pair CIS math with WH foundations; pairs: {:?}",
        plan.pairs.iter().map(pair_labels).collect::<Vec<_>>()
    );
    assert!(
        plan.pairs.iter().all(|pair| {
            let labels = pair_labels(pair);
            !(labels.iter().any(|l| l == "Free Elective")
                && labels.iter().any(|l| l.contains("First-Year Foundations")))
        }),
        "MATH 1400 must not consume CIS Free Elective against WH foundations; pairs: {:?}",
        plan.pairs.iter().map(pair_labels).collect::<Vec<_>>()
    );
}

/// LAS flex seats (`:p`) have no filter of their own — the criteria live on the
/// pool's coverage constraints. While the constraints are unmet, seats must not
/// be offered as Unrestricted overlap targets (a CIS core can't really fill LAS).
#[test]
fn cis_wh_las_flex_seats_do_not_pair_while_coverage_unmet() {
    let cis = resolve_major("SEAS", "CIS", &[]).expect("CIS");
    let wh = resolve_major("WH", "WH_FL_MT", &["FNCE".into(), "STAT".into()]).expect("WH_FL_MT");
    let cu_map = catalog_cu_map();
    let empty: Vec<String> = vec![];
    let per_degree = vec![
        validate_courses_for_degree(cis.requirements.clone(), &empty, &cu_map),
        validate_courses_for_degree(wh.requirements.clone(), &empty, &cu_map),
    ];
    let schools = vec!["SEAS".into(), "WH".into()];
    let majors = vec!["CIS".into(), "WH_FL_MT".into()];
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

    let flex_las_pairs: Vec<Vec<String>> = plan
        .pairs
        .iter()
        .filter(|pair| {
            pair.slots.iter().any(|s| {
                s.slot_key.contains(":p") && s.label.contains("Liberal Arts and Sciences")
            })
        })
        .map(|pair| {
            pair.slots
                .iter()
                .map(|s| format!("{}[{}]:{}", s.major, s.slot_key, s.label))
                .collect()
        })
        .collect();
    assert!(
        flex_las_pairs.is_empty(),
        "no pair may target an LAS flex seat while LAS coverage is unmet: {flex_las_pairs:?}"
    );

    // The real LAS overlap (via attribute coverage constraints) must survive.
    let all_pairs: Vec<Vec<String>> = plan
        .pairs
        .iter()
        .map(|pair| {
            pair.slots
                .iter()
                .map(|s| format!("{}[{}]:{}", s.major, s.slot_key, s.label))
                .collect()
        })
        .collect();
    assert!(
        plan.pairs.iter().any(|pair| {
            pair.slots
                .iter()
                .any(|s| s.major == "WH_FL_MT" && s.slot_key.contains(":c"))
        }),
        "constraint-based WH pool overlap pairs should still exist; pairs: {all_pairs:?}"
    );
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
    let named_idx = plan.opportunities.iter().position(|o| {
        o.suggested_courses
            .iter()
            .any(|c| c == "ESE 3010" || c == "STAT 4300")
    });
    let catchall_idx = plan.opportunities.iter().position(|o| {
        o.slots.iter().any(|s| s.label == "Unrestricted Electives")
            && o.slots
                .iter()
                .any(|s| s.label.contains("General Electives"))
    });
    if let (Some(named), Some(catchall)) = (named_idx, catchall_idx) {
        assert!(
            named < catchall,
            "named stats overlap must rank above unrestricted×elective catch-alls ({named} vs {catchall})"
        );
    }
    common::assert_overlap_plan_accuracy(
        &plan,
        &per_degree,
        &schools,
        &majors,
        &HashSet::new(),
        "EE+WH",
    );
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
                && pair
                    .slots
                    .iter()
                    .map(|s| s.degree_index)
                    .collect::<HashSet<_>>()
                    .len()
                    == 2
        }),
        "expected a cross-degree WRIT pair; pairs: {:?}",
        plan.pairs
    );
    common::assert_overlap_plan_accuracy(
        &plan,
        &per_degree,
        &schools,
        &majors,
        &HashSet::new(),
        "CAS+WH WRIT",
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
            pair.slots
                .iter()
                .any(|s| s.label == "Unrestricted Electives")
                && pair
                    .slots
                    .iter()
                    .map(|s| s.degree_index)
                    .collect::<HashSet<_>>()
                    .len()
                    == 2
        }),
        "WH Unrestricted Electives should pair cross-degree; pairs: {:?}",
        plan.pairs
            .iter()
            .map(|p| p.slots.iter().map(|s| &s.label).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
    common::assert_overlap_plan_accuracy(
        &plan,
        &per_degree,
        &schools,
        &majors,
        &HashSet::new(),
        "NEUR+WH unrestricted",
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
        plan.opportunities.iter().any(|opp| overlaps_las(
            &opp.slots
                .iter()
                .map(|s| s.label.as_str())
                .collect::<Vec<_>>()
        )) || plan.pairs.iter().any(|pair| {
            overlaps_las(
                &pair
                    .slots
                    .iter()
                    .map(|s| s.label.as_str())
                    .collect::<Vec<_>>(),
            )
        }),
        "CAS gen-ed should overlap WH LAS pool or SSH constraints; pairs: {:?}",
        plan.pairs
            .iter()
            .map(|p| p.slots.iter().map(|s| &s.label).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
    common::assert_overlap_plan_accuracy(
        &plan,
        &per_degree,
        &schools,
        &majors,
        &HashSet::new(),
        "NEUR+WH gen-ed/LAS",
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

    let scheduled_slots: HashSet<&str> = output
        .schedule
        .iter()
        .flat_map(|p| p.requirement_slots.iter().map(String::as_str))
        .collect();
    for group in &output.overlap_schedule_groups {
        assert_eq!(group.members.len(), 2);
        assert!(scheduled_slots.contains(group.group_id.as_str()));
        for member in &group.members {
            assert!(
                !scheduled_slots.contains(member.schedule_slot_id.as_str()),
                "paired slot {} should be suppressed for {}",
                member.schedule_slot_id,
                group.group_id
            );
        }
    }
}

#[test]
fn neur_wh_named_science_overlaps_wh_wunm() {
    let output = generate_schedule(dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL"));
    assert!(output.error.is_none(), "{:?}", output.error);
    let summary = output
        .cross_degree_summary
        .as_ref()
        .expect("cross degree summary");
    let plan = output.overlap_plan.as_ref().expect("overlap plan");

    assert!(
        plan.pairs.iter().any(|pair| {
            let neur = pair.slots.iter().any(|s| {
                s.major == "NEUR"
                    && (s.label == "Introduction to Brain & Behavior"
                        || s.label == "Introductory Chemistry")
            });
            let wunm = pair
                .slots
                .iter()
                .any(|s| s.label.contains("Natural Science & Math"));
            neur && wunm
        }),
        "a named NEUR science SingleCourse should pair with WH WUNM; pairs: {:?}",
        plan.pairs
            .iter()
            .map(|p| p.slots.iter().map(|s| format!("{}:{}", s.major, s.label)).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );

    let shared = ["NRSC 1110", "CHEM 1011"].iter().any(|course| {
        summary
            .course_allocations
            .get(*course)
            .is_some_and(|allocs| {
                allocs.len() == 2
                    && allocs.iter().any(|a| a.major == "NEUR")
                    && allocs.iter().any(|a| a.major == "WH_NOFL")
            })
    });
    assert!(
        shared,
        "NRSC 1110 or CHEM 1011 should be shared onto WH WUNM; allocations: {:?}",
        summary.course_allocations
    );
}

