//! Catalog-wide invariants: a course named in a top-level `SingleCourse` slot must
//! fulfill that slot when taken or frozen, including half-credit catalog courses.
//!
//! The Leadership Journey / M&T freshman panel bug was `consume_single_course_slot_cu`
//! requiring 1.0 CU, so WH 1010 / OIDD 2340 (0.5 CU) never counted. These tests
//! fail if that class of bug returns.

use std::collections::HashMap;

use degree_planner::Major;
use degree_planner::Requirement;
use degree_planner::course;
use degree_planner::course_relations;
use degree_planner::major::{
    concentrations_for, degree_catalog, minor_catalog, resolve_major, resolve_minor,
};
use degree_planner::penn_data::courses_data;
use degree_planner::requirement::{
    DegreeValidationResult, extract_concentration_info, validate_courses_for_degree,
};
use degree_planner::scheduler::{DegreeInput, FrozenCourse, ScheduleInput, generate_schedule};

const CU_EPS: f64 = 0.001;

fn cu_map() -> &'static HashMap<String, f64> {
    courses_data::cu_map()
}

fn catalog_cu(course: &str) -> f64 {
    cu_map().get(course).copied().unwrap_or(1.0)
}

fn is_half_credit(course: &str) -> bool {
    catalog_cu(course) + CU_EPS < 1.0
}

/// `(kind, school, program, concentrations)` for every implemented catalog entry.
fn implemented_programs() -> Vec<(String, String, String, Vec<String>)> {
    let mut out = Vec::new();
    for school in degree_catalog() {
        for major in school.majors {
            out.push((
                "major".into(),
                school.school_code.clone(),
                major.api_code,
                vec![],
            ));
        }
    }
    for school in minor_catalog() {
        for minor in school.majors {
            out.push((
                "minor".into(),
                school.school_code.clone(),
                minor.api_code,
                vec![],
            ));
        }
    }
    // Concentrations rewrite requirement lists (MEAM slots, Wharton STAT vs FNCE).
    for (school, program) in [
        ("SEAS", "MEAM"),
        ("WH", "WH_FL"),
        ("WH", "WH_NOFL"),
        ("WH", "WH_NOFL_MT"),
        ("WH", "WH_FL_MT"),
    ] {
        for conc in concentrations_for(school, program) {
            if conc == "None" {
                continue;
            }
            out.push(("major".into(), school.into(), program.into(), vec![conc]));
        }
    }
    out
}

fn resolve_program(kind: &str, school: &str, program: &str, concs: &[String]) -> Major {
    if kind == "minor" {
        resolve_minor(school, program, concs)
            .unwrap_or_else(|| panic!("resolve minor {school}:{program}"))
    } else {
        resolve_major(school, program, concs)
            .unwrap_or_else(|| panic!("resolve major {school}:{program}"))
    }
}

fn top_level_single_courses(major: &Major) -> Vec<&Requirement> {
    major
        .requirements
        .iter()
        .filter(|req| matches!(req, Requirement::SingleCourse { .. }))
        .collect()
}

fn single_course_lists(req: &Requirement, course: &str) -> bool {
    match req {
        Requirement::SingleCourse { possibilities, .. } => {
            possibilities.iter().any(|c| c == course)
        }
        _ => false,
    }
}

fn fulfilled_named_single_course(result: &DegreeValidationResult, course: &str) -> bool {
    result.fulfilled.iter().any(|m| {
        single_course_lists(&m.requirement, course)
            && m.course_ids
                .iter()
                .any(|c| course_relations::equivalent(c, course))
    })
}

fn program_label(kind: &str, school: &str, program: &str, concs: &[String]) -> String {
    if concs.is_empty() {
        format!("{kind}:{school}:{program}")
    } else {
        format!("{kind}:{school}:{program}[{}]", concs.join(","))
    }
}

/// Synthetic half-credit slot — does not depend on the Penn catalog.
#[test]
fn synthetic_half_credit_single_course_fulfills() {
    let reqs = vec![Requirement::SingleCourse {
        category: Some("Named half-credit".into()),
        possibilities: vec!["TEST 0500".into()],
    }];
    let cu = HashMap::from([("TEST 0500".into(), 0.5)]);
    let result = validate_courses_for_degree(reqs, &vec!["TEST 0500".into()], &cu);
    assert!(
        fulfilled_named_single_course(&result, "TEST 0500"),
        "0.5 CU named course must fill its SingleCourse slot; fulfilled={:?} unfulfilled={:?}",
        result
            .fulfilled
            .iter()
            .map(|m| &m.course_ids)
            .collect::<Vec<_>>(),
        result.unfulfilled.len()
    );
    assert!(result.unfulfilled.is_empty());
}

/// 2.0 CU named course still fills two identical 1 CU SingleCourse slots.
#[test]
fn synthetic_two_cu_course_fills_two_single_course_slots() {
    let slot = Requirement::SingleCourse {
        category: Some("Thesis".into()),
        possibilities: vec!["TEST 2000".into()],
    };
    let reqs = vec![slot.clone(), slot];
    let cu = HashMap::from([("TEST 2000".into(), 2.0)]);
    let result = validate_courses_for_degree(reqs, &vec!["TEST 2000".into()], &cu);
    let filled = result
        .fulfilled
        .iter()
        .filter(|m| single_course_lists(&m.requirement, "TEST 2000"))
        .count();
    assert_eq!(filled, 2, "2.0 CU course should fill both 1 CU named slots");
}

#[test]
fn every_top_level_single_course_possibility_fulfills_when_taken() {
    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for (kind, school, program, concs) in implemented_programs() {
        let major = resolve_program(&kind, &school, &program, &concs);
        let label = program_label(&kind, &school, &program, &concs);
        let slots = top_level_single_courses(&major);
        for req in slots {
            let Requirement::SingleCourse {
                possibilities,
                category,
            } = req
            else {
                continue;
            };
            let cat = category.as_deref().unwrap_or("");
            for course in possibilities {
                if !course::is_valid_course_code(course) {
                    continue;
                }
                checked += 1;
                let result = validate_courses_for_degree(
                    major.requirements.clone(),
                    &vec![course.clone()],
                    cu_map(),
                );
                if !fulfilled_named_single_course(&result, course) {
                    failures.push(format!(
                        "{label} / {cat} / {course} ({} CU)",
                        catalog_cu(course)
                    ));
                }
            }
        }
    }

    assert!(
        checked > 0,
        "expected to check at least one named SingleCourse possibility"
    );
    assert!(
        failures.is_empty(),
        "named SingleCourse possibilities must fulfill their slot when taken ({} failures):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

#[test]
fn catalog_half_credit_named_courses_are_covered_and_fulfill() {
    let mut half_credit: Vec<(String, String, String)> = Vec::new();

    for (kind, school, program, concs) in implemented_programs() {
        let major = resolve_program(&kind, &school, &program, &concs);
        let label = program_label(&kind, &school, &program, &concs);
        for req in top_level_single_courses(&major) {
            let Requirement::SingleCourse { possibilities, .. } = req else {
                continue;
            };
            for course in possibilities {
                if course::is_valid_course_code(course) && is_half_credit(course) {
                    half_credit.push((
                        label.clone(),
                        course.clone(),
                        catalog_cu(course).to_string(),
                    ));
                }
            }
        }
    }

    assert!(
        half_credit
            .iter()
            .any(|(label, course, _)| { label.contains("WH_NOFL_MT") && course == "WH 1010" }),
        "sweep must see WH 1010 on M&T FL-exempt; found {half_credit:?}"
    );
    assert!(
        half_credit
            .iter()
            .any(|(label, course, _)| { label.contains("WH_NOFL_MT") && course == "OIDD 2340" }),
        "sweep must see OIDD 2340 on M&T FL-exempt; found {half_credit:?}"
    );
    assert!(
        half_credit
            .iter()
            .any(|(_, course, _)| course == "MGMT 3010"),
        "sweep must see MGMT 3010 Leadership Journey; found {half_credit:?}"
    );
}

#[test]
fn frozen_half_credit_named_courses_show_on_wharton_requirements() {
    for program in ["WH_FL", "WH_NOFL", "WH_NOFL_MT", "WH_FL_MT"] {
        let major = resolve_major("WH", program, &["STAT".into()]).expect(program);
        let mut frozen_codes: Vec<String> = Vec::new();
        for req in top_level_single_courses(&major) {
            let Requirement::SingleCourse { possibilities, .. } = req else {
                continue;
            };
            if let Some(course) = possibilities
                .iter()
                .find(|c| course::is_valid_course_code(c) && is_half_credit(c))
            {
                if !frozen_codes.iter().any(|c| c == course) {
                    frozen_codes.push(course.clone());
                }
            }
        }
        assert!(
            !frozen_codes.is_empty(),
            "{program} should name at least one half-credit SingleCourse"
        );

        let output = generate_schedule(ScheduleInput {
            taken: vec![],
            degrees: vec![DegreeInput {
                major: program.into(),
                school: "WH".into(),
                kind: "major".into(),
                concentrations: vec!["STAT".into()],
                concentration: None,
            }],
            frozen: frozen_codes
                .iter()
                .map(|course_id| FrozenCourse {
                    course_id: course_id.clone(),
                    year: 1,
                    semester: "Fall".into(),
                })
                .collect(),
            allow_summer: Some(true),
            semester_cu_limits: None,
            gap_semesters: vec![],
            anon_session_id: None,
        });
        assert!(output.error.is_none(), "{program}: {:?}", output.error);
        let result = output
            .degree_results
            .iter()
            .find(|r| r.major == program)
            .unwrap_or_else(|| panic!("{program} degree_results"));

        let missing: Vec<&String> = frozen_codes
            .iter()
            .filter(|course| {
                !result.fulfilled_requirements.iter().any(|m| {
                    single_course_lists(&m.requirement, course)
                        && m.course_ids
                            .iter()
                            .any(|c| course_relations::equivalent(c, course))
                })
            })
            .collect();
        assert!(
            missing.is_empty(),
            "{program}: frozen half-credit courses must appear on fulfilled requirements: {missing:?}"
        );
    }
}

#[test]
fn overlay_concentration_named_courses_count_when_taken() {
    let mut failures: Vec<String> = Vec::new();

    for program in ["EE", "MSE", "CIS", "CMPE", "BE"] {
        for conc in concentrations_for("SEAS", program) {
            if conc == "None" {
                continue;
            }
            let major = resolve_major("SEAS", program, &[conc.clone()]).expect(program);
            let Some(conc_reqs) = major.concentrations.as_ref().and_then(|m| m.get(&conc)) else {
                continue;
            };
            let mut seen: Vec<String> = Vec::new();
            for req in conc_reqs {
                let Requirement::SingleCourse { possibilities, .. } = req else {
                    continue;
                };
                let Some(course) = possibilities.iter().find(|c| {
                    course::is_valid_course_code(c)
                        && !seen.iter().any(|s| course_relations::equivalent(s, c))
                }) else {
                    continue;
                };
                seen.push(course.clone());
                let taken = vec![course.clone()];
                let infos = extract_concentration_info(
                    &major.requirements,
                    &major.concentrations,
                    &[conc.clone()],
                    &taken,
                    cu_map(),
                    None,
                );
                let Some(info) = infos.iter().find(|i| i.name == conc) else {
                    failures.push(format!("{program}/{conc}: missing tracker"));
                    continue;
                };
                let matched = info
                    .matched_courses
                    .iter()
                    .flatten()
                    .any(|c| course_relations::equivalent(c, course));
                if !matched {
                    failures.push(format!(
                        "{program}/{conc} / {course} ({} CU)",
                        catalog_cu(course)
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "overlay concentration named courses must count when taken:\n  {}",
        failures.join("\n  ")
    );
}
