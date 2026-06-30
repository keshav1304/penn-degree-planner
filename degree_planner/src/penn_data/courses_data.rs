use std::collections::HashMap;
use std::sync::OnceLock;

use crate::Course;

const COURSES_JSON: &str = include_str!("courses.json");

static COURSES: OnceLock<Vec<Course>> = OnceLock::new();
static CU_MAP: OnceLock<HashMap<String, f64>> = OnceLock::new();

fn init_courses() -> &'static Vec<Course> {
    COURSES.get_or_init(|| {
        serde_json::from_str(COURSES_JSON).expect("courses.json must deserialize to Vec<Course>")
    })
}

/// Borrow the full course catalog. Parsed once from embedded JSON on first access.
pub fn courses() -> &'static [Course] {
    init_courses().as_slice()
}

/// Course-code → CU lookup, built once on first access.
pub fn cu_map() -> &'static HashMap<String, f64> {
    CU_MAP.get_or_init(|| {
        init_courses()
            .iter()
            .map(|c| (c.course_code.clone(), c.cu))
            .collect()
    })
}
