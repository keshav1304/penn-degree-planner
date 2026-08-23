use super::*;

#[test]
fn wh_fl_mt_pool_blocks_cc_ssh_double_count_in_same_group() {
    use degree_planner::penn_data::wharton_data::create_wh_fl_mt_major;

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
    use degree_planner::penn_data::wharton_data::create_wh_nofl_mt_major;

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
fn wh_concentration_names_are_short_codes_and_exclude_oidd() {
    use degree_planner::penn_data::wharton_data::{
        concentration_names, create_wh_concentrations, resolve_wh_concentration_key,
    };

    let catalog = create_wh_concentrations();
    assert!(!catalog.contains_key("OIDD"));
    assert!(catalog.contains_key("MAOM"));
    assert_eq!(catalog.len(), 13);

    let names = concentration_names();
    assert!(names.contains(&"FNCE".to_string()));
    assert!(names.contains(&"STAT".to_string()));
    assert!(names.contains(&"MAOM".to_string()));
    assert!(names.contains(&"HCMG".to_string()));
    assert!(names.contains(&"ODDP".to_string()));
    assert!(names.contains(&"ODGN".to_string()));
    assert!(names.contains(&"ODIS".to_string()));
    assert!(names.contains(&"ODOM".to_string()));
    assert!(!names.iter().any(|n| n == "OIDD"));

    assert_eq!(
        resolve_wh_concentration_key("Marketing & Operations Management"),
        Some("MAOM".to_string())
    );
    assert_eq!(
        resolve_wh_concentration_key("Health Care Management"),
        Some("HCMG".to_string())
    );
    assert_eq!(
        resolve_wh_concentration_key("OIDD: Decision Processes"),
        Some("ODDP".to_string())
    );
    assert_eq!(
        resolve_wh_concentration_key("OIDD: General"),
        Some("ODGN".to_string())
    );
    assert_eq!(
        resolve_wh_concentration_key("OIDD: Information Systems"),
        Some("ODIS".to_string())
    );
    assert_eq!(
        resolve_wh_concentration_key("OIDD: Operations Management"),
        Some("ODOM".to_string())
    );
    assert_eq!(resolve_wh_concentration_key("OIDD"), None);
}

#[test]
fn wh_oidd_information_systems_concentration_requires_four_electives() {
    use degree_planner::Requirement;
    use degree_planner::penn_data::wharton_data::create_wh_concentrations;

    let catalog = create_wh_concentrations();
    let is = catalog.get("ODIS").expect("ODIS concentration");
    assert_eq!(is.len(), 4);
    for slot in is {
        assert!(matches!(
            slot,
            Requirement::SingleCourse { possibilities, .. }
                if possibilities.len() == 7
        ));
    }
}

#[test]
fn wh_oidd_operations_management_concentration_requires_core_and_three_electives() {
    use degree_planner::Requirement;
    use degree_planner::penn_data::wharton_data::create_wh_concentrations;

    let catalog = create_wh_concentrations();
    let om = catalog.get("ODOM").expect("ODOM concentration");
    assert_eq!(om.len(), 4);
    assert!(matches!(
        &om[0],
        Requirement::SingleCourse { possibilities, .. }
            if possibilities == &vec!["OIDD 2200".to_string(), "OIDD 2210".to_string()]
    ));
    for slot in &om[1..] {
        assert!(matches!(
            slot,
            Requirement::SingleCourse { possibilities, .. }
                if possibilities.len() == 6
        ));
    }
}

#[test]
fn wh_oidd_general_concentration_requires_four_wuod_courses() {
    use degree_planner::Requirement;
    use degree_planner::penn_data::wharton_data::create_wh_concentrations;

    let catalog = create_wh_concentrations();
    let general = catalog.get("ODGN").expect("ODGN concentration");
    assert_eq!(general.len(), 1);
    assert!(matches!(
        &general[0],
        Requirement::Restriction {
            number: 4,
            attr: Some(attrs),
            ..
        } if attrs == &vec!["WUOD".to_string()]
    ));
}

#[test]
fn wh_oidd_decision_processes_concentration_requires_core_and_two_electives() {
    use degree_planner::Requirement;
    use degree_planner::penn_data::wharton_data::create_wh_concentrations;

    let catalog = create_wh_concentrations();
    let dp = catalog.get("ODDP").expect("ODDP concentration");
    assert_eq!(dp.len(), 4);
    assert!(matches!(
        &dp[0],
        Requirement::SingleCourse { possibilities, .. }
            if possibilities == &vec!["OIDD 2900".to_string()]
    ));
    assert!(matches!(
        &dp[1],
        Requirement::SingleCourse { possibilities, .. }
            if possibilities == &vec!["OIDD 2910".to_string()]
    ));
    for slot in &dp[2..] {
        assert!(matches!(
            slot,
            Requirement::SingleCourse { possibilities, .. }
                if possibilities.len() == 16
        ));
    }
}

#[test]
fn wh_hcmg_concentration_requires_1010_and_three_electives() {
    use degree_planner::Requirement;
    use degree_planner::penn_data::wharton_data::create_wh_concentrations;

    let catalog = create_wh_concentrations();
    let hc = catalog.get("HCMG").expect("HCMG concentration");
    assert_eq!(hc.len(), 4);
    assert!(matches!(
        &hc[0],
        Requirement::SingleCourse { possibilities, .. }
            if possibilities == &vec!["HCMG 1010".to_string()]
    ));
    for slot in &hc[1..] {
        assert!(matches!(
            slot,
            Requirement::Restriction {
                department,
                level: Some(2000),
                max_level: Some(4000),
                excluding,
                ..
            } if department.as_ref().is_some_and(|d| d == &vec!["HCMG".to_string()])
                && excluding.as_ref().is_some_and(|e| e.contains(&"HCMG 1010".to_string()))
        ));
    }
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
        .find(|c| c.name == "FNCE")
        .expect("FNCE concentration tracker");
    assert!(fnce.requirements_fulfilled > 0);
}
