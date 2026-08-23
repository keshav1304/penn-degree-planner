use super::*;

#[test]
fn business_breadth_slot_label_matches_scoped_id() {
    use degree_planner::penn_data::wharton_data;
    let major = wharton_data::create_wh_nofl_major(vec!["FNCE".into()]);
    let validation =
        validate_courses_for_degree(major.requirements.clone(), &vec![], &catalog_cu_map());
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
    assert_ne!(
        label, "Open requirement",
        "slot_id={slot_id} instance={instance}"
    );
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
    use degree_planner::scheduler::{
        DegreeInput, ScheduleInput, dual_undergrad_only, generate_schedule,
    };

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
                    schools.contains("SEAS_MS") && schools.iter().any(|s| *s != "SEAS_MS")
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
    use degree_planner::scheduler::{DegreeInput, ScheduleInput, generate_schedule};

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
                && pair
                    .slots
                    .iter()
                    .any(|s| s.school == "SEAS" || s.school == "WH")
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
