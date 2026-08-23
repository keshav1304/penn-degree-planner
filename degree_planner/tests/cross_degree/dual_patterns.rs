use super::*;

struct DualPattern {
    label: &'static str,
    school1: &'static str,
    major1: &'static str,
    conc1: Option<&'static str>,
    school2: &'static str,
    major2: &'static str,
    conc2: Option<&'static str>,
    max_occupied_year: i32,
    must_schedule: &'static [&'static str],
    require_overlap: bool,
}

fn pattern_input(case: &DualPattern) -> ScheduleInput {
    let mut input = dual_degree_input_with_conc(
        case.school1,
        case.major1,
        case.conc1,
        case.school2,
        case.major2,
        case.conc2,
    );
    if case.school1 == "WH" || case.school2 == "WH" {
        let wh = input
            .degrees
            .iter_mut()
            .find(|d| d.school == "WH")
            .expect("Wharton degree");
        if wh.concentration.is_none() && wh.concentrations.is_empty() {
            wh.concentration = Some("FNCE".into());
            wh.concentrations = vec!["FNCE".into()];
        }
    }
    input
}

fn assert_pattern(case: &DualPattern) {
    assert!(
        major::major_is_implemented(case.school1, case.major1),
        "{}: {} {} is not authored",
        case.label,
        case.school1,
        case.major1
    );
    assert!(
        major::major_is_implemented(case.school2, case.major2),
        "{}: {} {} is not authored",
        case.label,
        case.school2,
        case.major2
    );
    let output = generate_schedule(pattern_input(case));
    if case.require_overlap {
        assert_healthy_dual_degree_schedule(&output, case.label, case.max_occupied_year);
    } else {
        assert!(
            output.error.is_none(),
            "{}: pipeline error: {:?}",
            case.label,
            output.error
        );
        assert_eq!(output.degree_results.len(), 2, "{}", case.label);
        for result in &output.degree_results {
            assert!(
                result.error.is_none(),
                "{}: {} {} error: {:?}",
                case.label,
                result.school,
                result.major,
                result.error
            );
        }
        assert_schedule_respects_cu_limits(&output, case.label);
        let occupied = occupied_schedule_max_year(&output);
        assert!(
            occupied <= case.max_occupied_year,
            "{}: occupied max year {occupied} (limit {}); total_cu={:.1}",
            case.label,
            case.max_occupied_year,
            output.schedule.iter().map(|p| p.total_cu).sum::<f64>()
        );
        assert_no_generic_anyof_grid_labels(&output, case.label);
        assert_no_named_course_plus_option_placeholder(&output, case.label);
    }
    let courses: HashSet<&str> = output
        .schedule
        .iter()
        .flat_map(|p| p.courses.iter().map(String::as_str))
        .collect();
    for course in case.must_schedule {
        assert!(
            courses.contains(course),
            "{}: expected {course} on the grid; courses: {:?}",
            case.label,
            courses
        );
    }
}

fn run_patterns(cases: &[DualPattern]) {
    assert!(!cases.is_empty());
    for case in cases {
        assert_pattern(case);
    }
}

/// Huntsman is BA International Studies (INST) + BS Economics with language (WH_FL).
#[test]
fn huntsman_pairs_international_studies_with_wharton_language() {
    assert!(
        college_data::CAS_DEGREE_CATALOG
            .iter()
            .any(|e| e.api_code == "INST" && e.display_name == "International Studies"),
        "Huntsman College degree is International Studies (INST)"
    );
    assert!(major::major_is_implemented("CAS", "INST"));
    assert!(
        major::major_is_implemented("WH", "WH_FL"),
        "Huntsman uses the language-required Wharton path"
    );
    run_patterns(&[DualPattern {
        label: "Huntsman INST + WH_FL",
        school1: "CAS",
        major1: "INST",
        conc1: Some("Spanish"),
        school2: "WH",
        major2: "WH_FL",
        conc2: Some("FNCE"),
        max_occupied_year: 4,
        must_schedule: &["INSP 1001"],
        require_overlap: true,
    }]);
}

#[test]
fn coordinated_m_and_t_lsm_and_viper_schedules_are_healthy() {
    run_patterns(&[
        DualPattern {
            label: "M&T CIS + WH_NOFL_MT",
            school1: "SEAS",
            major1: "CIS",
            conc1: None,
            school2: "WH",
            major2: "WH_NOFL_MT",
            conc2: Some("FNCE"),
            max_occupied_year: 5,
            must_schedule: &["OIDD 2340"],
            require_overlap: true,
        },
        DualPattern {
            label: "M&T MEAM + WH_NOFL_MT",
            school1: "SEAS",
            major1: "MEAM",
            conc1: None,
            school2: "WH",
            major2: "WH_NOFL_MT",
            conc2: Some("FNCE"),
            max_occupied_year: 5,
            must_schedule: &["OIDD 2340"],
            require_overlap: true,
        },
        DualPattern {
            label: "LSM Biology + WH_NOFL",
            school1: "CAS",
            major1: "BIOL",
            conc1: None,
            school2: "WH",
            major2: "WH_NOFL",
            conc2: Some("FNCE"),
            max_occupied_year: 4,
            must_schedule: &["MATH 1400"],
            require_overlap: true,
        },
        DualPattern {
            label: "LSM Biochemistry + WH_NOFL",
            school1: "CAS",
            major1: "BIOC",
            conc1: None,
            school2: "WH",
            major2: "WH_NOFL",
            conc2: Some("FNCE"),
            max_occupied_year: 5,
            must_schedule: &[],
            require_overlap: true,
        },
        DualPattern {
            label: "VIPER Physics + MEAM",
            school1: "CAS",
            major1: "PHYS",
            conc1: Some("Physical Theory and Experimental Technique"),
            school2: "SEAS",
            major2: "MEAM",
            conc2: None,
            max_occupied_year: 5,
            must_schedule: &[],
            require_overlap: true,
        },
        DualPattern {
            label: "VIPER Math + CIS",
            school1: "CAS",
            major1: "MATH",
            conc1: Some("General Mathematics"),
            school2: "SEAS",
            major2: "CIS",
            conc2: None,
            max_occupied_year: 5,
            must_schedule: &[],
            require_overlap: true,
        },
    ]);
}

#[test]
fn uncoordinated_common_school_pairings_are_healthy() {
    run_patterns(&[
        DualPattern {
            label: "College + Wharton ECON + WH_NOFL",
            school1: "CAS",
            major1: "ECON",
            conc1: None,
            school2: "WH",
            major2: "WH_NOFL",
            conc2: Some("FNCE"),
            max_occupied_year: 5,
            must_schedule: &[],
            require_overlap: true,
        },
        DualPattern {
            label: "College + Wharton (language) ECON + WH_FL",
            school1: "CAS",
            major1: "ECON",
            conc1: None,
            school2: "WH",
            major2: "WH_FL",
            conc2: Some("FNCE"),
            max_occupied_year: 5,
            must_schedule: &[],
            require_overlap: true,
        },
        DualPattern {
            label: "College + Wharton PSYC + WH_NOFL",
            school1: "CAS",
            major1: "PSYC",
            conc1: None,
            school2: "WH",
            major2: "WH_NOFL",
            conc2: Some("FNCE"),
            max_occupied_year: 5,
            must_schedule: &[],
            require_overlap: true,
        },
        DualPattern {
            label: "SEAS + Wharton (uncoordinated) CIS + WH_NOFL",
            school1: "SEAS",
            major1: "CIS",
            conc1: None,
            school2: "WH",
            major2: "WH_NOFL",
            conc2: Some("FNCE"),
            max_occupied_year: 5,
            must_schedule: &[],
            require_overlap: true,
        },
        DualPattern {
            label: "College + SEAS ECON + CIS",
            school1: "CAS",
            major1: "ECON",
            conc1: None,
            school2: "SEAS",
            major2: "CIS",
            conc2: None,
            max_occupied_year: 5,
            must_schedule: &[],
            require_overlap: true,
        },
        DualPattern {
            label: "Nursing + College BSN_NOFL + NEUR",
            school1: "NURS",
            major1: "BSN_NOFL",
            conc1: None,
            school2: "CAS",
            major2: "NEUR",
            conc2: None,
            max_occupied_year: 6,
            must_schedule: &[],
            require_overlap: true,
        },
    ]);
}
