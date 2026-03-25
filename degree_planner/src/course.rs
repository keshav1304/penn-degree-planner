use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Error};

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    pub dept_code: String,
    pub course_code: String,
    pub title: String,
    pub description: Option<String>,
    pub semester: Option<String>,
    pub prereq: Option<String>,
    pub cu: String,
    pub also_offered_as: Option<String>,
    pub mutually_exclusive: Option<String>,
    pub coreq: Option<String>,
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