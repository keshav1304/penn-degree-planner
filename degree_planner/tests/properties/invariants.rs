use super::*;

#[test]
fn valid_catalog_courses_have_non_negative_cu() {
    for c in courses_data::courses() {
        if course::is_valid_course_code(&c.course_code) {
            assert!(c.cu >= 0.0, "{} has negative CU ({})", c.course_code, c.cu);
        }
    }
}

proptest! {
    #[test]
    fn cu_limit_never_below_one(
        schools in prop::collection::vec("(CAS|SEAS|WH)".prop_map(String::from), 1..3),
        year in 1i32..6,
        semester in prop_oneof!["Fall", "Spring", "Summer"],
    ) {
        let limit = default_semester_cu_limit(&schools, year, &semester);
        prop_assert!(limit >= 1.0);
        prop_assert!(limit <= 10.0);
    }

    #[test]
    fn dual_cas_never_gets_six_point_five(
        year in 2i32..5,
        semester in prop_oneof!["Fall", "Spring"],
    ) {
        let schools = vec!["CAS".into(), "CAS".into()];
        prop_assert_eq!(default_semester_cu_limit(&schools, year, &semester), 5.5);
    }
}

#[test]
fn ee_chem_gen_ed_backfills_y4s_before_y5() {
    let output = generate_schedule(dual_degree_input("SEAS", "EE", "CAS", "CHEM"));
    assert!(output.error.is_none(), "{:?}", output.error);

    let gen_ed_in_y5: Vec<_> = output
        .schedule
        .iter()
        .filter(|p| p.year >= 5)
        .flat_map(|p| {
            p.requirement_slots
                .iter()
                .map(|slot| (p.year, p.semester.as_str(), slot.clone()))
        })
        .filter(|(_, _, slot)| {
            output
                .slot_labels
                .get(slot.as_str())
                .is_some_and(|l| l == "General Education")
        })
        .collect();

    let y4s = output
        .schedule
        .iter()
        .find(|p| p.year == 4 && p.semester == "Spring")
        .expect("Y4 Spring");
    let limit = default_semester_cu_limit(&["SEAS".into(), "CAS".into()], 4, "Spring");
    let y4s_has_spare = y4s.total_cu + 1.0 <= limit + CU_EPS;

    assert!(
        gen_ed_in_y5.is_empty() || !y4s_has_spare,
        "gen-ed slots should backfill Y4 Spring before year 5 when space remains \
         (Y4S={:.1}/{:.1}, Y5 gen-eds={:?})",
        y4s.total_cu,
        limit,
        gen_ed_in_y5
    );
}

#[test]
fn generated_dual_schedules_always_respect_cu_limits() {
    for (label, input) in implemented_dual_undergrad_pairs() {
        let output = generate_schedule(input);
        assert_schedule_respects_cu_limits(&output, &label);
    }
}
