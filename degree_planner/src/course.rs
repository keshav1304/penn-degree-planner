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

pub fn read_courses(path: &str) -> Result<Vec<Course>, Error> {
    let file = File::open(path)?;
    let mut csv_reader = csv::Reader::from_reader(file);
    let mut courses = Vec::new();

    for result in csv_reader.deserialize::<Course>() {
        courses.push(result?);
    }

    Ok(courses)
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
