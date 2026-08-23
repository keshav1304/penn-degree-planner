use super::*;

#[test]
fn cas_and_wh_majors_both_include_writ_requirement() {
    let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
    let wh = resolve_major("WH", "WH_NOFL", &["FNCE".into()]).expect("WH_NOFL");
    assert!(major_has_writ_requirement(&neur), "CAS majors include WRIT");
    assert!(
        major_has_writ_requirement(&wh),
        "Wharton majors include WRIT"
    );
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
                        .is_some_and(|l| l.contains("General Education"))
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
            if member.label.contains("Foundational Approaches")
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
                    .is_some_and(|l| l.contains("General Education"))
                {
                    gen_ed_overlap += 1;
                }
            } else if output
                .slot_labels
                .get(slot)
                .is_some_and(|l| l == "General Education")
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
fn cas_single_econ_gen_ed_capped_at_twelve_with_unrestricted_electives() {
    use degree_planner::penn_data::college_data::{
        CAS_DEGREE_CU, CAS_UNRESTRICTED_ELECTIVES_CATEGORY, cas_auto_completed_sectors_for,
        cas_major_pool_major_cu, cas_open_gen_ed_slot_count, cas_shared_gened_flex_slots,
        cas_shared_unrestricted_elective_count, create_econ_major,
    };
    use degree_planner::scheduler::{DegreeInput, ScheduleInput, generate_schedule};

    let major = create_econ_major();
    let major_cu = cas_major_pool_major_cu(&major);
    let autos = cas_auto_completed_sectors_for("ECON", None);
    let open = cas_open_gen_ed_slot_count(&autos);
    let gen_ed_flex = cas_shared_gened_flex_slots(major_cu, open);
    let unrestricted = cas_shared_unrestricted_elective_count(major_cu, open);

    assert!(
        gen_ed_flex <= 12,
        "gen-ed flex should cap at 12, got {gen_ed_flex}"
    );
    assert!(
        unrestricted > 0,
        "expected unrestricted electives beyond gen-ed slots"
    );
    assert_eq!(
        1 + major_cu + gen_ed_flex + unrestricted,
        CAS_DEGREE_CU,
        "CAS degree should total 36 CU"
    );

    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "ECON".into(),
            school: "CAS".into(),
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

    let gen_ed_slots: usize = output
        .schedule
        .iter()
        .flat_map(|p| p.requirement_slots.iter())
        .filter(|s| {
            output
                .slot_labels
                .get(s.as_str())
                .is_some_and(|l| l == "General Education")
        })
        .count();
    assert!(
        gen_ed_slots <= 12,
        "single CAS ECON should schedule at most 12 gen-ed slots, got {gen_ed_slots}"
    );
    assert_eq!(
        gen_ed_slots as i32, gen_ed_flex,
        "scheduled gen-ed flex should match open coverage needs"
    );

    let unrest_rows = output.degree_results[0]
        .unfulfilled_requirements
        .iter()
        .chain(output.degree_results[0].fulfilled_requirements.iter())
        .filter(|m| m.requirement.get_category() == CAS_UNRESTRICTED_ELECTIVES_CATEGORY)
        .count();
    assert_eq!(unrest_rows as i32, unrestricted);
}

#[test]
fn cas_anch_non_gened_courses_allocate_to_unrestricted_not_pool_flex() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "ANCH".into(),
            school: "CAS".into(),
            kind: "major".to_string(),
            concentrations: vec![],
            concentration: None,
        }],
        frozen: vec![
            FrozenCourse {
                course_id: "AFRC 1100".into(),
                year: 1,
                semester: "Fall".into(),
            },
            FrozenCourse {
                course_id: "AFRC 0527".into(),
                year: 1,
                semester: "Spring".into(),
            },
        ],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);

    let dr = &output.degree_results[0];
    let gen_ed_flex: Vec<_> = dr
        .fulfilled_requirements
        .iter()
        .filter(|m| {
            m.instance_id
                .as_deref()
                .is_some_and(|id| id.starts_with("1:p"))
        })
        .collect();
    assert!(
        gen_ed_flex.is_empty(),
        "non-gen-ed courses should not fill pool flex slots: {:?}",
        gen_ed_flex
            .iter()
            .map(|m| (&m.instance_id, &m.course_ids))
            .collect::<Vec<_>>()
    );

    let unrestricted: Vec<_> = dr
        .fulfilled_requirements
        .iter()
        .filter(|m| {
            m.requirement.get_category() == "Unrestricted Electives"
                && m.course_ids.iter().any(|c| c.starts_with("AFRC"))
        })
        .collect();
    assert_eq!(
        unrestricted.len(),
        2,
        "expected both AFRC courses as unrestricted electives, got {:?}",
        unrestricted
            .iter()
            .map(|m| (&m.instance_id, &m.course_ids))
            .collect::<Vec<_>>()
    );
}

#[test]
fn cas_anch_redundant_gened_course_goes_unrestricted_after_sector_covered() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "ANCH".into(),
            school: "CAS".into(),
            kind: "major".to_string(),
            concentrations: vec![],
            concentration: None,
        }],
        frozen: vec![
            FrozenCourse {
                course_id: "ANTH 0040".into(),
                year: 1,
                semester: "Fall".into(),
            },
            FrozenCourse {
                course_id: "ANTH 1238".into(),
                year: 1,
                semester: "Spring".into(),
            },
        ],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);

    let dr = &output.degree_results[0];
    let pool_flex_with_anth: Vec<_> = dr
        .fulfilled_requirements
        .iter()
        .filter(|m| {
            m.instance_id
                .as_deref()
                .is_some_and(|id| id.starts_with("1:p"))
                && m.course_ids.iter().any(|c| c == "ANTH 0040")
        })
        .collect();
    assert_eq!(
        pool_flex_with_anth.len(),
        1,
        "ANTH 0040 should fill one unmet gen-ed constraint in the pool"
    );

    let anth_in_pool: Vec<_> = dr
        .fulfilled_requirements
        .iter()
        .filter(|m| {
            m.instance_id
                .as_deref()
                .is_some_and(|id| id.starts_with("1:p"))
                && m.course_ids.iter().any(|c| c == "ANTH 1238")
        })
        .collect();
    assert!(
        anth_in_pool.is_empty(),
        "second Hum/Soc Sci course should not enter pool when that sector is already covered: {:?}",
        anth_in_pool
    );

    let anth_unrestricted = dr.fulfilled_requirements.iter().any(|m| {
        m.requirement.get_category() == "Unrestricted Electives"
            && m.course_ids.iter().any(|c| c == "ANTH 1238")
    });
    assert!(
        anth_unrestricted,
        "redundant sector course should count as unrestricted elective"
    );
}

#[test]
fn seas_ee_cas_econ_gen_ed_slots_capped_at_twelve() {
    let output = generate_schedule(dual_degree_input("SEAS", "EE", "CAS", "ECON"));
    assert!(output.error.is_none(), "{:?}", output.error);

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
                .is_some_and(|l| l == "General Education")
        })
        .collect();

    let gen_ed_overlaps = output
        .overlap_schedule_groups
        .iter()
        .filter(|g| {
            output
                .slot_labels
                .get(&g.group_id)
                .is_some_and(|l| l.contains("General Education"))
        })
        .count();

    assert!(
        gen_ed_slots.len() + gen_ed_overlaps <= 12,
        "expected at most 12 CAS gen-ed schedule items for SEAS+ECON dual degree, got flex={} overlap={}: flex={:?}",
        gen_ed_slots.len(),
        gen_ed_overlaps,
        gen_ed_slots
    );
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
    let expected_flex = {
        use degree_planner::penn_data::college_data::{
            cas_college_auto_completed_sectors, cas_effective_combined_major_cu,
            cas_open_gen_ed_slot_count, cas_shared_gened_flex_slots,
        };
        let econ = resolve_major("CAS", "ECON", &[]).expect("ECON");
        let cis = resolve_major("CAS", "CIS", &[]).expect("CIS");
        let majors = vec![&econ, &cis];
        let combined = cas_effective_combined_major_cu(&majors, 0);
        let open = cas_open_gen_ed_slot_count(&cas_college_auto_completed_sectors(&majors));
        cas_shared_gened_flex_slots(combined, open) as usize
    };
    assert_eq!(
        gen_ed_slots.len(),
        expected_flex,
        "expected {expected_flex} shared gen-ed flex slots on schedule, got {}: {:?}",
        gen_ed_slots.len(),
        gen_ed_slots
    );
}

#[test]
fn cas_double_major_major_course_not_absorbed_as_unrestricted() {
    // CIS listed first → primary for college-wide residual slots. SOCI 1000 fills Society
    // gen-ed; ECON 0200 must stay on the ECON major and must NOT also fill unrestricted.
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: "CIS".into(),
                school: "CAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "ECON".into(),
                school: "CAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
        ],
        frozen: vec![
            FrozenCourse {
                course_id: "SOCI 1000".into(),
                year: 1,
                semester: "Fall".into(),
            },
            FrozenCourse {
                course_id: "ECON 0200".into(),
                year: 1,
                semester: "Spring".into(),
            },
        ],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);

    let econ = output
        .degree_results
        .iter()
        .find(|r| r.major == "ECON")
        .expect("ECON");
    assert!(
        econ.fulfilled_requirements.iter().any(|m| {
            m.requirement.get_category() == "Introductory Economics"
                && m.course_ids.iter().any(|c| c == "ECON 0200")
        }),
        "ECON 0200 should fulfill Introductory Economics"
    );

    for dr in &output.degree_results {
        let unrestricted_econ: Vec<_> = dr
            .fulfilled_requirements
            .iter()
            .filter(|m| {
                m.requirement.get_category() == "Unrestricted Electives"
                    && m.course_ids.iter().any(|c| c == "ECON 0200")
            })
            .collect();
        assert!(
            unrestricted_econ.is_empty(),
            "{} {} must not place ECON 0200 in Unrestricted Electives: {:?}",
            dr.school,
            dr.major,
            unrestricted_econ
                .iter()
                .map(|m| &m.course_ids)
                .collect::<Vec<_>>()
        );

        let flex_econ = dr.fulfilled_requirements.iter().any(|m| {
            m.instance_id.as_deref().is_some_and(|id| id.contains(":p"))
                && m.course_ids.iter().any(|c| c == "ECON 0200")
        });
        assert!(
            !flex_econ,
            "{} {} must not absorb ECON 0200 into gen-ed flex when it is an ECON major course",
            dr.school, dr.major
        );
    }
}

#[test]
fn cas_cis_no_within_major_double_count_of_core_as_elective() {
    // CIS 1600 fulfills Core; the CIS Elective restriction must not reclaim it.
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: "CIS".into(),
                school: "CAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "ECON".into(),
                school: "CAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
        ],
        frozen: vec![FrozenCourse {
            course_id: "CIS 1600".into(),
            year: 1,
            semester: "Fall".into(),
        }],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);

    let cis = output
        .degree_results
        .iter()
        .find(|r| r.major == "CIS")
        .expect("CIS");
    let cis1600_rows: Vec<_> = cis
        .fulfilled_requirements
        .iter()
        .filter(|m| m.course_ids.iter().any(|c| c == "CIS 1600"))
        .collect();
    assert_eq!(
        cis1600_rows.len(),
        1,
        "CIS 1600 must fulfill exactly one CIS major slot, got {:?}",
        cis1600_rows
            .iter()
            .map(|m| (m.requirement.get_category(), &m.instance_id, &m.course_ids))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        cis1600_rows[0].requirement.get_category(),
        "Core Courses",
        "CIS 1600 should count as Core, not CIS Elective"
    );
    assert!(
        !cis.fulfilled_requirements.iter().any(|m| {
            m.requirement.get_category() == "CIS Elective"
                && m.course_ids.iter().any(|c| c == "CIS 1600")
        }),
        "CIS 1600 must not also fulfill CIS Elective"
    );
}

#[test]
fn cas_double_major_fnce_unrestricted_primary_only() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: "CIS".into(),
                school: "CAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "ECON".into(),
                school: "CAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
        ],
        frozen: vec![FrozenCourse {
            course_id: "FNCE 1010".into(),
            year: 2,
            semester: "Fall".into(),
        }],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);

    let primary = &output.degree_results[0];
    assert_eq!(primary.major, "CIS");
    assert!(
        primary.fulfilled_requirements.iter().any(|m| {
            m.requirement.get_category() == "Unrestricted Electives"
                && m.course_ids.iter().any(|c| c == "FNCE 1010")
        }),
        "FNCE 1010 should fulfill primary Unrestricted Electives"
    );

    let secondary = &output.degree_results[1];
    assert!(
        !secondary
            .fulfilled_requirements
            .iter()
            .any(|m| { m.course_ids.iter().any(|c| c == "FNCE 1010") }),
        "secondary CAS major must not also claim FNCE 1010 after college-wide reconcile"
    );

    let allocs = output
        .cross_degree_summary
        .as_ref()
        .and_then(|s| s.course_allocations.get("FNCE 1010"))
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        allocs.len(),
        1,
        "FNCE 1010 should allocate to exactly one degree, got {:?}",
        allocs
    );
    assert_eq!(allocs[0].degree_index, 0);
    assert_eq!(allocs[0].major, "CIS");
}

#[test]
fn cas_double_major_unrestricted_computed_from_combined_major_cu() {
    use degree_planner::penn_data::college_data::{
        CAS_DEGREE_CU, CAS_UNRESTRICTED_ELECTIVES_CATEGORY, cas_college_auto_completed_sectors,
        cas_effective_combined_major_cu, cas_major_pool_major_cu, cas_open_gen_ed_slot_count,
        cas_shared_gened_flex_slots, cas_shared_unrestricted_elective_count,
    };

    let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
    let econ = resolve_major("CAS", "ECON", &[]).expect("ECON");
    let cas_majors = vec![&neur, &econ];
    let combined = cas_effective_combined_major_cu(&cas_majors, 0);
    let open = cas_open_gen_ed_slot_count(&cas_college_auto_completed_sectors(&cas_majors));
    let shared_unrestricted = cas_shared_unrestricted_elective_count(combined, open);
    let gen_ed_flex = cas_shared_gened_flex_slots(combined, open);
    assert_eq!(
        1 + combined + gen_ed_flex + shared_unrestricted,
        CAS_DEGREE_CU,
        "shared CAS degree components should total {CAS_DEGREE_CU} CU"
    );

    let output = generate_schedule(dual_degree_input("CAS", "NEUR", "CAS", "ECON"));
    assert!(output.error.is_none(), "{:?}", output.error);
    assert_eq!(output.degree_results.len(), 2);

    let primary = &output.degree_results[0];
    let secondary = &output.degree_results[1];
    let primary_unrestricted: Vec<_> = primary
        .unfulfilled_requirements
        .iter()
        .chain(primary.fulfilled_requirements.iter())
        .filter(|m| m.requirement.get_category() == CAS_UNRESTRICTED_ELECTIVES_CATEGORY)
        .collect();
    let secondary_unrestricted: Vec<_> = secondary
        .unfulfilled_requirements
        .iter()
        .chain(secondary.fulfilled_requirements.iter())
        .filter(|m| m.requirement.get_category() == CAS_UNRESTRICTED_ELECTIVES_CATEGORY)
        .collect();

    assert_eq!(
        primary_unrestricted.len() as i32,
        shared_unrestricted,
        "primary CAS degree should expose exactly {shared_unrestricted} unrestricted electives, got {}",
        primary_unrestricted.len()
    );
    assert!(
        secondary_unrestricted.is_empty(),
        "secondary CAS major should not list college-wide unrestricted electives"
    );

    let nominal = cas_major_pool_major_cu(&neur) + cas_major_pool_major_cu(&econ);
    assert!(
        nominal > combined || shared_unrestricted == 0,
        "combined major CU should not exceed nominal without overlap savings"
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
        None,
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
    common::assert_overlap_plan_accuracy(&plan, &per_degree, &schools, &majors, &taken, "CAS+CAS");
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
fn cas_writing_course_never_counts_elsewhere() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "ECON".into(),
            school: "CAS".into(),
            kind: "major".into(),
            concentrations: vec![],
            concentration: None,
        }],
        frozen: vec![FrozenCourse {
            course_id: "WRIT 0100".into(),
            year: 1,
            semester: "Fall".into(),
        }],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);
    let dr = &output.degree_results[0];
    let writ_rows: Vec<_> = dr
        .fulfilled_requirements
        .iter()
        .filter(|m| m.course_ids.iter().any(|c| c == "WRIT 0100"))
        .collect();
    assert_eq!(
        writ_rows.len(),
        1,
        "WRIT should appear on exactly one fulfilled row"
    );
    assert_eq!(
        writ_rows[0].instance_id.as_deref(),
        Some("0"),
        "WRIT must only fulfill Writing Seminar"
    );
    assert!(
        !dr.fulfilled_requirements.iter().any(|m| {
            m.instance_id.as_deref() != Some("0") && m.course_ids.iter().any(|c| c == "WRIT 0100")
        }),
        "WRIT must not appear on gen-ed, major, or unrestricted rows"
    );
}

#[test]
fn cas_sector_major_double_count_at_most_one() {
    use degree_planner::penn_data::attributes_data;
    use degree_planner::penn_data::college_data::{SECTOR_SOCIETY, cas_pool_constraints};
    use degree_planner::requirement::evaluate_cas_pool_constraints;
    use std::collections::HashSet;

    let cu = catalog_cu_map();
    let attrs = attributes_data::attributes();
    // Society auto-completed → AUHS / AUPW remain. Both courses carry those attrs.
    let constraints = cas_pool_constraints(&[SECTOR_SOCIETY.to_string()]);
    let major_only = vec!["ECON 0620".to_string(), "CHEM 1012".to_string()];
    let major_set: HashSet<String> = major_only.iter().cloned().collect();

    let evals = evaluate_cas_pool_constraints(
        &major_only,
        &major_only,
        &major_set,
        &constraints,
        &attrs,
        &cu,
    );
    let sector_hits: Vec<_> = evals
        .iter()
        .filter(|e| e.consumption_group.starts_with("cas:sector") && e.fulfilled)
        .collect();
    assert_eq!(
        sector_hits.len(),
        1,
        "at most one major course may double-count toward a sector; got {:?}",
        sector_hits
            .iter()
            .map(|e| (&e.label, &e.course_ids))
            .collect::<Vec<_>>()
    );

    // Treated as non-major → both sectors can be covered.
    let empty_majors = HashSet::new();
    let both = evaluate_cas_pool_constraints(
        &major_only,
        &major_only,
        &empty_majors,
        &constraints,
        &attrs,
        &cu,
    );
    let both_hits = both
        .iter()
        .filter(|e| e.consumption_group.starts_with("cas:sector") && e.fulfilled)
        .count();
    assert_eq!(
        both_hits, 2,
        "without major tagging, both sector courses should count"
    );
}

#[test]
fn orphan_taken_course_absent_from_degree_allocations() {
    // EE has a single Free Elective. Two non-major courses → one fills it, one is orphan.
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "EE".into(),
            school: "SEAS".into(),
            kind: "major".into(),
            concentrations: vec![],
            concentration: None,
        }],
        frozen: vec![
            FrozenCourse {
                course_id: "FNCE 1010".into(),
                year: 1,
                semester: "Fall".into(),
            },
            FrozenCourse {
                course_id: "MKTG 1010".into(),
                year: 1,
                semester: "Spring".into(),
            },
        ],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);
    let dr = &output.degree_results[0];
    let claimed: std::collections::HashSet<&str> = dr
        .fulfilled_requirements
        .iter()
        .chain(dr.unfulfilled_requirements.iter().filter(|m| m.partial))
        .flat_map(|m| m.course_ids.iter().map(|c| c.as_str()))
        .collect();
    let orphans: Vec<_> = ["FNCE 1010", "MKTG 1010"]
        .into_iter()
        .filter(|c| !claimed.contains(*c))
        .collect();
    assert_eq!(
        orphans.len(),
        1,
        "exactly one of the two free courses should be unclaimed, got claimed={claimed:?}"
    );
    let orphan = orphans[0];
    let allocs = output
        .cross_degree_summary
        .as_ref()
        .and_then(|s| s.course_allocations.get(orphan))
        .cloned()
        .unwrap_or_default();
    assert!(
        allocs.is_empty(),
        "orphan {orphan} must not appear in course_allocations: {:?}",
        allocs
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
        None,
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
        assert!(
            !unfulfilled,
            "degree {idx} should not leave WRIT open when taken"
        );
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
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "WH_NOFL".into(),
                school: "WH".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: Some("FNCE".into()),
            },
        ],
        frozen: vec![],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
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
