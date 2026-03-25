use std::collections::HashMap;

use serde::Serialize;

use crate::attributes_data;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize)]
pub enum Requirement {
    // pick 1 from possibilities
    SingleCourse {
        category: Option<String>,
        possibilities: Vec<String>    
    },

    // pi before you joinck N from possibilities
    CourseGroup {
        category: Option<String>,
        number: i32,
        possibilities: Vec<String>
    },

    // pick 1 from possibilities
    AnyOf {
        category: Option<String>,
        possibilities: Vec<Requirement>
    },

    // all the requirements
    AllOf {
        category: Option<String>,
        requirements: Vec<Requirement>
    },

    // requirement of N courses that fulfills a concentration specified in concentration section of major
    Concentration {
        category: Option<String>,
        number: i32,
        requirements: Vec<Requirement>,
    },

    // pick how many ever from a restricted set of possibilities
    Restriction {
        category: Option<String>,
        department: Option<String>,
        cu: Option<i32>,
        level: Option<i32>,
        attr: Option<Vec<String>>,
        excluding: Option<Vec<String>>,
        number: i32,
        no_school: Option<String>,
    },

    DoubleCount {
        category: Option<String>,
        double_counting_requirements: Vec<Requirement>,
        base_requirements: Vec<Requirement>
    }
}

impl Requirement {
    pub fn get_category(&self) -> String {
        match self {
            Requirement::SingleCourse { category, ..}
            | Requirement::CourseGroup { category, ..}
            | Requirement::Restriction { category, ..}
            | Requirement::Concentration { category, ..}
            | Requirement::DoubleCount { category, ..}
            | Requirement::AllOf { category, ..}
            | Requirement::AnyOf { category, ..} => category.clone().unwrap_or("".to_string()),
        }
    }

    /// Checks if the requirements are fulfilled by a vector of taken courses and returns a vector with 
    /// all the courses that do fulfill requirements
    pub fn fulfills_requirement(&self, taken: &Vec<String>, attributes: &HashMap<String, Vec<String>>) -> Option<Vec<String>> {
        match self {
            Requirement::SingleCourse { category, possibilities } => {
                for course in taken {
                    if possibilities.contains(course) {
                        return Some(vec![course.clone()]);
                    }
                }
                return None;
            },
            Requirement::CourseGroup { category, number, possibilities } => {
                let mut courses_taken_in_possibilities = Vec::new();
                for course in taken {
                    if possibilities.contains(course) {
                        courses_taken_in_possibilities.push(course.clone());
                    }
                }
                if courses_taken_in_possibilities.len() as i32 >= *number {
                    return Some(courses_taken_in_possibilities);
                } else {
                    return None;
                }
            },
            Requirement::AllOf { category, requirements } => {
                let mut taken_copy = taken.clone();
                let mut all_courses_fulfilled: Vec<String> = Vec::new();
                for req in requirements {
                    if let Some(mut courses_fulfilled) = req.fulfills_requirement(&taken_copy, attributes) {
                        taken_copy.retain(|x| !courses_fulfilled.contains(x));
                        all_courses_fulfilled.append(&mut courses_fulfilled);
                    } else {
                        return None;
                    }
                }
                return Some(all_courses_fulfilled);
            },
            Requirement::AnyOf { category, possibilities } => {
                for req in possibilities {
                    if let Some(courses_fulfilled) = req.fulfills_requirement(taken, attributes) {
                        return Some(courses_fulfilled);
                    }
                }
                return None;
            },
            Requirement::Concentration { category, number, requirements } => {
                let composite_requirement = &Requirement::AllOf { category: Some("Concentration".to_string()), requirements: requirements.clone() };
                composite_requirement.fulfills_requirement(taken, attributes)
            },
            Requirement::Restriction { category, department, cu, level, attr, excluding, no_school, number } => {
                let mut all_courses_fulfilled: Vec<String> = Vec::new();
                for course in taken {
                    if let Some((dept, course_id)) = course.split_once(' ') { 
                        let mut status = true;
                        if let Some(excluding_courses) = excluding {
                            for excluded_course in excluding_courses {
                                if excluded_course == course {
                                    status = false;
                                }
                            }
                        }
                        if let Some(school_name) = no_school {
                            let wh_dept_names = vec!["MGMT".to_string(), "MKTG".to_string(), 
                                                                    "BEPP".to_string(), "FNCE".to_string(), "STAT".to_string(),
                                                                    "OIDD".to_string(), "ACCT".to_string(), "HCMG".to_string(), 
                                                                    "LGST".to_string(), "REAL".to_string(),];
                            let seas_dept_names: Vec<String> = vec!["ESE".to_string(), "CIS".to_string(), 
                                                                    "MEAM".to_string(), "MSE".to_string(), "CBE".to_string(),
                                                                    "BE".to_string(), "EAS".to_string(), "ENGR".to_string(), 
                                                                    "ENM".to_string(), "NETS".to_string(),];
                            let cas_dept_names: Vec<String> = Vec::new();
                            let nurs_dept_names: Vec<String> = Vec::new();
                            match school_name.as_str() {
                                "WH" => {
                                    status = status && wh_dept_names.contains(&dept.to_string());
                                },
                                "SEAS" => {
                                    status = status && seas_dept_names.contains(&dept.to_string());
                                },
                                "CAS" => {
                                    status = status && cas_dept_names.contains(&dept.to_string());
                                },
                                "NURS" => {
                                    status = status && nurs_dept_names.contains(&dept.to_string());
                                },
                                _ => {unimplemented!()}
                            }
                        }
                        if let Some(department_name) = department {
                            status = status && (dept == department_name);
                        }
                        if let Some(min_level) = level {
                            status = status && (course_id.parse::<i32>().expect("Failed to parse course level") >= *min_level);
                        }
                        if let Some(attr_names) = attr {
                            let mut sub_status = false;
                            for attr_name in attr_names {
                                if let Some(courses_in_attribute) = attributes.get(attr_name) {
                                    sub_status = sub_status || (courses_in_attribute.contains(course));
                                } else {
                                    println!("{} - Invalid attribute provided in requirements!", attr_name);
                                }
                            }
                            status = status && sub_status;
                        }
                        
                        if status {
                            all_courses_fulfilled.push(course.clone());                    
                        }

                        if all_courses_fulfilled.len() as i32 == *number {
                            return Some(all_courses_fulfilled);
                        }
                    } else {
                        return None;
                    }
                }
                return None;
            },
            Requirement::DoubleCount { category, double_counting_requirements, base_requirements } => {
                let mut taken_copy = taken.clone();
                let mut all_courses_fulfilled: Vec<String> = Vec::new();
                for req in base_requirements {
                    if let Some(mut courses_fulfilled) = req.fulfills_requirement(&taken_copy, attributes) {
                        taken_copy.retain(|x| !courses_fulfilled.contains(x));
                        all_courses_fulfilled.append(&mut courses_fulfilled);
                    } else {
                        return None;
                    }
                }

                let mut all_courses_fulfilled_copy = all_courses_fulfilled.clone();
                let mut double_counting_fulfilled: Vec<String> = Vec::new();
                for req in double_counting_requirements {
                    if let Some(mut courses_fulfilled) = req.fulfills_requirement(&all_courses_fulfilled_copy, attributes) {
                        all_courses_fulfilled_copy.retain(|x| !courses_fulfilled.contains(x));
                        double_counting_fulfilled.append(&mut courses_fulfilled);
                    } else {
                        return None;
                    }
                }
                return Some(all_courses_fulfilled);
            }
        }
    }

    pub fn suggest_for_requirement(&self, taken: &Vec<String>, attributes: &HashMap<String, Vec<String>>) -> Option<Vec<String>> {
        match self {
            Requirement::SingleCourse { category, possibilities } => {
                for course in possibilities {
                    if !taken.contains(course) {
                        return Some(vec![course.clone()]);
                    }
                }
                return None;
            },
            Requirement::CourseGroup { category, number, possibilities } => {
                let mut suggested_courses = Vec::new();
                for course in possibilities {
                    if !taken.contains(course) {
                        suggested_courses.push(course.clone());
                        if suggested_courses.len() as i32 == *number {
                            return Some(suggested_courses);
                        }
                    }
                }
                return None;
            },
            Requirement::AnyOf { category, possibilities } => {
                for req in possibilities {
                    match req.suggest_for_requirement(taken, attributes) {
                        Some(val) => return Some(val),
                        None => {},
                    }
                }
                return None;
            },
            Requirement::AllOf { category, requirements } => {
                let mut suggested_courses = Vec::new();
                for req in requirements {
                    match req.suggest_for_requirement(taken, attributes) {
                        Some(mut val) => suggested_courses.append(&mut val),
                        None => return None,
                    }
                }
                return Some(suggested_courses);
            },
            Requirement::Concentration { category, number, requirements } => {
                let composite_requirement = &Requirement::AllOf { category: Some("Concentration".to_string()), requirements: requirements.clone() };
                composite_requirement.suggest_for_requirement(taken, attributes)
            },
            Requirement::Restriction { category, department, cu, level, attr, excluding, number, no_school } => {
                let mut response = format!("{} course(s)", number);
                if let Some(dept) = department {
                    response += " from ";
                    response += dept;
                }
                if let Some(min_level) = level {
                    response += " with minimum level ";
                    response += &min_level.to_string();
                }
                if let Some(attr_names) = attr {
                    response += " from attribute ";
                    response += &attr_names.join("/");
                }
                if let Some(excluded_courses) = excluding {
                    response += " excluding ";
                    response += &format!("{:?}", excluded_courses);
                }
                if let Some(no_school_name) = no_school {
                    response += " not from ";
                    response += no_school_name;
                }
                return Some(vec![response]);
            },
            Requirement::DoubleCount { category, double_counting_requirements, base_requirements } => {
                return Some(vec!["Double counting suggestions not yet implemented".to_string()]);
            },
        }
    }

    pub fn create_requirement_description(&self) -> String {
        match self {
            Requirement::SingleCourse { category, possibilities } => {
                return "".to_string();
            }, 
            Requirement::AnyOf { category, possibilities } => {
                return "".to_string();
            }, 
            Requirement::AllOf { category, requirements } => {
                return "".to_string();
            }, 
            Requirement::CourseGroup { category, number, possibilities } => {
                return "".to_string();
            }, 
            Requirement::DoubleCount { category, double_counting_requirements, base_requirements } => {
                return "".to_string();
            },
            Requirement::Restriction { category, department, cu, level, attr, excluding, number, no_school } => {
                return "".to_string();
            }, 
            Requirement::Concentration { category, number, requirements } => {
                return "".to_string();
            }, 
        }
    }
}


/// finding whether taken fulfills degree and to what extent
pub fn validate_courses_for_degree(mut requirements: Vec<Requirement>, taken: &Vec<String>) -> (Vec<MappedRequirement>, Vec<Requirement>) {
    let attributes = attributes_data::create_attributes();
    let mut fulfilled_requirements = Vec::new();
    let mut taken_mut = taken.clone();
    let mut requirements_not_fulfilled = Vec::new();

    requirements.sort();
    
    for req in requirements {
        let category_name = req.get_category();
        
        if let Some(courses_fulfilling) = req.fulfills_requirement(&taken_mut, &attributes) {
            taken_mut.retain(|x| !courses_fulfilling.contains(x));

            fulfilled_requirements.push(MappedRequirement { requirement: req, course_ids: courses_fulfilling } );
        } else {
            requirements_not_fulfilled.push(req);
        }
    }

    return (fulfilled_requirements, requirements_not_fulfilled);
}

/// suggesting courses for certain requirements
pub fn suggest_courses_for_requirements(unfulfilled_requirements: &Vec<Requirement>, taken: &Vec<String>) -> Vec<MappedRequirement> {
    let attributes = attributes_data::create_attributes();
    let mut suggested_courses = Vec::new();
    for req in unfulfilled_requirements {
        match req.suggest_for_requirement(taken, &attributes) {
            Some(val) => {
                suggested_courses.push(MappedRequirement { requirement: req.clone(), course_ids: val})
            },
            None => println!("Unable to find a course to fulfill {}", req.get_category())
        }
    }

    return suggested_courses;
}

#[derive(Debug, Serialize)]
pub struct MappedRequirement {
    pub requirement: Requirement,
    pub course_ids: Vec<String>,
}