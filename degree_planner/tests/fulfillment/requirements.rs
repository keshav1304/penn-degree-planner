use super::*;

#[test]
fn restriction_number_expands_to_individual_slots() {
    let cu_map = sample_cu_map();
    let expanded = expand_restriction_slots(vec![Requirement::Restriction {
        category: Some("Test elective".into()),
        department: Some(vec!["TEST".into()]),
        cu: None,
        level: None,
        max_level: None,
        attr: None,
        excluding: None,
        number: 4,
        no_school: None,
    }]);
    let explicit = vec![one_cu_restriction(); 4];
    let taken = vec![
        "TEST 1000".into(),
        "TEST 1001".into(),
        "TEST 1002".into(),
        "TEST 1003".into(),
    ];
    let a = validate_courses_for_degree(expanded, &taken, &cu_map);
    let b = validate_courses_for_degree(explicit, &taken, &cu_map);
    assert_eq!(a.fulfilled.len(), b.fulfilled.len());
    assert_eq!(a.unfulfilled.len(), b.unfulfilled.len());
}

#[test]
fn half_cu_course_does_not_fill_one_cu_slot() {
    let attributes = attributes_data::create_attributes();
    let mut cu_map = HashMap::new();
    cu_map.insert("TEST 1000".into(), 0.5);
    let taken = vec!["TEST 1000".into()];
    assert!(
        one_cu_restriction()
            .fulfills_requirement(&taken, &attributes, &cu_map)
            .is_none()
    );
}

#[test]
fn attribute_exclusion_blocks_even_when_attr_matches() {
    let attributes = attributes_data::create_attributes();
    assert!(!course_matches_restriction(
        "AFRC 0030",
        &None,
        &None,
        &None,
        &Some(vec!["AIRE".into()]),
        &Some(vec!["AUFS".into()]),
        &None,
        &attributes,
    ));
    assert!(course_matches_restriction(
        "BEPP 2010",
        &None,
        &None,
        &None,
        &Some(vec!["AIRE".into()]),
        &Some(vec!["AUFS".into()]),
        &None,
        &attributes,
    ));
}

#[test]
fn nurs_exploration_excludes_nurs_courses() {
    let attributes = attributes_data::create_attributes();
    let no_school = Some("NURS".to_string());
    assert!(!course_matches_restriction(
        "NURS 1030",
        &None,
        &None,
        &None,
        &None,
        &None,
        &no_school,
        &attributes,
    ));
    assert!(course_matches_restriction(
        "ECON 0100",
        &None,
        &None,
        &None,
        &None,
        &None,
        &no_school,
        &attributes,
    ));
}

#[test]
fn taken_courses_partition_into_fulfilled_and_open() {
    let cu_map = catalog_cu_map();
    let major = resolve_major("SEAS", "CIS", &[]).expect("CIS");
    let taken = vec!["CIS 1100".into(), "CIS 1200".into()];
    let v = validate_courses_for_degree(major.requirements.clone(), &taken, &cu_map);
    assert!(!v.fulfilled.is_empty());
    let fulfilled_courses: HashSet<_> = v
        .fulfilled
        .iter()
        .flat_map(|m| m.course_ids.iter())
        .collect();
    for c in &taken {
        assert!(fulfilled_courses.contains(c));
    }
}

#[test]
fn requirement_slot_ids_are_distinguishable_from_courses() {
    assert!(is_requirement_slot_id("req:0"));
    assert!(!is_requirement_slot_id("CIS 1200"));
    assert!(is_schedulable_requirement_slot_id("req:1:f0"));
    assert!(!is_schedulable_requirement_slot_id("req:1:c0"));
    assert!(is_pool_constraint_slot_id("req:1:c0"));
}

#[test]
fn anyof_placeholder_uses_category_name_not_generic_options_text() {
    use degree_planner::penn_data::requirement_builders::{any_of, restriction};

    let req = any_of(
        "Additional Biology",
        vec![
            restriction(1)
                .departments(&["BIOL"])
                .level(2000)
                .max_level(5999)
                .into(),
            restriction(1).attr(&["ABB2"]).into(),
        ],
    );
    assert_eq!(
        req.schedule_label_for_requirement(),
        "Additional Biology",
        "long AnyOf slots should show the category, not 'One of the following options'"
    );
    let slot_id = req
        .schedulable_placeholder_id(Some("1:f9:c0"))
        .expect("Additional Biology should have a category placeholder id");
    assert!(
        slot_id.contains("A:Additional_Biology"),
        "placeholder id should keep the category slug, got {slot_id}"
    );
    assert_eq!(req.slot_label_for_id(&slot_id), "Additional Biology");

    let suggested = req.suggest_for_requirement(
        &vec![],
        attributes_data::attributes(),
        catalog_cu_map(),
        Some("1:f9:c0"),
        None,
    );
    assert_eq!(
        suggested,
        Some(vec![slot_id.clone()]),
        "unpaired Additional Biology should schedule as the category block, not BIOL 2000–5999"
    );
}
