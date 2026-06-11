use std::collections::{HashMap, HashSet};

use crate::attributes_data;
use crate::course;
use crate::courses_data;
use crate::requirement::course_matches_restriction;

/// Inverted catalog indexes for fast restriction candidate lookup.
#[derive(Debug, Clone)]
pub struct CatalogIndex {
    pub all_courses: Vec<String>,
    courses_by_attr: HashMap<String, HashSet<String>>,
    courses_by_dept: HashMap<String, HashSet<String>>,
}

impl CatalogIndex {
    pub fn build() -> Self {
        let attributes = attributes_data::create_attributes();
        let mut courses_by_attr: HashMap<String, HashSet<String>> = HashMap::new();
        for (attr, courses) in &attributes {
            courses_by_attr
                .entry(attr.clone())
                .or_default()
                .extend(courses.iter().cloned());
        }

        let mut courses_by_dept: HashMap<String, HashSet<String>> = HashMap::new();
        let mut all_courses = Vec::new();
        for c in courses_data::all_courses() {
            if !course::is_valid_course_code(&c.course_code) {
                continue;
            }
            all_courses.push(c.course_code.clone());
            courses_by_dept
                .entry(c.dept_code.clone())
                .or_default()
                .insert(c.course_code.clone());
        }
        all_courses.sort();

        Self {
            all_courses,
            courses_by_attr,
            courses_by_dept,
        }
    }

    pub fn unrestricted_undergrad_set(&self) -> HashSet<String> {
        self.all_courses
            .iter()
            .filter(|c| !course::is_graduate_level(c))
            .cloned()
            .collect()
    }

    /// Courses matching a restriction, excluding `taken`.
    pub fn candidates_for_restriction(
        &self,
        department: &Option<Vec<String>>,
        level: &Option<i32>,
        attr: &Option<Vec<String>>,
        excluding: &Option<Vec<String>>,
        no_school: &Option<String>,
        taken: &HashSet<String>,
    ) -> HashSet<String> {
        let attributes = attributes_data::create_attributes();
        let mut base: Option<HashSet<String>> = None;

        if let Some(attrs) = attr.as_ref().filter(|a| !a.is_empty()) {
            let mut union = HashSet::new();
            for name in attrs {
                if let Some(set) = self.courses_by_attr.get(name) {
                    union.extend(set.iter().cloned());
                }
            }
            base = Some(union);
        } else if let Some(depts) = department.as_ref().filter(|d| !d.is_empty()) {
            let mut union = HashSet::new();
            for dept in depts {
                if let Some(set) = self.courses_by_dept.get(dept) {
                    union.extend(set.iter().cloned());
                }
            }
            base = Some(union);
        } else if no_school.is_some() {
            base = Some(self.unrestricted_undergrad_set());
        } else {
            base = Some(self.unrestricted_undergrad_set());
        }

        let mut out = base.unwrap_or_default();
        out.retain(|c| {
            !taken.contains(c)
                && course_matches_restriction(
                    c,
                    department,
                    level,
                    attr,
                    excluding,
                    no_school,
                    &attributes,
                )
        });
        out
    }

    pub fn candidates_for_one_of(
        &self,
        possibilities: &[String],
        taken: &HashSet<String>,
    ) -> HashSet<String> {
        possibilities
            .iter()
            .filter(|c| course::is_valid_course_code(c) && !taken.contains(*c))
            .cloned()
            .collect()
    }
}
