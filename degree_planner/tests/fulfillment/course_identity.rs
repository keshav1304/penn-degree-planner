use super::*;

#[test]
fn valid_codes_match_dept_number_pattern() {
    assert!(course::is_valid_course_code("CIS 1200"));
    assert!(course::is_valid_course_code("MATH 1400"));
    assert!(!course::is_valid_course_code("CIS1200"));
    assert!(!course::is_valid_course_code("not a course"));
    assert!(!course::is_valid_course_code(""));
}

#[test]
fn graduate_level_uses_course_number_threshold() {
    assert!(!course::is_graduate_level("CIS 1200"));
    assert!(course::is_graduate_level("CIS 5190"));
}

proptest! {
    #[test]
    fn invalid_strings_are_rejected(s in "\\PC*") {
        prop_assume!(!s.contains(' '));
        prop_assume!(!s.chars().any(|c| c.is_ascii_digit()));
        prop_assert!(!course::is_valid_course_code(&s));
    }

    #[test]
    fn synthetic_valid_codes_round_trip(dept in "[A-Z]{2,4}", num in 1000u32..9999) {
        let code = format!("{dept} {num}");
        prop_assume!(course::is_valid_course_code(&code));
        prop_assert_eq!(course::course_number(&code), Some(num as i32));
    }
}
