//! Major-builder integration tests: `resolve_major`, catalog coverage, and pool invariants.
//!
//! Assertions derive expected behavior from planner helpers (`degree_catalog`,
//! `cas_pool_constraints`, `concentration_names`, etc.) rather than golden snapshots.

use std::collections::HashSet;

use proptest::prelude::*;

use degree_planner::Major;
use degree_planner::Requirement;
use degree_planner::major::{major_has_authored_requirements, major_is_implemented, resolve_major};
use degree_planner::penn_data::college_data::{
    self, cas_auto_completed_sectors_for, cas_gened_requirement_row_count, cas_major_pool_major_cu,
    cas_pool_constraints, CAS_DEGREE_CU, CAS_GENED_POOL_CATEGORY,
};
use degree_planner::penn_data::wharton_data::{
    self, concentration_names, create_wh_fl_major, create_wh_fl_mt_major, create_wh_nofl_major,
    create_wh_nofl_mt_major, normalize_wh_concentrations, resolve_wh_concentration_key,
};
use degree_planner::requirement::expand_restriction_slots;

// ─── Shared helpers ───────────────────────────────────────────────────────────

fn requirement_tree_contains(req: &Requirement, pred: &dyn Fn(&Requirement) -> bool) -> bool {
    if pred(req) {
        return true;
    }
    match req {
        Requirement::AnyOf { possibilities, .. } | Requirement::CourseGroup { possibilities, .. } => {
            possibilities
                .iter()
                .any(|child| requirement_tree_contains(child, pred))
        }
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

fn major_tree_contains(major: &Major, pred: &dyn Fn(&Requirement) -> bool) -> bool {
    major
        .requirements
        .iter()
        .any(|r| requirement_tree_contains(r, pred))
}

fn find_course_pool<'a>(major: &'a Major, category: &str) -> Option<&'a Requirement> {
    major.requirements.iter().find_map(|r| match r {
        Requirement::CoursePool {
            category: cat, ..
        } if cat.as_deref() == Some(category) => Some(r),
        _ => None,
    })
}

fn pool_constraints(pool: &Requirement) -> &[degree_planner::requirement::PoolConstraint] {
    match pool {
        Requirement::CoursePool { constraints, .. } => constraints,
        _ => panic!("expected CoursePool"),
    }
}

fn pool_constraint_attr_codes(pool: &Requirement) -> HashSet<String> {
    pool_constraints(pool)
        .iter()
        .filter_map(|c| match &c.requirement {
            Requirement::Restriction { attr, .. } => attr.clone(),
            _ => None,
        })
        .flatten()
        .collect()
}

fn normalize_like_resolve(mut major: Major) -> Major {
    major.requirements = expand_restriction_slots(major.requirements);
    if let Some(map) = major.concentrations.take() {
        major.concentrations = Some(
            map.into_iter()
                .map(|(name, requirements)| (name, expand_restriction_slots(requirements)))
                .collect(),
        );
    }
    major
}

fn default_concentrations(school: &str, code: &str) -> Vec<String> {
    match (school, code) {
        ("WH", _) => vec!["FNCE".into()],
        ("CAS", "PPE" | "PHYS" | "MATH" | "HSOC" | "INST") => college_data::cas_concentration_names(code)
            .into_iter()
            .next()
            .into_iter()
            .collect(),
        ("SEAS", "MEAM") => vec!["General".into()],
        ("SEAS_MS", "MS_BE") => vec!["Thesis".into()],
        ("SEAS_MS", "MS_MEAM") => vec!["Design and Manufacturing".into()],
        _ => vec![],
    }
}

fn implemented_catalog_cases() -> Vec<(String, String, Vec<String>)> {
    let mut cases = Vec::new();
    for school in degree_planner::major::degree_catalog() {
        for entry in &school.majors {
            if !major_is_implemented(&school.school_code, &entry.api_code) {
                continue;
            }
            cases.push((
                school.school_code.clone(),
                entry.api_code.clone(),
                default_concentrations(&school.school_code, &entry.api_code),
            ));
        }
    }
    cases
}

fn wh_builder_for(code: &str, concs: Vec<String>) -> Major {
    match code {
        "WH_FL" => create_wh_fl_major(concs),
        "WH_NOFL" => create_wh_nofl_major(concs),
        "WH_FL_MT" => create_wh_fl_mt_major(concs),
        "WH_NOFL_MT" => create_wh_nofl_mt_major(concs),
        other => panic!("unknown Wharton code {other}"),
    }
}

fn count_top_level_business_breadth(major: &Major) -> usize {
    major
        .requirements
        .iter()
        .filter(|r| {
            matches!(
                r,
                Requirement::AnyOf { category, .. }
                    if category
                        .as_deref()
                        .is_some_and(|c| c.to_lowercase().contains("business breadth"))
            )
        })
        .count()
}

fn assert_cas_gened_pool_invariants(major: &Major, short_name: &str, concentration: Option<&str>) {
    assert!(
        major_tree_contains(major, &|req| {
            matches!(
                req,
                Requirement::Restriction { category, department, .. }
                    if category.as_deref() == Some("Writing Seminar")
                        && department.as_ref().is_some_and(|d| d.contains(&"WRIT".to_string()))
            )
        }),
        "{short_name} should include a standalone writing requirement",
    );

    let pool = find_course_pool(major, CAS_GENED_POOL_CATEGORY)
        .unwrap_or_else(|| panic!("{short_name} missing gen-ed pool"));
    let Requirement::CoursePool {
        fixed_slots,
        flexible_slots,
        constraints,
        ..
    } = pool
    else {
        panic!("expected gen-ed CoursePool");
    };

    let auto = cas_auto_completed_sectors_for(short_name, concentration);
    assert_eq!(
        constraints.len(),
        cas_pool_constraints(&auto).len(),
        "{short_name} gen-ed constraints should match auto-completed sectors",
    );
    assert!(*flexible_slots > 0, "{short_name} gen-ed pool needs flexible slots");
    assert!(
        *flexible_slots <= cas_gened_requirement_row_count() as i32,
        "{short_name} gen-ed flex should not exceed FA+sector row cap",
    );

    let major_cu_in_pool = cas_major_pool_major_cu(major);
    if major_cu_in_pool > 0 {
        assert!(
            !fixed_slots.is_empty(),
            "{short_name} majors with pool major CU should embed requirements in fixed slots",
        );
    } else {
        assert!(
            fixed_slots.is_empty(),
            "{short_name} placeholder majors should not embed major requirements in the gen-ed pool",
        );
    }

    let unrestricted_count = major
        .requirements
        .iter()
        .filter(|r| r.get_category() == "Unrestricted Electives")
        .count() as i32;
    assert_eq!(
        1 + major_cu_in_pool + flexible_slots + unrestricted_count,
        CAS_DEGREE_CU,
        "{short_name} writing + pool + unrestricted electives should total {CAS_DEGREE_CU} CU",
    );
}

fn assert_wh_las_pool_invariants(major: &Major, expects_foreign_language: bool) {
    let pool = find_course_pool(major, "Liberal Arts and Sciences")
        .unwrap_or_else(|| panic!("{} missing LAS pool", major.short_name));
    let Requirement::CoursePool {
        flexible_slots,
        constraints,
        fixed_slots,
        ..
    } = pool
    else {
        panic!("expected LAS CoursePool");
    };

    assert!(fixed_slots.is_empty(), "LAS pools use flexible placeholders");
    assert!(*flexible_slots > 0);
    assert!(!constraints.is_empty());

    let attrs = pool_constraint_attr_codes(pool);
    assert_eq!(
        attrs.contains("WUFL"),
        expects_foreign_language,
        "{} WUFL expectation",
        major.name
    );

    let coverage_units: i32 = constraints.iter().map(|c| c.count).sum();
    assert!(
        coverage_units >= *flexible_slots,
        "LAS coverage units should cover pool capacity",
    );
}

fn assert_resolve_matches_direct_builder(school: &str, code: &str, concs: &[String]) {
    let direct = normalize_like_resolve(match school {
        "WH" => wh_builder_for(code, concs.to_vec()),
        _ => resolve_major(school, code, concs).expect("seed resolve for non-WH builder parity"),
    });
    let resolved = resolve_major(school, code, concs).expect("resolve_major");
    assert_eq!(resolved.short_name, direct.short_name);
    assert_eq!(resolved.name, direct.name);
    assert_eq!(resolved.requirements.len(), direct.requirements.len());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Catalog ↔ resolve_major — every implemented degree builds
// ═══════════════════════════════════════════════════════════════════════════════

mod catalog_resolve {
    use super::*;

    #[test]
    fn implemented_catalog_majors_resolve_with_authored_requirements() {
        for (school, code, concs) in implemented_catalog_cases() {
            let major = resolve_major(&school, &code, &concs)
                .unwrap_or_else(|| panic!("{school}/{code} should resolve"));
            assert!(
                !major.requirements.is_empty(),
                "{school}/{code} needs requirements",
            );
            assert!(
                major_has_authored_requirements(&school, &major),
                "{school}/{code} should count as implemented",
            );
            assert_eq!(
                major_is_implemented(&school, &code),
                major_has_authored_requirements(&school, &major),
            );
        }
    }

    #[test]
    fn cas_catalog_placeholders_resolve_but_stay_unimplemented() {
        for entry in college_data::CAS_DEGREE_CATALOG {
            if major_is_implemented("CAS", entry.api_code) {
                continue;
            }
            let major = resolve_major("CAS", entry.api_code, &[])
                .unwrap_or_else(|| panic!("{} should still resolve as stub", entry.api_code));
            assert_eq!(major.short_name, entry.api_code);
            assert!(
                !major_has_authored_requirements("CAS", &major),
                "{placeholder} placeholder should remain gen-ed-only",
                placeholder = entry.api_code,
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        #[test]
        fn resolve_major_is_stable_on_repeat(
            case_index in 0..implemented_catalog_cases().len(),
        ) {
            let cases = implemented_catalog_cases();
            prop_assume!(case_index < cases.len());
            let (school, code, concs) = &cases[case_index];
            let first = resolve_major(school, code, concs).expect("first resolve");
            let second = resolve_major(school, code, concs).expect("second resolve");
            prop_assert_eq!(first.short_name, second.short_name);
            prop_assert_eq!(first.requirements.len(), second.requirements.len());
            prop_assert_eq!(first.name, second.name);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. CAS major builder — shared gen-ed pool mechanics
// ═══════════════════════════════════════════════════════════════════════════════

mod cas_builder {
    use super::*;
    use degree_planner::penn_data::college_data::{
        create_anch_major, create_bioc_major, create_biol_major, create_chem_major,
        create_cis_cas_major, create_dsgn_major, create_econ_major, create_mathecon_major,
        create_neur_major, create_psyc_major,
    };

    #[test]
    fn implemented_cas_majors_satisfy_gened_pool_invariants() {
        for (major, short_name) in [
            (create_econ_major(), "ECON"),
            (create_neur_major(), "NEUR"),
            (create_bioc_major(), "BIOC"),
            (create_biol_major(), "BIOL"),
            (create_chem_major(), "CHEM"),
            (create_anch_major(), "ANCH"),
            (create_mathecon_major(), "MECON"),
            (create_cis_cas_major(), "CIS"),
            (create_psyc_major(), "PSYC"),
            (create_dsgn_major(), "DSGN"),
        ] {
            assert_cas_gened_pool_invariants(&major, short_name, None);
            assert!(major_has_authored_requirements("CAS", &major));
        }
    }

    #[test]
    fn cas_placeholder_stubs_use_empty_major_pool_and_full_gened_flex() {
        let biop = college_data::create_cas_placeholder_major(
            college_data::cas_catalog_entry("BIOP").expect("BIOP catalog entry"),
        );
        assert_cas_gened_pool_invariants(&biop, "BIOP", None);
        assert_eq!(cas_major_pool_major_cu(&biop), 0);
        assert!(!major_has_authored_requirements("CAS", &biop));
    }

    #[test]
    fn cas_concentration_majors_build_from_catalog_names() {
        for code in ["PPE", "PHYS", "MATH", "HSOC"] {
            for conc in college_data::cas_concentration_names(code) {
                let concs = vec![conc.clone()];
                let major = resolve_major("CAS", code, &concs)
                    .unwrap_or_else(|| panic!("{code} / {conc}"));
                assert!(!major.requirements.is_empty());
                assert_cas_gened_pool_invariants(&major, code, Some(&conc));
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Wharton major builder — variants, LAS pools, concentration catalog
// ═══════════════════════════════════════════════════════════════════════════════

mod wh_builder {
    use super::*;

    #[test]
    fn wh_majors_share_one_concentration_catalog() {
        let catalog_keys = wharton_data::concentration_names();
        for code in ["WH_FL", "WH_NOFL", "WH_FL_MT", "WH_NOFL_MT"] {
            let major = wh_builder_for(code, vec!["FNCE".into()]);
            let catalog = major
                .concentrations
                .as_ref()
                .expect("Wharton majors expose concentrations");
            let mut keys: Vec<_> = catalog.keys().cloned().collect();
            keys.sort();
            assert_eq!(keys, catalog_keys);
            assert!(catalog.values().all(|reqs| !reqs.is_empty()));
        }
    }

    #[test]
    fn wh_legacy_concentration_labels_map_to_catalog_keys() {
        for (label, key) in [
            ("Finance", "FNCE"),
            ("Management", "MGMT"),
            ("OIDD: Decision Processes", "ODDP"),
            ("Statistics and Data Science", "STAT"),
        ] {
            let resolved = resolve_wh_concentration_key(label).expect(label);
            assert!(
                wharton_data::create_wh_concentrations().contains_key(&resolved),
                "{label} -> {resolved}",
            );
            assert_eq!(resolved, key);
        }
        assert!(resolve_wh_concentration_key("OIDD").is_none());
    }

    #[test]
    fn wh_fl_requires_foreign_language_in_las_pool_nofl_does_not() {
        let fl = create_wh_fl_major(vec!["FNCE".into()]);
        let nofl = create_wh_nofl_major(vec!["FNCE".into()]);
        assert_wh_las_pool_invariants(&fl, true);
        assert_wh_las_pool_invariants(&nofl, false);
    }

    #[test]
    fn wh_nofl_schedules_more_business_breadth_than_fl_for_same_concentrations() {
        let concs = vec!["FNCE".into()];
        let fl = create_wh_fl_major(concs.clone());
        let nofl = create_wh_nofl_major(concs.clone());
        assert!(
            count_top_level_business_breadth(&nofl) > count_top_level_business_breadth(&fl),
            "NOFL should schedule more standalone business-breadth slots than FL",
        );

        let double = vec!["MGMT".into(), "FNCE".into()];
        let nofl_double = create_wh_nofl_major(double);
        assert!(
            count_top_level_business_breadth(&nofl_double)
                < count_top_level_business_breadth(&nofl),
            "double concentration should reduce NOFL business-breadth slots",
        );
    }

    #[test]
    fn wh_mt_fl_uses_las_pool_and_nofl_mt_uses_standalone_las_requirements() {
        let fl_mt = create_wh_fl_mt_major(vec!["FNCE".into()]);
        assert_wh_las_pool_invariants(&fl_mt, true);

        let nofl_mt = create_wh_nofl_mt_major(vec!["STAT".into()]);
        assert!(
            find_course_pool(&nofl_mt, "Liberal Arts and Sciences").is_none(),
            "NOFL M&T uses standalone LAS requirements, not a course pool",
        );
        assert!(
            major_tree_contains(&nofl_mt, &|req| {
                req.get_category()
                    .starts_with("Liberal Arts and Sciences")
            }),
        );
    }

    #[test]
    fn wh_resolve_major_matches_direct_builders() {
        for code in ["WH_FL", "WH_NOFL", "WH_FL_MT", "WH_NOFL_MT"] {
            assert_resolve_matches_direct_builder("WH", code, &["FNCE".into()]);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]

        #[test]
        fn normalize_wh_concentrations_keeps_catalog_keys_and_at_most_two(
            picks in prop::collection::vec(
                prop::sample::select(concentration_names()),
                1..=4,
            ),
        ) {
            let normalized = normalize_wh_concentrations(
                &picks.iter().cloned().map(String::from).collect::<Vec<_>>(),
            );
            prop_assert!(normalized.len() <= 2);
            for key in &normalized {
                prop_assert!(wharton_data::create_wh_concentrations().contains_key(key));
            }
            let mut deduped = picks;
            deduped.sort();
            deduped.dedup();
            prop_assert!(normalized.len() <= deduped.len());
        }

        #[test]
        fn wh_variants_build_with_any_catalog_concentration(
            code in prop_oneof!["WH_FL", "WH_NOFL", "WH_FL_MT", "WH_NOFL_MT"],
            conc in prop::sample::select(concentration_names()),
        ) {
            let major = wh_builder_for(&code, vec![conc.clone()]);
            prop_assert!(!major.requirements.is_empty());
            let catalog = major.concentrations.as_ref().expect("catalog");
            prop_assert!(catalog.contains_key(&conc));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Nursing major builder — language track vs exempt track
// ═══════════════════════════════════════════════════════════════════════════════

mod nursing_builder {
    use super::*;
    use degree_planner::penn_data::nursing_data::{
        create_bsn_major, create_bsn_nofl_major, create_nutr_bsn_major, create_nutr_bsn_nofl_major,
    };

    fn has_language_requirement_slots(major: &Major) -> bool {
        major_tree_contains(major, &|req| req.get_category().starts_with("Language Requirement"))
    }

    fn has_free_elective_slots(major: &Major) -> bool {
        major_tree_contains(major, &|req| req.get_category().starts_with("Free Elective"))
    }

    #[test]
    fn nursing_language_and_exempt_tracks_differ_on_elective_slots() {
        let bsn = create_bsn_major();
        let bsn_nofl = create_bsn_nofl_major();
        assert!(has_language_requirement_slots(&bsn));
        assert!(!has_free_elective_slots(&bsn));
        assert!(!has_language_requirement_slots(&bsn_nofl));
        assert!(has_free_elective_slots(&bsn_nofl));

        let nutr = create_nutr_bsn_major();
        let nutr_nofl = create_nutr_bsn_nofl_major();
        assert!(has_language_requirement_slots(&nutr));
        assert!(!has_language_requirement_slots(&nutr_nofl));
        assert!(has_free_elective_slots(&nutr_nofl));
    }

    #[test]
    fn nursing_majors_resolve_through_catalog() {
        for (code, build) in [
            ("BSN", create_bsn_major as fn() -> Major),
            ("BSN_NOFL", create_bsn_nofl_major),
            ("NUTR_BSN", create_nutr_bsn_major),
            ("NUTR_BSN_NOFL", create_nutr_bsn_nofl_major),
        ] {
            let direct = build();
            let resolved = resolve_major("NURS", code, &[]).expect(code);
            assert_eq!(resolved.short_name, direct.short_name);
            assert_eq!(resolved.name, direct.name);
            assert!(major_has_authored_requirements("NURS", &resolved));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. SEAS major builder — undergrad programs and MEAM concentrations
// ═══════════════════════════════════════════════════════════════════════════════

mod seas_builder {
    use super::*;
    use degree_planner::penn_data::seas_data::{
        concentration_names_for, create_ai_major, create_be_major, create_cis_major,
        create_cmpe_major, create_dmd_major, create_ee_major, create_meam_major, create_mse_major,
    };

    #[test]
    fn seas_undergrad_programs_resolve_with_matching_short_names() {
        for (code, major) in [
            ("EE", create_ee_major()),
            ("MSE", create_mse_major()),
            ("CIS", create_cis_major()),
            ("AI", create_ai_major()),
            ("CMPE", create_cmpe_major()),
            ("BE", create_be_major()),
            ("DMD", create_dmd_major()),
        ] {
            assert_eq!(major.short_name, code);
            assert!(major_has_authored_requirements("SEAS", &major));
            let resolved = resolve_major("SEAS", code, &[]).expect(code);
            assert_eq!(resolved.short_name, code);
            assert_eq!(
                resolved.requirements.len(),
                normalize_like_resolve(major).requirements.len(),
            );
        }
    }

    #[test]
    fn be_general_electives_pool_covers_more_units_than_flex_slots() {
        let be = create_be_major();
        let pool = find_course_pool(&be, "General Electives").expect("BE pool");
        let Requirement::CoursePool {
            fixed_slots,
            flexible_slots,
            constraints,
            ..
        } = pool
        else {
            panic!("expected pool");
        };
        assert!(fixed_slots.is_empty());
        assert!(*flexible_slots > 0);
        let coverage_units: i32      = constraints.iter().map(|c| c.count).sum();
        assert!(
            coverage_units > *flexible_slots,
            "BE pool allows double-counting across constraint groups",
        );
    }

    #[test]
    fn meam_builds_for_every_catalog_concentration() {
        for conc in concentration_names_for("MEAM") {
            let major = create_meam_major(conc.clone());
            assert_eq!(major.short_name, "MEAM");
            assert!(!major.requirements.is_empty());
            let expected = normalize_like_resolve(create_meam_major(conc.clone()));
            let resolved = resolve_major("SEAS", "MEAM", &[conc]).expect("MEAM resolve");
            assert_eq!(resolved.requirements.len(), expected.requirements.len());
            assert_eq!(resolved.short_name, expected.short_name);
        }
    }
}
