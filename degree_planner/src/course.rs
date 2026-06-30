use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Error};

use crate::penn_data::courses_data;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Course {
    pub dept_code: String,
    pub course_code: String,
    pub title: String,
    pub description: Option<String>,
    pub semester: Option<String>,
    pub prereq: Option<String>,
    pub cu: f64,
    pub also_offered_as: Option<String>,
    pub mutually_exclusive: Option<String>,
    pub coreq: Option<String>,
}

/// Slim course row returned by `/search_courses`.
#[derive(Debug, Clone, Serialize)]
pub struct CourseSearchHit {
    pub dept_code: String,
    pub course_code: String,
    pub title: String,
    pub cu: f64,
}

const DEFAULT_SEARCH_LIMIT: usize = 50;
const MAX_SEARCH_LIMIT: usize = 100;

/// Search the embedded catalog by course code, title, or department code.
pub fn search_courses(query: &str, limit: Option<usize>) -> Vec<CourseSearchHit> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    let limit = limit
        .unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT);

    courses_data::courses()
        .iter()
        .filter(|course| {
            course.course_code.to_lowercase().contains(&q)
                || course.title.to_lowercase().contains(&q)
                || course.dept_code.to_lowercase().contains(&q)
        })
        .take(limit)
        .map(|course| CourseSearchHit {
            dept_code: course.dept_code.clone(),
            course_code: course.course_code.clone(),
            title: course.title.clone(),
            cu: course.cu,
        })
        .collect()
}

pub fn find_course(path: &str, query: &str) -> Result<Option<Course>, Error> {
    let file = File::open(path)?;
    let mut csv_reader = csv::Reader::from_reader(file);

    for result in csv_reader.deserialize::<Course>() {
        let course = result?;

        let code = course.course_code.to_lowercase();
        let title = course.title.to_lowercase();
        let q = query.to_lowercase();

        if code.contains(&q) || title.contains(&q) {
            return Ok(Some(course).into());
        }
    }

    Ok(None)
}

pub fn is_valid_course_code(s: &str) -> bool {
    if let Some((prefix, suffix)) = s.split_once(' ') {
        let is_letters = !prefix.is_empty() && prefix.chars().all(|c| c.is_alphabetic());
        
        let is_numbers = !suffix.is_empty() && suffix.chars().all(|c| c.is_numeric());

        is_letters && is_numbers
    } else {
        false
    }
}

/// Numeric portion of a catalog course code (e.g. `CIS 5190` → 5190).
pub fn course_number(s: &str) -> Option<i32> {
    s.split_once(' ')
        .and_then(|(_, num)| num.parse().ok())
}

/// Penn convention: course numbers 5000+ are graduate level.
pub fn is_graduate_level(s: &str) -> bool {
    course_number(s).is_some_and(|n| n >= 5000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_courses_empty_query_returns_nothing() {
        assert!(search_courses("", None).is_empty());
        assert!(search_courses("   ", None).is_empty());
    }

    #[test]
    fn search_courses_matches_course_code() {
        let hits = search_courses("CIS 1200", None);
        assert!(hits.iter().any(|c| c.course_code == "CIS 1200"));
    }

    #[test]
    fn search_courses_respects_limit() {
        let hits = search_courses("MATH", Some(3));
        assert!(hits.len() <= 3);
    }
}