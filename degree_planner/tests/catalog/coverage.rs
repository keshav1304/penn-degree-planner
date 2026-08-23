use super::*;

#[test]
fn every_catalog_school_lists_at_least_one_major() {
    for school in major::degree_catalog() {
        assert!(
            !school.majors.is_empty(),
            "{} should expose majors",
            school.school_code
        );
    }
}

#[test]
fn math_major_resolves_with_concentrations() {
    use degree_planner::Requirement;

    fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        if pred(req) {
            return true;
        }
        match req {
            Requirement::AnyOf { possibilities, .. }
            | Requirement::CourseGroup { possibilities, .. } => possibilities
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::AllOf { requirements, .. }
            | Requirement::Concentration { requirements, .. } => requirements
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            _ => false,
        }
    }

    fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        major
            .requirements
            .iter()
            .any(|r| requirement_tree_contains(r, pred))
    }

    let general =
        resolve_major("CAS", "MATH", &["General Mathematics".into()]).expect("MATH general");
    assert_eq!(general.short_name, "MATH");
    assert!(general.concentrations.is_some());
    assert!(
        major_contains(&general, &|req| {
            matches!(
                req,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.contains(&"MATH 3001".to_string())
            )
        }),
        "General Mathematics should include MATH 3001"
    );
    assert!(
        major_contains(&general, &|req| {
            matches!(
                req,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.contains(&"MATH 4100".to_string())
            )
        }),
        "General Mathematics should include MATH 4100"
    );
    assert!(
        major_contains(&general, &|req| {
            matches!(
                req,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.contains(&"MATH 3710".to_string())
            )
        }),
        "General Mathematics algebra should include MATH 3710 pair option"
    );
    assert!(
        major_contains(&general, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, .. }
                    if category.as_deref() == Some("Mathematics Electives")
            )
        }),
        "General Mathematics should include mathematics electives"
    );
    assert!(
        !major_contains(&general, &|req| {
            matches!(
                req,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities == &vec!["MATH 3200".to_string()]
            )
        }),
        "General Mathematics should not require MATH 3200 statistics"
    );

    let bio = resolve_major("CAS", "MATH", &["Biological Mathematics".into()]).expect("MATH bio");
    assert!(
        !major_contains(&bio, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, .. }
                    if category.as_deref() == Some("Mathematics Electives")
            )
        }),
        "Biological Mathematics should not include math electives"
    );
    assert!(
        major_contains(&bio, &|req| {
            matches!(
                req,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.contains(&"STAT 4310".to_string())
            )
        }),
        "Biological Mathematics should include STAT 4310"
    );
    let bio_conc_in_pool = bio.requirements.iter().any(|r| {
        let Requirement::CoursePool { fixed_slots, .. } = r else {
            return false;
        };
        fixed_slots.iter().any(|slot| {
            matches!(
                slot,
                Requirement::Concentration { category, .. }
                    if category.as_deref() == Some("Biological Mathematics")
            )
        })
    });
    assert!(
        bio_conc_in_pool,
        "Biological Mathematics concentration should live in the major course pool"
    );
    assert!(major::major_is_implemented("CAS", "MATH"));
}

#[test]
fn psyc_major_resolves_with_distribution_and_electives() {
    use degree_planner::Requirement;

    fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        if pred(req) {
            return true;
        }
        match req {
            Requirement::AnyOf { possibilities, .. }
            | Requirement::CourseGroup { possibilities, .. } => possibilities
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::AllOf { requirements, .. }
            | Requirement::Concentration { requirements, .. } => requirements
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            _ => false,
        }
    }

    fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        major
            .requirements
            .iter()
            .any(|r| requirement_tree_contains(r, pred))
    }

    let psyc = resolve_major("CAS", "PSYC", &[]).expect("PSYC major");
    assert_eq!(psyc.short_name, "PSYC");
    assert!(
        major_contains(&psyc, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, .. }
                    if category.as_deref() == Some("Introductory Psychology")
            )
        }),
        "Psychology should include introductory requirement"
    );
    assert!(
        major_contains(&psyc, &|req| {
            matches!(
                req,
                Requirement::Restriction { category, attr, .. }
                    if category.as_deref() == Some("Biological Basis of Behavior")
                        && attr.as_ref().is_some_and(|a| a.contains(&"APCI".to_string()))
            )
        }),
        "Psychology should include APCI biological elective"
    );
    assert!(
        major_contains(&psyc, &|req| {
            matches!(
                req,
                Requirement::Restriction {
                    category,
                    department,
                    level: Some(4000),
                    excluding,
                    ..
                } if category.as_deref() == Some("Research Experience")
                    && department.as_ref().is_some_and(|d| d.contains(&"PSYC".to_string()))
                    && excluding.as_ref().is_some_and(|e| e.contains(&"PSYC 4997".to_string()))
            )
        }),
        "Psychology research requirement should exclude PSYC 4997"
    );
    assert!(
        major_contains(&psyc, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, .. }
                    if category.as_deref() == Some("Psychology Electives")
            )
        }),
        "Psychology should include four elective slots"
    );
    assert!(major::major_is_implemented("CAS", "PSYC"));
}

#[test]
fn inst_major_requires_four_cu_of_one_intermediate_language() {
    use degree_planner::Requirement;

    fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        if pred(req) {
            return true;
        }
        match req {
            Requirement::AnyOf { possibilities, .. }
            | Requirement::CourseGroup { possibilities, .. } => possibilities
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::AllOf { requirements, .. }
            | Requirement::Concentration { requirements, .. } => requirements
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            _ => false,
        }
    }

    fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        major
            .requirements
            .iter()
            .any(|r| requirement_tree_contains(r, pred))
    }

    fn count_matching(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> usize {
        let here = if pred(req) { 1 } else { 0 };
        let children = match req {
            Requirement::AnyOf { possibilities, .. }
            | Requirement::CourseGroup { possibilities, .. } => possibilities
                .iter()
                .map(|child| count_matching(child, pred))
                .sum(),
            Requirement::AllOf { requirements, .. }
            | Requirement::Concentration { requirements, .. } => requirements
                .iter()
                .map(|child| count_matching(child, pred))
                .sum(),
            Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                .iter()
                .map(|child| count_matching(child, pred))
                .sum(),
            _ => 0,
        };
        here + children
    }

    let spanish = resolve_major("CAS", "INST", &["Spanish".into()]).expect("INST Spanish");
    assert_eq!(spanish.short_name, "INST");
    assert_eq!(spanish.name, "International Studies");
    assert_eq!(
        college_data::cas_major_pool_major_cu(&spanish),
        14,
        "Huntsman INST major block is 14 CU"
    );
    let spanish_lang_slots = spanish
        .requirements
        .iter()
        .map(|r| {
            count_matching(r, &|req| {
                matches!(
                    req,
                    Requirement::Restriction {
                        category,
                        department,
                        level,
                        max_level,
                        number,
                        ..
                    } if category.as_deref() == Some("Language")
                        && department.as_ref().is_some_and(|d| d == &vec!["SPAN".to_string()])
                        && *level == Some(500)
                        && *max_level == Some(4999)
                        && *number == 1
                )
            })
        })
        .sum::<usize>();
    assert_eq!(
        spanish_lang_slots, 4,
        "Spanish track should require 4 CU of SPAN 0500–4999 (not elementary 0100–0400)"
    );
    let arabic = resolve_major("CAS", "INST", &["Arabic".into()]).expect("INST Arabic");
    assert!(
        major_contains(&arabic, &|req| {
            matches!(
                req,
                Requirement::Restriction {
                    department,
                    ..
                } if department.as_ref().is_some_and(|d| d == &vec!["ARAB".to_string()])
            )
        }),
        "Arabic track should lock language slots to ARAB"
    );
    assert!(
        !major_contains(&arabic, &|req| {
            matches!(
                req,
                Requirement::Restriction {
                    department,
                    category,
                    ..
                } if category.as_deref() == Some("Language")
                    && department.as_ref().is_some_and(|d| d.contains(&"SPAN".to_string()))
            )
        }),
        "Arabic track must not accept SPAN for the language requirement"
    );
    assert!(
        major_contains(&spanish, &|req| {
            matches!(
                req,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.contains(&"INSP 1001".to_string())
            )
        }),
        "INST requires INSP 1001"
    );
    assert!(
        major_contains(&spanish, &|req| {
            matches!(
                req,
                Requirement::Restriction {
                    category,
                    attr,
                    ..
                } if category.as_deref() == Some("International Studies")
                    && attr.as_ref().is_some_and(|a| a == &vec!["UNIS".to_string()])
            )
        }),
        "INST requires 2 CU with UNIS"
    );
    assert!(
        major_contains(&spanish, &|req| {
            matches!(
                req,
                Requirement::Restriction {
                    category,
                    attr,
                    ..
                } if category.as_deref() == Some("International Business")
                    && attr.as_ref().is_some_and(|a| a == &vec!["WUIS".to_string()])
            )
        }),
        "INST requires 2 CU with WUIS"
    );
    assert!(major::major_is_implemented("CAS", "INST"));
    assert_eq!(
        major::concentrations_for("CAS", "INST"),
        vec![
            "Arabic",
            "Chinese",
            "French",
            "German",
            "Hindi",
            "Italian",
            "Japanese",
            "Korean",
            "Portuguese",
            "Russian",
            "Spanish",
        ]
    );
}

#[test]
fn bsn_major_resolves_with_requirements() {
    use degree_planner::Requirement;

    fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        if pred(req) {
            return true;
        }
        match req {
            Requirement::AnyOf { possibilities, .. }
            | Requirement::CourseGroup { possibilities, .. } => possibilities
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::AllOf { requirements, .. }
            | Requirement::Concentration { requirements, .. } => requirements
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            _ => false,
        }
    }

    fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        major
            .requirements
            .iter()
            .any(|r| requirement_tree_contains(r, pred))
    }

    let bsn = resolve_major("NURS", "BSN", &[]).expect("BSN major");
    assert_eq!(bsn.short_name, "BSN");
    assert_eq!(bsn.requirements.len(), 30);
    assert!(
        major_contains(&bsn, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, .. }
                    if category.as_deref() == Some("Writing Requirement")
            )
        }),
        "BSN should include writing requirement"
    );
    assert!(
        major_contains(&bsn, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, possibilities, .. }
                    if category.as_deref() == Some("Language Requirement 1")
                        && possibilities.iter().any(|child| matches!(
                            child,
                            Requirement::Restriction { attr, .. }
                                if attr.as_ref().is_some_and(|a| a.contains(&"WUFL".to_string()))
                        ))
            )
        }),
        "BSN language slots should use WUFL attribute per Nursing handbook"
    );
    assert!(
        major_contains(&bsn, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, .. }
                    if category.as_deref() == Some("The Planet & Our Climate")
            )
        }),
        "BSN should include planet sector"
    );
    assert!(
        major_contains(&bsn, &|req| {
            matches!(
                req,
                Requirement::Restriction {
                    category,
                    no_school: Some(school),
                    ..
                } if category.as_deref() == Some("Exploration Course Requirement")
                    && school == "NURS"
            )
        }),
        "BSN should include exploration requirement outside Nursing"
    );
    assert!(
        major_contains(&bsn, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, .. }
                    if category.as_deref() == Some("Ethics Requirement")
            )
        }),
        "BSN should include ethics AnyOf"
    );
    assert!(
        major_contains(&bsn, &|req| {
            matches!(
                req,
                Requirement::Restriction {
                    category,
                    department,
                    level: Some(3510),
                    max_level: Some(3690),
                    ..
                } if category.as_deref() == Some("Nursing Case Study")
                    && department.as_ref().is_some_and(|d| d.contains(&"NURS".to_string()))
            )
        }),
        "BSN should include case study restriction"
    );
    assert!(major::major_is_implemented("NURS", "BSN"));
    assert!(major::major_is_implemented("NURS", "BSN_NOFL"));
}

#[test]
fn nutr_bsn_major_resolves_with_nutrition_requirements() {
    use degree_planner::Requirement;

    fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        if pred(req) {
            return true;
        }
        match req {
            Requirement::AnyOf { possibilities, .. }
            | Requirement::CourseGroup { possibilities, .. } => possibilities
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::AllOf { requirements, .. }
            | Requirement::Concentration { requirements, .. } => requirements
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            Requirement::CoursePool { fixed_slots, .. } => fixed_slots
                .iter()
                .any(|child| requirement_tree_contains(child, pred)),
            _ => false,
        }
    }

    fn major_contains(major: &degree_planner::Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
        major
            .requirements
            .iter()
            .any(|r| requirement_tree_contains(r, pred))
    }

    let nutr = resolve_major("NURS", "NUTR_BSN", &[]).expect("NUTR_BSN major");
    assert_eq!(nutr.short_name, "NUTR_BSN");
    assert_eq!(nutr.requirements.len(), 37);
    assert!(
        major_contains(&nutr, &|req| {
            matches!(
                req,
                Requirement::SingleCourse { possibilities, .. }
                    if possibilities.contains(&"NURS 3120".to_string())
                        || possibilities.contains(&"NURS 5230".to_string())
                        || possibilities.contains(&"NURS 5240".to_string())
            )
        }),
        "Nutrition Science BSN should include required nutrition courses"
    );
    let nune_count = nutr
        .requirements
        .iter()
        .filter(|req| {
            matches!(
                req,
                Requirement::Restriction { attr, .. }
                    if attr.as_ref().is_some_and(|a| a.contains(&"NUNE".to_string()))
            )
        })
        .count();
    assert_eq!(
        nune_count, 4,
        "Nutrition Science BSN should include four NUNE electives"
    );
    assert!(
        major_contains(&nutr, &|req| {
            matches!(
                req,
                Requirement::AnyOf { category, .. }
                    if category.as_deref() == Some("Diversity, Universality, Justice, & Equity")
            )
        }),
        "Nutrition Science BSN should use DUJE sector label"
    );
    assert!(major::major_is_implemented("NURS", "NUTR_BSN"));
    assert!(major::major_is_implemented("NURS", "NUTR_BSN_NOFL"));
}

#[test]
fn bsn_nofl_uses_free_electives_not_language() {
    let nofl = resolve_major("NURS", "BSN_NOFL", &[]).expect("BSN_NOFL major");
    assert!(
        nofl.requirements.iter().any(|req| {
            matches!(
                req,
                Requirement::Restriction { category, .. }
                    if category.as_deref() == Some("Free Elective 1")
            )
        }),
        "NOFL variant should use free electives"
    );
    assert!(
        !nofl.requirements.iter().any(|req| {
            matches!(
                req,
                Requirement::Restriction { category, .. }
                    if category
                        .as_deref()
                        .is_some_and(|c| c.starts_with("Language Requirement"))
            )
        }),
        "NOFL variant should not include language slots"
    );
}

#[test]
fn cas_lists_all_college_majors() {
    assert_eq!(college_data::CAS_DEGREE_CATALOG.len(), 56);
    assert!(college_data::cas_catalog_entry("BIOL").is_some());
    assert!(college_data::cas_catalog_entry("NOT_A_MAJOR").is_none());
}

#[test]
fn minor_catalog_includes_eent() {
    let catalog = major::minor_catalog();
    let seas = catalog
        .iter()
        .find(|s| s.school_code == "SEAS")
        .expect("SEAS in minor catalog");
    assert!(
        seas.majors.iter().any(|m| m.api_code == "EENT"),
        "Engineering Entrepreneurship minor should be selectable"
    );
}

#[test]
fn minor_catalog_includes_data_science() {
    let catalog = major::minor_catalog();
    let seas = catalog
        .iter()
        .find(|s| s.school_code == "SEAS")
        .expect("SEAS in minor catalog");
    assert!(
        seas.majors.iter().any(|m| m.api_code == "DATA_SCI"),
        "Data Science minor should be selectable"
    );
}

#[test]
fn data_science_minor_resolves_six_cu() {
    let minor = major::resolve_minor("SEAS", "DATA_SCI", &[]).expect("DATA_SCI minor resolves");
    assert_eq!(minor.short_name, "DATA_SCI");
    assert_eq!(minor.name, "Data Science");

    let expanded = requirement::expand_restriction_slots(minor.requirements.clone());
    assert_eq!(
        expanded.len(),
        5,
        "4 core SingleCourse rows + 1 CourseGroup electives"
    );
    assert!(
        expanded
            .iter()
            .filter(|r| matches!(r, Requirement::SingleCourse { .. }))
            .count()
            == 4,
        "core requirements should be SingleCourse rows"
    );
    assert!(
        expanded.iter().any(|r| matches!(
            r,
            Requirement::CourseGroup {
                category: Some(cat),
                number: 2,
                ..
            } if cat == "Data Science Electives"
        )),
        "electives should be a CourseGroup of 2 from 5 areas"
    );
}

#[test]
fn minor_catalog_includes_wh_stat_data_science() {
    let catalog = major::minor_catalog();
    let wh = catalog
        .iter()
        .find(|s| s.school_code == "WH")
        .expect("WH in minor catalog");
    assert!(
        wh.majors.iter().any(|m| m.api_code == "STAT_DS"),
        "Statistics and Data Science minor should be selectable"
    );
}

#[test]
fn stat_data_science_minor_resolves_seven_cu() {
    use degree_planner::Requirement;

    let minor = major::resolve_minor("WH", "STAT_DS", &[]).expect("STAT_DS minor resolves");
    assert_eq!(minor.short_name, "STAT_DS");
    assert_eq!(minor.name, "Statistics and Data Science");

    let expanded = requirement::expand_restriction_slots(minor.requirements.clone());
    assert_eq!(
        expanded.len(),
        7,
        "3 core SingleCourse rows + 4 STAT elective restriction slots"
    );
    assert_eq!(
        expanded
            .iter()
            .filter(|r| matches!(r, Requirement::SingleCourse { .. }))
            .count(),
        3,
        "core requirements should be SingleCourse rows"
    );
    let elective_slots: Vec<_> = expanded
        .iter()
        .filter(|r| {
            matches!(
                r,
                Requirement::Restriction {
                    department: Some(depts),
                    level: Some(4050),
                    ..
                } if depts == &["STAT".to_string()]
            )
        })
        .collect();
    assert_eq!(
        elective_slots.len(),
        4,
        "electives should be four STAT restriction slots at min level 4050"
    );
}

#[test]
fn minor_catalog_includes_cas_math() {
    let catalog = major::minor_catalog();
    let cas = catalog
        .iter()
        .find(|s| s.school_code == "CAS")
        .expect("CAS in minor catalog");
    assert!(
        cas.majors.iter().any(|m| m.api_code == "MATH"),
        "Mathematics minor should be selectable"
    );
}

#[test]
fn all_concentrations_separates_math_major_and_minor() {
    let map = major::all_concentrations();
    let major_concs = map
        .get("CAS:MATH:major")
        .expect("CAS Mathematics major concentrations");
    assert!(major_concs.contains(&"Biological Mathematics".to_string()));
    let minor_concs = map
        .get("CAS:MATH:minor")
        .expect("CAS Mathematics minor entry");
    assert!(
        minor_concs.is_empty(),
        "Mathematics minor has no concentrations: {:?}",
        minor_concs
    );
}

#[test]
fn math_minor_has_no_concentrations() {
    let concs = major::concentrations_for_program("CAS", "MATH", "minor");
    assert!(
        concs.is_empty(),
        "Mathematics minor has no concentrations per catalog"
    );
    let major_concs = major::concentrations_for("CAS", "MATH");
    assert!(
        major_concs.contains(&"Biological Mathematics".to_string()),
        "MATH major still exposes concentrations"
    );
}

#[test]
fn math_minor_has_no_biological_mathematics_requirements() {
    let minor = major::resolve_minor("CAS", "MATH", &["Biological Mathematics".into()])
        .expect("CAS Mathematics minor resolves");
    fn categories(reqs: &[requirement::Requirement]) -> Vec<String> {
        reqs.iter()
            .flat_map(|r| {
                let mut cats = vec![r.get_category()];
                match r {
                    requirement::Requirement::AnyOf { possibilities, .. }
                    | requirement::Requirement::AllOf {
                        requirements: possibilities,
                        ..
                    } => {
                        cats.extend(categories(possibilities));
                    }
                    _ => {}
                }
                cats
            })
            .filter(|c| !c.is_empty())
            .collect()
    }
    let cats = categories(&minor.requirements);
    assert!(
        !cats.iter().any(|c| c == "Biological Mathematics"),
        "minor requirements should not include Biological Mathematics: {:?}",
        cats
    );
    assert!(minor.concentrations.is_none());
}

#[test]
fn math_minor_resolves_seven_cu_minimum() {
    let minor = major::resolve_minor("CAS", "MATH", &[]).expect("CAS Mathematics minor resolves");
    assert_eq!(minor.short_name, "MATH");

    let expanded = requirement::expand_restriction_slots(minor.requirements.clone());
    assert_eq!(
        expanded.len(),
        7,
        "MATH minor: 2 calculus + lin alg/proofs + proof-based + 3 electives"
    );
}

#[test]
fn degree_catalog_excludes_minors() {
    let catalog = major::degree_catalog();
    for school in &catalog {
        assert!(
            !school.majors.iter().any(|m| m.api_code == "EENT"),
            "minors should not appear in degree catalog"
        );
    }
}

#[test]
fn eent_minor_resolves_six_cu() {
    let minor = major::resolve_minor("SEAS", "EENT", &["Standard".to_string()])
        .expect("EENT minor resolves");
    assert_eq!(minor.short_name, "EENT");

    let expanded = requirement::expand_restriction_slots(minor.requirements.clone());
    assert_eq!(
        expanded.len(),
        6,
        "EENT Standard: 5450 + 5460/5490 choice + 4 elective slots"
    );

    let fellows = major::resolve_minor("SEAS", "EENT", &["Fellows".to_string()])
        .expect("EENT Fellows resolves");
    let fellows_expanded = requirement::expand_restriction_slots(fellows.requirements.clone());
    assert_eq!(
        fellows_expanded.len(),
        6,
        "EENT Fellows: 5410 + 5430 + 4 elective slots"
    );
}

#[test]
fn eent_single_half_cu_does_not_satisfy_electives() {
    let minor = major::resolve_minor("SEAS", "EENT", &["Standard".to_string()])
        .expect("EENT minor resolves");
    let cu_map = courses_data::cu_map().clone();
    let taken = vec!["EAS 5450".into(), "EAS 5460".into(), "MGMT 2670".into()];
    let validation =
        requirement::validate_courses_for_degree(minor.requirements.clone(), &taken, &cu_map);
    assert!(
        !validation
            .fulfilled
            .iter()
            .any(|m| m.requirement.get_category() == "EENT Electives"),
        "one 0.5 CU elective cannot satisfy a 1 CU elective slot"
    );
    let elective_fulfilled = validation
        .fulfilled
        .iter()
        .filter(|m| m.requirement.get_category() == "EENT Electives")
        .count();
    assert_eq!(
        elective_fulfilled, 0,
        "half-credit alone should not fulfill any elective slot"
    );
}

#[test]
fn eent_half_cu_pairs_accumulate_toward_four_cu_electives() {
    let minor = major::resolve_minor("SEAS", "EENT", &["Standard".to_string()])
        .expect("EENT minor resolves");
    let cu_map = courses_data::cu_map().clone();
    let taken = vec![
        "EAS 5450".into(),
        "EAS 5460".into(),
        "MGMT 2670".into(),
        "MKTG 2270".into(),
        "FNCE 2500".into(),
        "NETS 1120".into(),
        "EAS 5070".into(),
    ];
    let validation =
        requirement::validate_courses_for_degree(minor.requirements.clone(), &taken, &cu_map);
    let elective_fulfilled: Vec<_> = validation
        .fulfilled
        .iter()
        .filter(|m| m.requirement.get_category() == "EENT Electives")
        .collect();
    assert_eq!(
        elective_fulfilled.len(),
        4,
        "four 1 CU elective slots should be fulfilled"
    );
    assert!(
        elective_fulfilled.iter().any(|m| {
            m.course_ids.contains(&"MGMT 2670".to_string())
                && m.course_ids.contains(&"MKTG 2270".to_string())
        }),
        "paired 0.5 CU courses should count as 1 CU in one elective slot"
    );
    assert_eq!(
        validation.unfulfilled.len(),
        0,
        "EENT minor should be fully satisfied"
    );
}

#[test]
fn math_minor_double_counts_with_cas_major() {
    let output = generate_schedule(ScheduleInput {
        taken: vec!["MATH 1400".into(), "MATH 1410".into(), "MATH 1040".into()],
        degrees: vec![
            DegreeInput {
                major: "ECON".into(),
                school: "CAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "MATH".into(),
                school: "CAS".into(),
                kind: "minor".into(),
                concentrations: vec![],
                concentration: None,
            },
        ],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });

    assert_eq!(output.degree_results.len(), 2);
    let minor = output
        .degree_results
        .iter()
        .find(|r| r.kind == "minor")
        .expect("minor result");
    assert_eq!(minor.major, "MATH");
    assert!(
        minor
            .fulfilled_requirements
            .iter()
            .any(|m| m.course_ids.iter().any(|c| c == "MATH 1400")),
        "calculus should count on math minor when shared with CAS major plan"
    );
}

#[test]
fn minor_double_counts_with_major_schedule() {
    let output = generate_schedule(ScheduleInput {
        taken: vec!["MATH 1400".into(), "MATH 1410".into(), "EAS 5450".into()],
        degrees: vec![
            DegreeInput {
                major: "CIS".into(),
                school: "SEAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "EENT".into(),
                school: "SEAS".into(),
                kind: "minor".into(),
                concentrations: vec!["Standard".into()],
                concentration: Some("Standard".into()),
            },
        ],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });

    assert_eq!(output.degree_results.len(), 2);
    let minor = output
        .degree_results
        .iter()
        .find(|r| r.kind == "minor")
        .expect("minor result");
    assert_eq!(minor.major, "EENT");
    assert!(
        minor
            .fulfilled_requirements
            .iter()
            .any(|m| m.course_ids.iter().any(|c| c == "EAS 5450")),
        "EENT core should count on minor even when shared with major plan"
    );
}

#[test]
fn minor_blocks_grad_only_double_count() {
    let output = generate_schedule(ScheduleInput {
        taken: vec!["EAS 5450".into()],
        degrees: vec![
            DegreeInput {
                major: "MS_CIS".into(),
                school: "SEAS_MS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "EENT".into(),
                school: "SEAS".into(),
                kind: "minor".into(),
                concentrations: vec!["Standard".into()],
                concentration: Some("Standard".into()),
            },
        ],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });

    let minor = output
        .degree_results
        .iter()
        .find(|r| r.kind == "minor")
        .expect("minor result");
    assert!(
        !minor
            .fulfilled_requirements
            .iter()
            .any(|m| m.course_ids.iter().any(|c| c == "EAS 5450")),
        "graduate-only overlap must not count toward the minor"
    );
}

#[test]
fn minor_allows_undergrad_grad_and_minor_overlap() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![
            "MATH 1400".into(),
            "MATH 1410".into(),
            "EAS 5450".into(),
            "CIS 5190".into(),
        ],
        degrees: vec![
            DegreeInput {
                major: "CIS".into(),
                school: "SEAS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "MS_CIS".into(),
                school: "SEAS_MS".into(),
                kind: "major".into(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "EENT".into(),
                school: "SEAS".into(),
                kind: "minor".into(),
                concentrations: vec!["Standard".into()],
                concentration: Some("Standard".into()),
            },
        ],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });

    let minor = output
        .degree_results
        .iter()
        .find(|r| r.kind == "minor")
        .expect("minor result");
    assert!(
        minor
            .fulfilled_requirements
            .iter()
            .any(|m| m.course_ids.iter().any(|c| c == "EAS 5450")),
        "undergrad + masters + minor may share EENT core courses"
    );
}

#[test]
fn degree_results_preserve_payload_order_when_one_major_unresolved() {
    // Payload: valid, invalid, valid — results must stay aligned by index for the UI.
    let payload = ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: "CIS".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "NOT_A_REAL_MAJOR".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "EE".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
        ],
        frozen: vec![],
        allow_summer: Some(false),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    };
    let output = generate_schedule(payload);
    assert_eq!(output.degree_results.len(), 3);

    let majors: Vec<&str> = output
        .degree_results
        .iter()
        .map(|r| r.major.as_str())
        .collect();
    assert_eq!(
        majors,
        vec!["CIS", "NOT_A_REAL_MAJOR", "EE"],
        "degree_results must follow payload order; got {majors:?}"
    );
    assert!(output.degree_results[0].error.is_none());
    assert!(
        output.degree_results[1]
            .error
            .as_deref()
            .is_some_and(|e| e.contains("not implemented")),
        "middle entry should be the unresolved major error"
    );
    assert!(output.degree_results[2].error.is_none());

    // #region agent log
    {
        use std::io::Write;
        let log_path = "/Users/thoughtworks/Documents/2. Technology/Course Schedule Optimizer/penn-degree-planner/.cursor/debug-5cbabc.log";
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let _ = writeln!(
                f,
                r#"{{"sessionId":"5cbabc","runId":"post-fix","hypothesisId":"H1","location":"test_suite.rs:degree_results_preserve_payload_order","message":"degree_results order after generate","data":{{"majors":{:?},"errors":{:?}}},"timestamp":{}}}"#,
                majors,
                output
                    .degree_results
                    .iter()
                    .map(|r| r.error.is_some())
                    .collect::<Vec<_>>(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            );
        }
    }
    // #endregion
}

#[test]
fn degree_catalog_excludes_unimplemented_majors() {
    let catalog = major::degree_catalog();
    let cas = catalog
        .iter()
        .find(|s| s.school_code == "CAS")
        .expect("CAS in catalog");
    assert!(
        !cas.majors.iter().any(|m| m.api_code == "BIOP"),
        "gen-ed-only placeholders should not appear in the UI catalog"
    );
    assert!(
        cas.majors.iter().any(|m| m.api_code == "BIOC"),
        "authored CAS Biochemistry should appear in the UI catalog"
    );
    assert!(
        cas.majors.iter().any(|m| m.api_code == "BIOL"),
        "authored CAS Biology should appear in the UI catalog"
    );
    assert!(
        cas.majors.iter().any(|m| m.api_code == "ECON"),
        "implemented CAS majors should remain selectable"
    );
    assert!(
        cas.majors.iter().any(|m| m.api_code == "INST"),
        "authored Huntsman International Studies should appear in the UI catalog"
    );
    assert!(
        catalog.iter().any(|s| s.school_code == "NURS")
            && catalog.iter().any(|s| {
                s.school_code == "NURS"
                    && s.majors.iter().any(|m| m.api_code == "BSN")
                    && s.majors.iter().any(|m| m.api_code == "BSN_NOFL")
                    && s.majors.iter().any(|m| m.api_code == "NUTR_BSN")
                    && s.majors.iter().any(|m| m.api_code == "NUTR_BSN_NOFL")
            }),
        "implemented Nursing majors should appear in the UI catalog"
    );
    let seas_ms = catalog
        .iter()
        .find(|s| s.school_code == "SEAS_MS")
        .expect("SEAS_MS in catalog");
    assert!(
        seas_ms.majors.iter().any(|m| m.api_code == "MS_MEAM"),
        "implemented MS_MEAM should appear in the UI catalog"
    );
    assert!(
        !seas_ms.majors.iter().any(|m| m.api_code == "MS_MSE"),
        "placeholder grad programs should not appear in the UI catalog"
    );
}

#[test]
fn major_is_implemented_inferred_from_requirements() {
    let biop = resolve_major("CAS", "BIOP", &[]).expect("BIOP resolves");
    assert!(
        !major::major_has_authored_requirements("CAS", &biop),
        "CAS placeholder with empty pool major CU should not count as implemented"
    );
    let bioc = resolve_major("CAS", "BIOC", &[]).expect("BIOC resolves");
    assert!(major::major_has_authored_requirements("CAS", &bioc));
    let biol = resolve_major("CAS", "BIOL", &[]).expect("BIOL resolves");
    assert!(major::major_has_authored_requirements("CAS", &biol));
    let econ = resolve_major("CAS", "ECON", &[]).expect("ECON resolves");
    assert!(major::major_has_authored_requirements("CAS", &econ));
    let meam = resolve_major("SEAS_MS", "MS_MEAM", &[]).expect("MS_MEAM resolves");
    assert!(major::major_is_implemented("SEAS_MS", "MS_MEAM"));
    assert!(major::major_has_authored_requirements("SEAS_MS", &meam));
    assert!(major::major_is_implemented("SEAS_MS", "MS_EE"));
    assert!(major::major_is_implemented("SEAS_MS", "MS_BE"));
}

#[test]
fn restriction_description_omits_excluding() {
    let req = Requirement::Restriction {
        category: None,
        department: Some(vec!["PSYC".to_string()]),
        cu: None,
        level: Some(1000),
        max_level: Some(4999),
        attr: None,
        excluding: Some(vec!["PSYC 4997".to_string()]),
        number: 1,
        no_school: None,
    };
    let desc = req.create_requirement_description();
    assert!(!desc.to_lowercase().contains("excluding"));
    assert!(desc.contains("PSYC"));
}

#[test]
fn dmd_major_resolves_with_thirty_seven_cu() {
    let dmd = resolve_major("SEAS", "DMD", &[]).expect("DMD");
    assert_eq!(dmd.short_name, "DMD");
    assert_eq!(dmd.name, "Digital Media Design");
    assert_eq!(dmd.requirements.len(), 35);
    assert!(dmd.concentrations.is_none());
    assert!(major::major_is_implemented("SEAS", "DMD"));
}

#[test]
fn be_major_general_electives_use_course_pool() {
    use degree_planner::Requirement;

    let be = resolve_major("SEAS", "BE", &[]).expect("BE");
    assert_eq!(be.short_name, "BE");
    assert_eq!(be.requirements.len(), 33);
    let pool = be
        .requirements
        .iter()
        .find(|r| matches!(r, Requirement::CoursePool { category, .. } if category.as_deref() == Some("General Electives")))
        .expect("BE should have a General Electives CoursePool");
    if let Requirement::CoursePool {
        fixed_slots,
        flexible_slots,
        constraints,
        ..
    } = pool
    {
        assert!(fixed_slots.is_empty());
        assert_eq!(*flexible_slots, 7);
        assert_eq!(constraints.len(), 5);
        let units: i32 = constraints.iter().map(|c| c.count).sum();
        assert_eq!(units, 8, "8 coverage requirements on 7 pool courses");
    } else {
        panic!("expected CoursePool");
    }
}

#[test]
fn be_pool_slots_count_only_valid_courses() {
    let be = resolve_major("SEAS", "BE", &[]).expect("BE");
    let cu_map = catalog_cu_map();
    let taken = vec!["EAS 2030".to_string()];
    let validation = validate_courses_for_degree(be.requirements.clone(), &taken, &cu_map);
    let pool = validation
        .pool_coverage_info
        .iter()
        .find(|p| p.category == "General Electives")
        .expect("General Electives pool");
    assert_eq!(
        pool.flexible_slots_filled, 1,
        "one taken course should fill one flex slot, got {}",
        pool.flexible_slots_filled
    );
    let cov_done = pool.constraints.iter().filter(|c| c.fulfilled).count();
    assert_eq!(
        cov_done, 2,
        "EAS 2030 should satisfy ethics + one distribution constraint, got {cov_done}"
    );
    assert!(
        pool.pool_courses
            .iter()
            .all(|c| course::is_valid_course_code(c)),
        "pool_courses should not include requirement slot placeholders: {:?}",
        pool.pool_courses
    );
}

#[test]
fn be_pool_schedule_flex_filled_matches_valid_courses_only() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "BE".into(),
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
    assert!(output.error.is_none(), "{:?}", output.error);
    let pool = output.degree_results[0]
        .pool_coverage_info
        .iter()
        .find(|p| p.category == "General Electives")
        .expect("General Electives pool");
    assert_eq!(
        pool.flexible_slots_filled as usize,
        pool.pool_courses.len(),
        "flex slots filled ({}) should match valid pool courses ({})",
        pool.flexible_slots_filled,
        pool.pool_courses.len()
    );
}

#[test]
fn be_pool_schedule_uses_only_general_electives_labels() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "BE".into(),
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
    assert!(output.error.is_none(), "{:?}", output.error);
    for (slot, label) in &output.slot_labels {
        if slot.contains(":p") && slot.contains("29:") {
            assert!(
                label.contains("General Electives"),
                "BE pool flex slot {slot} should be General Electives, got {label}"
            );
        }
        if label.contains("Social Science") {
            panic!(
                "pool constraint label on schedule: {slot} => {label}; all: {:?}",
                output.slot_labels
            );
        }
    }
}

#[test]
fn be_wh_dual_pool_schedule_avoids_constraint_slot_labels() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![
            DegreeInput {
                major: "BE".into(),
                school: "SEAS".into(),
                kind: "major".to_string(),
                concentrations: vec![],
                concentration: None,
            },
            DegreeInput {
                major: "WH_NOFL".into(),
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
    assert!(output.error.is_none(), "{:?}", output.error);
    for (slot, label) in &output.slot_labels {
        if slot.contains(":p") && slot.contains("29:") {
            assert!(
                label.contains("General Electives"),
                "BE pool flex slot {slot} should be General Electives, got {label}"
            );
        }
        assert!(
            !label.contains("Social Science"),
            "pool constraint label on schedule: {slot} => {label}"
        );
    }
    for group in &output.overlap_schedule_groups {
        for m in &group.members {
            if m.degree_index == 0 && m.label.contains("Social Science") {
                panic!(
                    "BE overlap should use pool category, not constraint name: {:?}",
                    group
                );
            }
            if m.degree_index == 0 {
                assert!(
                    m.label.contains("General Electives"),
                    "BE overlap member should be General Electives, got {}",
                    m.label
                );
            }
        }
    }
    let plan = output.overlap_plan.as_ref().expect("overlap plan");
    assert!(
        plan.opportunities.iter().any(|o| {
            o.slots
                .iter()
                .any(|s| common::is_pool_flex_key(&s.slot_key) || s.slot_key.contains(":c"))
        }),
        "BE CoursePool flex/coverage slots must stay overlap-eligible; slots: {:?}",
        plan.opportunities
            .iter()
            .take(12)
            .flat_map(|o| o.slots.iter().map(|s| s.slot_key.as_str()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn ms_be_major_resolves_with_ten_cu() {
    let thesis = resolve_major("SEAS_MS", "MS_BE", &[]).expect("MS_BE thesis default");
    assert_eq!(thesis.short_name, "MS_BE");
    assert_eq!(thesis.name, "Bioengineering, MSE");
    assert_eq!(thesis.requirements.len(), 10);
    assert!(thesis.concentrations.is_some());
    assert!(
        thesis
            .requirements
            .iter()
            .filter(|r| matches!(
                r,
                Requirement::SingleCourse {
                    possibilities,
                    ..
                } if possibilities == &["BE 9990".to_string()]
            ))
            .count()
            == 2,
        "thesis track should require 2 CU of BE 9990"
    );

    let non_thesis =
        resolve_major("SEAS_MS", "MS_BE", &["Non-thesis".into()]).expect("MS_BE non-thesis");
    assert_eq!(non_thesis.requirements.len(), 10);
    assert!(
        !non_thesis.requirements.iter().any(|r| matches!(
            r,
            Requirement::SingleCourse { possibilities, .. }
                if possibilities.contains(&"BE 9990".to_string())
        )),
        "non-thesis track should not require BE 9990"
    );
    assert_eq!(
        major::concentrations_for("SEAS_MS", "MS_BE"),
        vec!["Thesis", "Non-thesis"]
    );
}

#[test]
fn ms_be_taken_thesis_fills_both_slots() {
    let major = resolve_major("SEAS_MS", "MS_BE", &[]).expect("MS_BE");
    let cu_map = courses_data::cu_map();
    assert!(
        (cu_map.get("BE 9990").copied().unwrap_or(0.0) - 2.0).abs() < 1e-6,
        "BE 9990 catalog CU should be 2.0"
    );
    let result = validate_courses_for_degree(
        major.requirements.clone(),
        &vec!["BE 9990".to_string()],
        cu_map,
    );
    let thesis_fulfilled = result
        .fulfilled
        .iter()
        .filter(|m| {
            matches!(
                &m.requirement,
                Requirement::SingleCourse {
                    possibilities,
                    ..
                } if possibilities == &["BE 9990".to_string()]
            )
        })
        .count();
    let thesis_open = result
        .unfulfilled
        .iter()
        .filter(|m| {
            matches!(
                &m.requirement,
                Requirement::SingleCourse {
                    possibilities,
                    ..
                } if possibilities == &["BE 9990".to_string()]
            )
        })
        .count();
    assert_eq!(
        thesis_fulfilled, 2,
        "one taken BE 9990 should fill both 1 CU thesis slots"
    );
    assert_eq!(thesis_open, 0, "no open thesis slots after taking BE 9990");
}

#[test]
fn ms_be_schedule_uses_two_thesis_placeholders() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "MS_BE".into(),
            school: "SEAS_MS".into(),
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
    let suggested = &output.degree_results[0].suggested_for_unfulfilled;
    let thesis_suggestions: Vec<_> = suggested
        .iter()
        .filter(|m| {
            matches!(
                &m.requirement,
                Requirement::SingleCourse {
                    category: Some(cat),
                    possibilities,
                    ..
                } if cat == "Master's Thesis" && possibilities == &["BE 9990".to_string()]
            )
        })
        .collect();
    assert_eq!(thesis_suggestions.len(), 2, "two open thesis units");
    for mapped in &thesis_suggestions {
        assert_eq!(mapped.course_ids.len(), 1);
        let id = &mapped.course_ids[0];
        assert!(
            requirement::is_schedulable_requirement_slot_id(id),
            "duplicate sole-course thesis should schedule as req: placeholder, got {id}"
        );
        assert_ne!(
            id, "BE 9990",
            "must not emit a single concrete BE 9990 for both units"
        );
    }
    assert_ne!(
        thesis_suggestions[0].course_ids[0], thesis_suggestions[1].course_ids[0],
        "placeholders must be distinct so units can sit in different semesters"
    );

    let schedule_ids: Vec<String> = output
        .schedule
        .iter()
        .flat_map(|p| {
            p.courses
                .iter()
                .cloned()
                .chain(p.requirement_slots.iter().cloned())
        })
        .collect();
    let thesis_on_grid: Vec<_> = thesis_suggestions
        .iter()
        .map(|m| m.course_ids[0].as_str())
        .filter(|id| schedule_ids.iter().any(|s| s == id))
        .collect();
    assert_eq!(
        thesis_on_grid.len(),
        2,
        "both thesis placeholders should appear on the schedule grid"
    );

    let with_taken = generate_schedule(ScheduleInput {
        taken: vec!["BE 9990".into()],
        degrees: vec![DegreeInput {
            major: "MS_BE".into(),
            school: "SEAS_MS".into(),
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
    assert!(with_taken.error.is_none(), "{:?}", with_taken.error);
    let open_thesis = with_taken.degree_results[0]
        .suggested_for_unfulfilled
        .iter()
        .filter(|m| {
            matches!(
                &m.requirement,
                Requirement::SingleCourse {
                    possibilities,
                    ..
                } if possibilities == &["BE 9990".to_string()]
            )
        })
        .count();
    assert_eq!(
        open_thesis, 0,
        "taken BE 9990 should clear thesis suggestions"
    );
}

#[test]
fn ms_be_non_thesis_schedule_has_no_be_9990() {
    let output = generate_schedule(ScheduleInput {
        taken: vec![],
        degrees: vec![DegreeInput {
            major: "MS_BE".into(),
            school: "SEAS_MS".into(),
            kind: "major".to_string(),
            concentrations: vec!["Non-thesis".into()],
            concentration: None,
        }],
        frozen: vec![],
        allow_summer: Some(true),
        semester_cu_limits: None,
        gap_semesters: vec![],
        anon_session_id: None,
    });
    assert!(output.error.is_none(), "{:?}", output.error);
    assert!(
        !output.degree_results[0]
            .suggested_for_unfulfilled
            .iter()
            .any(|m| {
                matches!(
                    &m.requirement,
                    Requirement::SingleCourse { possibilities, .. }
                        if possibilities.contains(&"BE 9990".to_string())
                )
            }),
        "non-thesis track should not suggest BE 9990"
    );
}

#[test]
fn single_sole_course_still_suggests_concrete_code() {
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
    assert!(output.error.is_none(), "{:?}", output.error);
    let cis_1200 = output.degree_results[0]
        .suggested_for_unfulfilled
        .iter()
        .find(|m| {
            matches!(
                &m.requirement,
                Requirement::SingleCourse {
                    possibilities,
                    ..
                } if possibilities == &["CIS 1200".to_string()]
            )
        });
    let Some(mapped) = cis_1200 else {
        // CIS tree may nest CIS 1200; accept concrete appearance on the schedule.
        assert!(
            output
                .schedule
                .iter()
                .any(|p| p.courses.iter().any(|c| c == "CIS 1200")),
            "sole-possibility SingleCourse should still schedule as concrete CIS 1200"
        );
        return;
    };
    assert_eq!(mapped.course_ids, vec!["CIS 1200".to_string()]);
}

#[test]
fn ms_meam_major_resolves_with_ten_cu_and_concentrations() {
    let design = resolve_major("SEAS_MS", "MS_MEAM", &[]).expect("MS_MEAM design default");
    assert_eq!(design.short_name, "MS_MEAM");
    assert_eq!(
        design.name,
        "Mechanical Engineering and Applied Mechanics, MSE"
    );
    assert_eq!(design.requirements.len(), 10);
    assert!(design.concentrations.is_some());
    assert_eq!(design.concentrations.as_ref().unwrap().len(), 5);
    assert!(
        design.requirements.iter().any(|r| matches!(
            r,
            Requirement::SingleCourse {
                category: Some(cat),
                possibilities,
                ..
            } if cat == "Concentration" && possibilities == &["MEAM 5140".to_string()]
        )),
        "Design and Manufacturing should require MEAM 5140"
    );
    assert_eq!(
        design
            .requirements
            .iter()
            .filter(|r| r.get_category() == "Concentration")
            .count(),
        3,
        "concentration should contribute 1 required + 2 core elective slots"
    );

    let heat = resolve_major(
        "SEAS_MS",
        "MS_MEAM",
        &["Heat Transfer, Fluid Mechanics, and Energy".into()],
    )
    .expect("MS_MEAM heat");
    assert!(
        heat.requirements.iter().any(|r| matches!(
            r,
            Requirement::SingleCourse {
                category: Some(cat),
                possibilities,
                ..
            } if cat == "Concentration"
                && possibilities.contains(&"MEAM 5360".to_string())
                && possibilities.contains(&"MEAM 5700".to_string())
        )),
        "Heat concentration should allow MEAM 5360 or MEAM 5700 as required"
    );

    assert_eq!(
        major::concentrations_for("SEAS_MS", "MS_MEAM"),
        vec![
            "Design and Manufacturing",
            "Heat Transfer, Fluid Mechanics, and Energy",
            "Mechanics of Materials",
            "Mechatronic and Robotic Systems",
            "Micro/Nano Systems",
        ]
    );
}

#[test]
fn chem_major_resolves() {
    let chem = resolve_major("CAS", "CHEM", &[]).expect("CHEM");
    assert_eq!(chem.short_name, "CHEM");
    assert_eq!(chem.name, "Chemistry");
    assert!(chem.concentrations.is_none());
}

#[test]
fn phys_major_resolves_with_concentrations() {
    use degree_planner::penn_data::college_data::phys_concentration_names;

    assert_eq!(phys_concentration_names().len(), 6);
    let astro = resolve_major("CAS", "PHYS", &["Astrophysics".into()]).expect("PHYS");
    assert_eq!(astro.short_name, "PHYS");
    assert!(astro.concentrations.is_some());
    let bio = resolve_major("CAS", "PHYS", &["Biological Science".into()]).expect("PHYS bio");
    assert_eq!(bio.short_name, "PHYS");
    let chem = resolve_major("CAS", "PHYS", &["Chemical Principles".into()]).expect("PHYS chem");
    assert_eq!(chem.short_name, "PHYS");
    let comp = resolve_major("CAS", "PHYS", &["Computer Techniques".into()]).expect("PHYS comp");
    assert_eq!(comp.short_name, "PHYS");
    let theory = resolve_major(
        "CAS",
        "PHYS",
        &["Physical Theory and Experimental Technique".into()],
    )
    .expect("PHYS theory");
    assert_eq!(theory.short_name, "PHYS");
}

#[test]
fn econ_gen_ed_marks_society_sector_completed_by_major() {
    use degree_planner::penn_data::college_data::{
        SECTOR_SOCIETY, build_cas_gen_ed_info, cas_auto_completed_sectors_for, create_econ_major,
    };

    let major = create_econ_major();
    let cu_map = HashMap::from([("WRIT 0100".to_string(), 1.0)]);
    let taken = vec!["WRIT 0100".to_string()];
    let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
    let pool = validation
        .pool_coverage_info
        .into_iter()
        .find(|p| p.category == "General Education")
        .expect("gen ed pool");
    let info = build_cas_gen_ed_info(&pool, &cas_auto_completed_sectors_for("ECON", None));

    assert_eq!(info.foundational_approaches.len(), 5);
    assert_eq!(info.sectors.len(), 7);
    let society = info
        .sectors
        .iter()
        .find(|s| s.attr == SECTOR_SOCIETY)
        .expect("society sector");
    assert!(society.fulfilled);
    assert!(society.fulfilled_by_major);
}

#[test]
fn neur_gen_ed_marks_living_and_physical_world_completed_by_major() {
    use degree_planner::penn_data::college_data::{
        SECTOR_LIVING_WORLD, SECTOR_PHYSICAL_WORLD, build_cas_gen_ed_info,
        cas_auto_completed_sectors_for,
    };

    let major = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
    let cu_map = HashMap::from([("WRIT 0100".to_string(), 1.0)]);
    let taken = vec!["WRIT 0100".to_string()];
    let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
    let pool = validation
        .pool_coverage_info
        .into_iter()
        .find(|p| p.category == "General Education")
        .expect("gen ed pool");
    let info = build_cas_gen_ed_info(&pool, &cas_auto_completed_sectors_for("NEUR", None));

    for attr in [SECTOR_LIVING_WORLD, SECTOR_PHYSICAL_WORLD] {
        let sector = info
            .sectors
            .iter()
            .find(|s| s.attr == attr)
            .unwrap_or_else(|| panic!("{attr} sector"));
        assert!(sector.fulfilled);
        assert!(sector.fulfilled_by_major);
    }
}

#[test]
fn anth_medical_concentration_completes_hum_soc_sci_sector() {
    use degree_planner::penn_data::college_data::{
        SECTOR_HUM_SOC_SCI, cas_auto_completed_sectors_for,
    };

    let sectors =
        cas_auto_completed_sectors_for("ANTH", Some("Medical Anthropology & Global Health"));
    assert_eq!(sectors, vec![SECTOR_HUM_SOC_SCI.to_string()]);
}

#[test]
fn implemented_majors_resolve_with_requirements() {
    let cases = [
        ("CAS", "NEUR", vec![] as Vec<&str>),
        ("CAS", "ECON", vec![]),
        ("SEAS", "CIS", vec![]),
        ("WH", "WH_NOFL", vec!["FNCE"]),
        ("SEAS_MS", "MS_ROBO", vec![]),
        ("NURS", "BSN", vec![]),
        ("NURS", "NUTR_BSN", vec![]),
    ];
    for (school, major_code, concs) in cases {
        let conc_vec: Vec<String> = concs.into_iter().map(str::to_string).collect();
        let m = resolve_major(school, major_code, &conc_vec)
            .unwrap_or_else(|| panic!("{school}/{major_code} should resolve"));
        assert!(
            !m.requirements.is_empty(),
            "{school}/{major_code} needs requirements"
        );
    }
}

#[test]
fn cas_placeholder_majors_are_valid_stubs() {
    let biop = resolve_major("CAS", "BIOP", &[]).expect("BIOP");
    assert_eq!(biop.short_name, "BIOP");
    assert!(biop.concentrations.is_none());
}

#[test]
fn neur_includes_brain_behavior_and_abbe_pool() {
    let major = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
    let pool = major
        .requirements
        .iter()
        .find_map(|r| match r {
            Requirement::CoursePool { .. } => Some(r),
            _ => None,
        })
        .expect("gen ed pool");
    let Requirement::CoursePool { fixed_slots, .. } = pool else {
        panic!("expected pool");
    };
    assert!(
        fixed_slots
            .iter()
            .any(|r| r.get_category() == "Introduction to Brain & Behavior")
    );
    let abbe = fixed_slots
        .iter()
        .filter(|r| {
            matches!(
                r,
                Requirement::Restriction {
                    attr: Some(attrs),
                    ..
                } if attrs == &vec!["ABBE".to_string()]
            )
        })
        .count();
    assert_eq!(abbe, 3);
}
