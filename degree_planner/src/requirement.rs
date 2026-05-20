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

    // pick before you join N from possibilities
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
        department: Option<Vec<String>>,
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

const MAX_LISTED_COURSES: usize = 4;

fn format_truncated_list(items: &[String], prefix: &str) -> String {
    if items.is_empty() {
        return format!("{}(options not specified)", prefix);
    }
    if items.len() == 1 {
        return items[0].clone();
    }
    if items.len() <= MAX_LISTED_COURSES {
        return format!("{}{}", prefix, items.join(", "));
    }
    let shown: Vec<String> = items.iter().take(MAX_LISTED_COURSES).cloned().collect();
    let more = items.len() - MAX_LISTED_COURSES;
    format!("{}{} (+{} more)", prefix, shown.join(", "), more)
}

fn format_restriction_description(
    department: &Option<Vec<String>>,
    cu: &Option<i32>,
    level: &Option<i32>,
    attr: &Option<Vec<String>>,
    excluding: &Option<Vec<String>>,
    number: &i32,
    no_school: &Option<String>,
) -> String {
    let mut response = format!("{} course(s)", number);
    if let Some(depts) = department {
        response.push_str(" from ");
        response.push_str(&depts.join("/"));
    }
    if let Some(min_level) = level {
        response.push_str(&format!(" with minimum level {}", min_level));
    }
    if let Some(attr_names) = attr {
        response.push_str(" from attribute ");
        response.push_str(&attr_names.join("/"));
    }
    if let Some(excluded_courses) = excluding {
        response.push_str(" excluding ");
        response.push_str(&excluded_courses.join(", "));
    }
    if let Some(no_school_name) = no_school {
        response.push_str(" not from ");
        response.push_str(no_school_name);
    }
    if let Some(cu_val) = cu {
        response.push_str(&format!(" ({} CU)", cu_val));
    }
    response
}

/// Whether a catalog course code satisfies a Restriction requirement.
pub fn course_matches_restriction(
    course: &str,
    department: &Option<Vec<String>>,
    level: &Option<i32>,
    attr: &Option<Vec<String>>,
    excluding: &Option<Vec<String>>,
    no_school: &Option<String>,
    attributes: &HashMap<String, Vec<String>>,
) -> bool {
    let Some((dept, course_id)) = course.split_once(' ') else {
        return false;
    };
    if !course_id.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if let Some(excluding_courses) = excluding {
        if excluding_courses.iter().any(|ex| ex == course) {
            return false;
        }
    }
    if let Some(school_name) = no_school {
        let wh_dept_names = vec![
            "MGMT", "MKTG", "BEPP", "FNCE", "STAT", "OIDD", "ACCT", "HCMG", "LGST", "REAL",
        ];
        let seas_dept_names = vec![
            "ESE", "CIS", "MEAM", "MSE", "CBE", "BE", "EAS", "ENGR", "ENM", "NETS",
        ];
        let blocked: Vec<&str> = match school_name.as_str() {
            "WH" => wh_dept_names,
            "SEAS" => seas_dept_names,
            "CAS" | "NURS" => vec![],
            _ => return false,
        };
        if blocked.contains(&dept) {
            return false;
        }
    }
    if let Some(department_names) = department {
        if !department_names.iter().any(|d| d == dept) {
            return false;
        }
    }
    if let Some(min_level) = level {
        if course_id.parse::<i32>().unwrap_or(0) < *min_level {
            return false;
        }
    }
    if let Some(attr_names) = attr {
        let mut matches_attr = false;
        for attr_name in attr_names {
            if let Some(courses_in_attribute) = attributes.get(attr_name) {
                if courses_in_attribute.contains(&course.to_string()) {
                    matches_attr = true;
                }
            }
        }
        if !matches_attr {
            return false;
        }
    }
    true
}

pub fn filter_valid_course_ids(ids: Vec<String>) -> Vec<String> {
    ids.into_iter()
        .filter(|id| crate::course::is_valid_course_code(id))
        .collect()
}

/// Stable schedule-only identifier for an open requirement slot (not a course code).
pub fn is_requirement_slot_id(s: &str) -> bool {
    s.starts_with("req:")
}

pub fn filter_schedule_suggestion_ids(ids: Vec<String>) -> Vec<String> {
    ids.into_iter()
        .filter(|id| crate::course::is_valid_course_code(id) || is_requirement_slot_id(id))
        .collect()
}

impl Requirement {
    /// Stable id for scheduling a restriction placeholder (display via `create_requirement_description`).
    /// Find the nested requirement that owns a schedule slot id (e.g. inside AnyOf).
    pub fn find_for_slot_id<'a>(&'a self, slot_id: &str) -> Option<&'a Requirement> {
        if self.requirement_slot_id().as_deref() == Some(slot_id) {
            return Some(self);
        }
        match self {
            Requirement::AnyOf { possibilities, .. } => {
                for child in possibilities {
                    if let Some(found) = child.find_for_slot_id(slot_id) {
                        return Some(found);
                    }
                }
            }
            Requirement::AllOf { requirements, .. } | Requirement::Concentration { requirements, .. } => {
                for child in requirements {
                    if let Some(found) = child.find_for_slot_id(slot_id) {
                        return Some(found);
                    }
                }
            }
            Requirement::DoubleCount {
                base_requirements,
                double_counting_requirements,
                ..
            } => {
                for child in base_requirements
                    .iter()
                    .chain(double_counting_requirements.iter())
                {
                    if let Some(found) = child.find_for_slot_id(slot_id) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub fn slot_label_for_id(&self, slot_id: &str) -> String {
        self.find_for_slot_id(slot_id)
            .map(|r| r.create_requirement_description())
            .unwrap_or_else(|| "Open requirement".to_string())
    }

    pub fn requirement_slot_id(&self) -> Option<String> {
        match self {
            Requirement::Restriction {
                number,
                department,
                level,
                attr,
                excluding,
                no_school,
                ..
            } => {
                let dept = department
                    .as_ref()
                    .map(|d| d.join("/"))
                    .unwrap_or_default();
                let attr_s = attr.as_ref().map(|a| a.join("/")).unwrap_or_default();
                let excl = excluding
                    .as_ref()
                    .map(|e| e.join(","))
                    .unwrap_or_default();
                let lvl = level.map(|l| l.to_string()).unwrap_or_default();
                let school = no_school.clone().unwrap_or_default();
                Some(format!(
                    "req:R:{}:{}:{}:{}:{}:{}",
                    number, dept, lvl, attr_s, excl, school
                ))
            }
            _ => None,
        }
    }

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
    pub fn fulfills_requirement(&self, taken: &Vec<String>, attributes: &HashMap<String, Vec<String>>, cu_map: &HashMap<String, f64>) -> Option<Vec<String>> {
        match self {
            Requirement::SingleCourse { category, possibilities, .. } => {
                for course in taken {
                    if possibilities.contains(course) {
                        return Some(vec![course.clone()]);
                    }
                }
                return None;
            },
            Requirement::CourseGroup { category, number, possibilities, .. } => {
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
            Requirement::AllOf { category, requirements, .. } => {
                let mut taken_copy = taken.clone();
                let mut all_courses_fulfilled: Vec<String> = Vec::new();
                for req in requirements {
                    if let Some(mut courses_fulfilled) = req.fulfills_requirement(&taken_copy, attributes, cu_map) {
                        taken_copy.retain(|x| !courses_fulfilled.contains(x));
                        all_courses_fulfilled.append(&mut courses_fulfilled);
                    } else {
                        return None;
                    }
                }
                return Some(all_courses_fulfilled);
            },
            Requirement::AnyOf { category, possibilities, .. } => {
                for req in possibilities {
                    if let Some(courses_fulfilled) = req.fulfills_requirement(taken, attributes, cu_map) {
                        return Some(courses_fulfilled);
                    }
                }
                return None;
            },
            Requirement::Concentration { category, number, requirements, .. } => {
                let composite_requirement = &Requirement::AllOf { category: Some("Concentration".to_string()), requirements: requirements.clone() };
                composite_requirement.fulfills_requirement(taken, attributes, cu_map)
            },
            Requirement::Restriction { category, department, cu, level, attr, excluding, no_school, number, .. } => {
                let mut all_courses_fulfilled: Vec<String> = Vec::new();
                let mut accumulated_cu: f64 = 0.0;
                let target_cu: f64 = *number as f64; // each slot represents 1.0 CU
                for course in taken {
                    if course_matches_restriction(
                        course, department, level, attr, excluding, no_school, attributes,
                    ) {
                        let course_cu = *cu_map.get(course).unwrap_or(&1.0);
                        all_courses_fulfilled.push(course.clone());
                        accumulated_cu += course_cu;
                        if accumulated_cu >= target_cu {
                            return Some(all_courses_fulfilled);
                        }
                    }
                }
                return None;
            },
            Requirement::DoubleCount { category, double_counting_requirements, base_requirements } => {
                let mut taken_copy = taken.clone();
                let mut all_courses_fulfilled: Vec<String> = Vec::new();
                for req in base_requirements {
                    if let Some(mut courses_fulfilled) = req.fulfills_requirement(&taken_copy, attributes, cu_map) {
                        taken_copy.retain(|x| !courses_fulfilled.contains(x));
                        all_courses_fulfilled.append(&mut courses_fulfilled);
                    } else {
                        return None;
                    }
                }

                let mut all_courses_fulfilled_copy = all_courses_fulfilled.clone();
                let mut double_counting_fulfilled: Vec<String> = Vec::new();
                for req in double_counting_requirements {
                    if let Some(mut courses_fulfilled) = req.fulfills_requirement(&all_courses_fulfilled_copy, attributes, cu_map) {
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

    pub fn suggest_for_requirement(&self, taken: &Vec<String>, attributes: &HashMap<String, Vec<String>>, cu_map: &HashMap<String, f64>) -> Option<Vec<String>> {
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
                    match req.suggest_for_requirement(taken, attributes, cu_map) {
                        Some(val) => return Some(val),
                        None => {},
                    }
                }
                return None;
            },
            Requirement::AllOf { category, requirements } => {
                let mut suggested_courses = Vec::new();
                for req in requirements {
                    match req.suggest_for_requirement(taken, attributes, cu_map) {
                        Some(mut val) => suggested_courses.append(&mut val),
                        None => return None,
                    }
                }
                return Some(suggested_courses);
            },
            Requirement::Concentration { category, number, requirements } => {
                let composite_requirement = &Requirement::AllOf { category: Some("Concentration".to_string()), requirements: requirements.clone() };
                composite_requirement.suggest_for_requirement(taken, attributes, cu_map)
            },
            Requirement::Restriction { .. } => self
                .requirement_slot_id()
                .map(|slot_id| vec![slot_id]),
            Requirement::DoubleCount { category, double_counting_requirements, base_requirements } => {
                // Find which base requirements are still unfulfilled
                let mut taken_copy = taken.clone();
                let mut unfulfilled_base: Vec<&Requirement> = Vec::new();
                let mut fulfilled_base_courses: Vec<String> = Vec::new();

                for req in base_requirements {
                    if let Some(courses) = req.fulfills_requirement(&taken_copy, attributes, cu_map) {
                        taken_copy.retain(|x| !courses.contains(x));
                        fulfilled_base_courses.extend(courses);
                    } else {
                        unfulfilled_base.push(req);
                    }
                }

                // Build suggestions for unfulfilled base requirements
                let mut suggestions: Vec<String> = Vec::new();
                for req in &unfulfilled_base {
                    if let Some(s) = req.suggest_for_requirement(taken, attributes, cu_map) {
                        suggestions.extend(s);
                    }
                }

                // Double-counting info is now exposed separately via extract_double_count_info
                if suggestions.is_empty() {
                    return None;
                }
                return Some(suggestions);
            },
        }
    }

    pub fn create_requirement_description(&self) -> String {
        match self {
            Requirement::SingleCourse { possibilities, .. } => {
                format_truncated_list(possibilities, "One of: ")
            }
            Requirement::CourseGroup { number, possibilities, .. } => {
                let prefix = format!("{} of: ", number);
                format_truncated_list(possibilities, &prefix)
            }
            Requirement::Restriction {
                department,
                cu,
                level,
                attr,
                excluding,
                number,
                no_school,
                ..
            } => format_restriction_description(
                department, cu, level, attr, excluding, number, no_school,
            ),
            Requirement::AnyOf { possibilities, .. } => {
                if possibilities.len() == 1 {
                    possibilities[0].create_requirement_description()
                } else {
                    "One of the following options".to_string()
                }
            }
            Requirement::AllOf { requirements, .. } => {
                format!("Complete all {} sub-requirements", requirements.len())
            }
            Requirement::Concentration { number, .. } => {
                format!("Concentration: {} course(s)", number)
            }
            Requirement::DoubleCount {
                double_counting_requirements,
                base_requirements,
                ..
            } => {
                let base_descs: Vec<String> = base_requirements
                    .iter()
                    .map(|r| {
                        let desc = r.create_requirement_description();
                        if desc.is_empty() {
                            r.get_category()
                        } else {
                            desc
                        }
                    })
                    .collect();
                let dc_descs: Vec<String> = double_counting_requirements
                    .iter()
                    .map(|r| {
                        let desc = r.create_requirement_description();
                        if desc.is_empty() {
                            r.get_category()
                        } else {
                            desc
                        }
                    })
                    .collect();
                format!(
                    "Take: {}. ({} must also satisfy: {})",
                    base_descs.join("; "),
                    double_counting_requirements.len(),
                    dc_descs.join("; ")
                )
            }
        }
    }

    /// Returns a specificity score — lower = more specific (should be matched first).
    /// This ensures the greedy matcher processes narrow requirements before broad ones.
    pub fn specificity_score(&self) -> usize {
        match self {
            Requirement::SingleCourse { possibilities, .. } => {
                // Very specific: only a handful of exact courses
                possibilities.len()
            },
            Requirement::CourseGroup { possibilities, .. } => {
                possibilities.len()
            },
            Requirement::AllOf { requirements, .. } => {
                // Sum of children — each sub-req adds specificity
                requirements.iter().map(|r| r.specificity_score()).sum::<usize>().max(1)
            },
            Requirement::AnyOf { possibilities, .. } => {
                // As specific as the most specific option
                possibilities.iter().map(|r| r.specificity_score()).min().unwrap_or(100)
            },
            Requirement::Concentration { requirements, .. } => {
                requirements.iter().map(|r| r.specificity_score()).sum::<usize>().max(1)
            },
            Requirement::DoubleCount { base_requirements, .. } => {
                base_requirements.iter().map(|r| r.specificity_score()).sum::<usize>().max(1)
            },
            Requirement::Restriction { category, department, attr, no_school, .. } => {
                // Business Breadth is extremely broad — push to the back
                if let Some(cat) = category {
                    if cat.to_lowercase().contains("business breadth") {
                        return 500;
                    }
                    if cat.to_lowercase().contains("unrestricted") || cat.to_lowercase().contains("free elective") {
                        return 1000;
                    }
                }
                match (department.is_some(), attr.is_some(), no_school.is_some()) {
                    (true, true, _) => 10,   // dept + attr: very specific
                    (true, false, _) => 50,  // dept only
                    (false, true, _) => 50,  // attr only
                    (false, false, true) => 200, // "not from school X" — broad
                    (false, false, false) => 1000, // completely unconstrained
                }
            },
        }
    }
}


/// finding whether taken fulfills degree and to what extent
pub fn validate_courses_for_degree(mut requirements: Vec<Requirement>, taken: &Vec<String>, cu_map: &HashMap<String, f64>) -> (Vec<MappedRequirement>, Vec<Requirement>) {
    let attributes = attributes_data::create_attributes();
    let mut fulfilled_requirements = Vec::new();
    let mut taken_mut = taken.clone();
    let mut requirements_not_fulfilled = Vec::new();

    requirements.sort_by_key(|r| r.specificity_score());
    
    for req in requirements {
        let category_name = req.get_category();

        match req {
            Requirement::DoubleCount { category, double_counting_requirements, base_requirements } => {
                for base_req in base_requirements {
                    if let Some(courses_fulfilling) = base_req.fulfills_requirement(&taken_mut, &attributes, cu_map) {
                        taken_mut.retain(|x| !courses_fulfilling.contains(x));

                        fulfilled_requirements.push(MappedRequirement { requirement: base_req, course_ids: courses_fulfilling } );
                    } else {
                        requirements_not_fulfilled.push(base_req);
                    }
                }
            }
            _ => {
                if let Some(courses_fulfilling) = req.fulfills_requirement(&taken_mut, &attributes, cu_map) {
                    taken_mut.retain(|x| !courses_fulfilling.contains(x));

                    fulfilled_requirements.push(MappedRequirement { requirement: req, course_ids: courses_fulfilling } );
                } else {
                    requirements_not_fulfilled.push(req);
                }
            }
        }
    }

    return (fulfilled_requirements, requirements_not_fulfilled);
}

/// suggesting courses for certain requirements
pub fn suggest_courses_for_requirements(unfulfilled_requirements: &Vec<Requirement>, taken: &Vec<String>, cu_map: &HashMap<String, f64>) -> Vec<MappedRequirement> {
    let attributes = attributes_data::create_attributes();
    let mut suggested_courses = Vec::new();
    for req in unfulfilled_requirements {
        match req.suggest_for_requirement(taken, &attributes, cu_map) {
            Some(val) => {
                let course_ids = filter_schedule_suggestion_ids(val);
                if !course_ids.is_empty() {
                    suggested_courses.push(MappedRequirement { requirement: req.clone(), course_ids });
                }
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

#[derive(Debug, Serialize, Clone)]
pub struct DoubleCountInfo {
    pub category: String,
    pub base_courses: Vec<String>,
    pub dc_descriptions: Vec<String>,
    pub dc_fulfilled: Vec<bool>,
    pub dc_matched_courses: Vec<Vec<String>>,
}

/// Extract structured double-count metadata from a list of requirements and taken courses.


// use fulfilled_requirements
// find all the requirements that are DoubleCount requirements
// find all the base requirements that are fulfilled for that DoubleCount
// put that into base_courses
// find all the base requirements in the suggestions
// put those suggestions into base_courses
// find the double_counting requirements' descriptions and put those into dc_descriptions
// check whether dc is fulfilled for that base_req and put into dc_matched_courses
// and set dc_fulfilled to true if all are fulfilled

pub fn extract_double_count_info(requirements: &Vec<Requirement>, taken: &Vec<String>, fulfilled: &Vec<MappedRequirement>, suggested: &Vec<MappedRequirement>, cu_map: &HashMap<String, f64>) -> Vec<DoubleCountInfo> {
    let attributes = attributes_data::create_attributes();
    let mut result = Vec::new();

    for req in requirements {
        if let Requirement::DoubleCount { category, double_counting_requirements, base_requirements } = req {
            let cat_name = category.clone().unwrap_or("Double Count".to_string());

            // 1. Find which base requirement courses are fulfilled
            let mut base_courses: Vec<String> = Vec::new();
            for mapped_req in fulfilled {
                if base_requirements.contains(&mapped_req.requirement) {
                    base_courses.extend(mapped_req.course_ids.clone());
                }
            }
            for mapped_req in suggested {
                if base_requirements.contains(&mapped_req.requirement) {
                    base_courses.extend(mapped_req.course_ids.clone());
                }
            }

            // 2. Check each double-counting constraint against base courses
            let mut dc_pool = base_courses.clone();
            let mut dc_descriptions = Vec::new();
            let mut dc_fulfilled = Vec::new();
            let mut dc_matched_courses: Vec<Vec<String>> = Vec::new();

            for dc_req in double_counting_requirements {
                // Generate a human-readable description of this constraint
                let desc = dc_req.create_requirement_description();
                dc_descriptions.push(desc);

                if let Some(courses) = dc_req.fulfills_requirement(&dc_pool, &attributes, cu_map) {
                    dc_pool.retain(|x| !courses.contains(x));
                    dc_fulfilled.push(true);
                    dc_matched_courses.push(courses);
                } else {
                    dc_fulfilled.push(false);
                    dc_matched_courses.push(vec![]);
                }
            }

            result.push(DoubleCountInfo {
                category: cat_name,
                base_courses,
                dc_descriptions,
                dc_fulfilled,
                dc_matched_courses,
            });
        }
    }

    result
}

#[derive(Debug, Serialize, Clone)]
pub struct ConcentrationInfo {
    pub name: String,
    pub is_core: bool,
    pub requirements_total: usize,
    pub requirements_fulfilled: usize,
    pub requirement_descriptions: Vec<String>,
    pub requirement_fulfilled: Vec<bool>,
    pub matched_courses: Vec<Vec<String>>,
}

/// Extract concentration progress info for overlay-style concentrations.
/// For core concentrations (Requirement::Concentration in requirements), only name + is_core are populated.
pub fn extract_concentration_info(
    requirements: &Vec<Requirement>,
    concentrations: &Option<std::collections::BTreeMap<String, Vec<Requirement>>>,
    selected_concentration: &Option<String>,
    taken: &Vec<String>,
    cu_map: &HashMap<String, f64>,
) -> Vec<ConcentrationInfo> {
    let attributes = attributes_data::create_attributes();

    // Check if this major has a core concentration (Requirement::Concentration in requirements)
    let has_core = requirements.iter().any(|r| matches!(r, Requirement::Concentration { .. }));

    let conc_map = match concentrations {
        Some(map) => map,
        None => return vec![],
    };

    if has_core {
        // For core concentrations, just return the name and is_core flag.
        // The actual requirement validation is done via the normal requirement flow.
        if let Some(selected) = selected_concentration {
            if conc_map.contains_key(selected) {
                return vec![ConcentrationInfo {
                    name: selected.clone(),
                    is_core: true,
                    requirements_total: 0,
                    requirements_fulfilled: 0,
                    requirement_descriptions: vec![],
                    requirement_fulfilled: vec![],
                    matched_courses: vec![],
                }];
            }
        }
        return vec![];
    }

    // Overlay-style: evaluate the selected concentration requirements
    let selected = match selected_concentration {
        Some(s) => s,
        None => return vec![],
    };

    let conc_reqs = match conc_map.get(selected) {
        Some(reqs) => reqs,
        None => return vec![],
    };

    let mut req_descriptions = Vec::new();
    let mut req_fulfilled = Vec::new();
    let mut matched_courses: Vec<Vec<String>> = Vec::new();
    let mut remaining_taken = taken.clone();

    for req in conc_reqs {
        let desc = req.create_requirement_description();
        let desc = if desc.is_empty() {
            req.get_category()
        } else {
            desc
        };
        req_descriptions.push(desc);

        // Check fulfillment
        if let Some(courses) = req.fulfills_requirement(&remaining_taken, &attributes, cu_map) {
            remaining_taken.retain(|x| !courses.contains(x));
            req_fulfilled.push(true);
            matched_courses.push(courses);
        } else {
            req_fulfilled.push(false);
            matched_courses.push(vec![]);
        }
    }

    let fulfilled_count = req_fulfilled.iter().filter(|&&x| x).count();

    vec![ConcentrationInfo {
        name: selected.clone(),
        is_core: false,
        requirements_total: conc_reqs.len(),
        requirements_fulfilled: fulfilled_count,
        requirement_descriptions: req_descriptions,
        requirement_fulfilled: req_fulfilled,
        matched_courses,
    }]
}