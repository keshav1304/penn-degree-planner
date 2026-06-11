use std::collections::HashSet;

use crate::catalog_index::CatalogIndex;
use crate::requirement::{course_matches_restriction, Requirement};

/// Compiled predicate: which catalog courses can satisfy a single open slot.
#[derive(Debug, Clone)]
pub enum CourseMatcher {
    OneOf(Vec<String>),
    Restriction {
        department: Option<Vec<String>>,
        level: Option<i32>,
        attr: Option<Vec<String>>,
        excluding: Option<Vec<String>>,
        no_school: Option<String>,
    },
    AnyOf(Vec<CourseMatcher>),
    AllOf(Vec<CourseMatcher>),
    /// Pool flex / unrestricted elective — too broad to enumerate alone.
    Unrestricted,
}

impl CourseMatcher {
    pub fn is_enumerable(&self) -> bool {
        !matches!(self, CourseMatcher::Unrestricted)
    }

    pub fn specificity_score(&self) -> usize {
        match self {
            CourseMatcher::OneOf(v) => v.len().max(1),
            CourseMatcher::Restriction { attr, department, no_school, .. } => {
                if attr.as_ref().is_some_and(|a| !a.is_empty()) {
                    50
                } else if department.as_ref().is_some_and(|d| !d.is_empty()) {
                    100
                } else if no_school.is_some() {
                    500
                } else {
                    1000
                }
            }
            CourseMatcher::AnyOf(children) => children
                .iter()
                .map(|c| c.specificity_score())
                .min()
                .unwrap_or(1000),
            CourseMatcher::AllOf(children) => children
                .iter()
                .map(|c| c.specificity_score())
                .sum(),
            CourseMatcher::Unrestricted => usize::MAX,
        }
    }
}

pub fn compile_matcher(req: &Requirement, committed_branch: Option<usize>) -> CourseMatcher {
    match req {
        Requirement::SingleCourse { possibilities, .. } => {
            CourseMatcher::OneOf(possibilities.clone())
        }
        Requirement::Restriction {
            department,
            level,
            attr,
            excluding,
            no_school,
            ..
        } => CourseMatcher::Restriction {
            department: department.clone(),
            level: *level,
            attr: attr.clone(),
            excluding: excluding.clone(),
            no_school: no_school.clone(),
        },
        Requirement::AnyOf { possibilities, .. } => {
            if let Some(b) = committed_branch {
                if let Some(child) = possibilities.get(b) {
                    return compile_matcher(child, None);
                }
            }
            CourseMatcher::AnyOf(
                possibilities
                    .iter()
                    .map(|p| compile_matcher(p, None))
                    .collect(),
            )
        }
        Requirement::AllOf { requirements, .. } => CourseMatcher::AllOf(
            requirements
                .iter()
                .map(|r| compile_matcher(r, None))
                .collect(),
        ),
        Requirement::CourseGroup { possibilities, .. } => CourseMatcher::AnyOf(
            possibilities
                .iter()
                .map(|p| compile_matcher(p, None))
                .collect(),
        ),
        Requirement::Concentration { requirements, .. } => CourseMatcher::AllOf(
            requirements
                .iter()
                .map(|r| compile_matcher(r, None))
                .collect(),
        ),
        Requirement::CoursePool { .. } => CourseMatcher::Unrestricted,
    }
}

pub fn course_satisfies_matcher(
    matcher: &CourseMatcher,
    course: &str,
    attributes: &std::collections::HashMap<String, Vec<String>>,
) -> bool {
    match matcher {
        CourseMatcher::OneOf(list) => list.iter().any(|c| c == course),
        CourseMatcher::Restriction {
            department,
            level,
            attr,
            excluding,
            no_school,
        } => course_matches_restriction(
            course,
            department,
            level,
            attr,
            excluding,
            no_school,
            attributes,
        ),
        CourseMatcher::AnyOf(children) => children
            .iter()
            .any(|c| course_satisfies_matcher(c, course, attributes)),
        CourseMatcher::AllOf(children) => children
            .iter()
            .all(|c| course_satisfies_matcher(c, course, attributes)),
        CourseMatcher::Unrestricted => crate::course::is_valid_course_code(course),
    }
}

/// Finite candidate set for a matcher, or `None` when unrestricted (use peer slot's set).
pub fn candidates_for_matcher(
    matcher: &CourseMatcher,
    index: &CatalogIndex,
    taken: &HashSet<String>,
) -> Option<HashSet<String>> {
    match matcher {
        CourseMatcher::Unrestricted => None,
        CourseMatcher::OneOf(list) => Some(index.candidates_for_one_of(list, taken)),
        CourseMatcher::Restriction {
            department,
            level,
            attr,
            excluding,
            no_school,
        } => Some(index.candidates_for_restriction(
            department,
            level,
            attr,
            excluding,
            no_school,
            taken,
        )),
        CourseMatcher::AnyOf(children) => {
            let mut union = HashSet::new();
            for child in children {
                if let Some(set) = candidates_for_matcher(child, index, taken) {
                    union.extend(set);
                }
            }
            Some(union)
        }
        CourseMatcher::AllOf(children) => {
            let mut sets: Vec<HashSet<String>> = Vec::new();
            for child in children {
                let set = candidates_for_matcher(child, index, taken)?;
                sets.push(set);
            }
            if sets.is_empty() {
                return Some(HashSet::new());
            }
            let mut acc = sets[0].clone();
            for s in sets.into_iter().skip(1) {
                acc = acc.intersection(&s).cloned().collect();
            }
            Some(acc)
        }
    }
}

/// Effective candidates when pairing with another slot (unrestricted defers to the specific matcher).
pub fn effective_candidates(
    matcher: &CourseMatcher,
    index: &CatalogIndex,
    taken: &HashSet<String>,
    peer: Option<&HashSet<String>>,
) -> HashSet<String> {
    match candidates_for_matcher(matcher, index, taken) {
        Some(set) => set,
        None => peer.cloned().unwrap_or_default(),
    }
}
