use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::course_relations;

const ATTRIBUTES_JSON: &str = include_str!("attributes.json");

static ATTRIBUTES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
static ATTRIBUTE_SETS: OnceLock<HashMap<String, HashSet<String>>> = OnceLock::new();

fn init_attributes() -> &'static HashMap<String, Vec<String>> {
    ATTRIBUTES.get_or_init(|| {
        serde_json::from_str(ATTRIBUTES_JSON)
            .expect("attributes.json must deserialize to HashMap<String, Vec<String>>")
    })
}

/// Borrow the static course-attribute index, parsed once on first access.
pub fn attributes() -> &'static HashMap<String, Vec<String>> {
    init_attributes()
}

fn attribute_sets() -> &'static HashMap<String, HashSet<String>> {
    ATTRIBUTE_SETS.get_or_init(|| {
        attributes()
            .iter()
            .map(|(name, courses)| (name.clone(), courses.iter().cloned().collect()))
            .collect()
    })
}

/// Whether `course` is listed under `attr`, including also-offered aliases.
pub fn attribute_contains_course(attr: &str, course: &str) -> bool {
    let Some(set) = attribute_sets().get(attr) else {
        return false;
    };
    if set.contains(course) {
        return true;
    }
    for alias in course_relations::aliases(course) {
        if set.contains(alias) {
            return true;
        }
    }
    course_relations::vec_contains_equiv(
        attributes().get(attr).map(|v| v.as_slice()).unwrap_or(&[]),
        course,
    )
}

/// Clone of the attribute map for callers that need an owned or mutable copy.
pub fn create_attributes() -> HashMap<String, Vec<String>> {
    attributes().clone()
}
