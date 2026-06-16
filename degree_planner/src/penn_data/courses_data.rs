use std::sync::OnceLock;

use crate::Course;

const COURSES_JSON: &str = include_str!("courses.json");

static COURSES: OnceLock<Vec<Course>> = OnceLock::new();

fn courses() -> &'static Vec<Course> {
    COURSES.get_or_init(|| {
        serde_json::from_str(COURSES_JSON).expect("courses.json must deserialize to Vec<Course>")
    })
}

/// Returns all courses. Parsed once from embedded JSON on first access.
pub fn all_courses() -> Vec<Course> {
    courses().clone()
}
