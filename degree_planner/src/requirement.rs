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

fn slot_scope_slug(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Business-breadth AnyOf slots use `req:BB:{category_slug}` (one per BB block).
pub fn business_breadth_slot_id(category: &str) -> String {
    format!("req:BB:{}", slot_scope_slug(category))
}

fn scoped_slot_id(scope: Option<&str>, fingerprint: &str) -> String {
    match scope.filter(|s| !s.is_empty()) {
        Some(s) => format!("req:{}:{}", s, fingerprint),
        None => format!("req:{}", fingerprint),
    }
}

pub fn filter_schedule_suggestion_ids(ids: Vec<String>) -> Vec<String> {
    ids.into_iter()
        .filter(|id| crate::course::is_valid_course_code(id) || is_requirement_slot_id(id))
        .collect()
}

impl Requirement {
    /// Stable id for scheduling a restriction placeholder (display via `create_requirement_description`).
    /// Index of this requirement node in the major's top-level `requirements` list.
    pub fn path_in_major(major: &[Requirement], needle: &Requirement) -> Option<String> {
        for (i, req) in major.iter().enumerate() {
            if req == needle {
                return Some(i.to_string());
            }
            if let Some(p) = req.find_path_in_subtree(needle, &i.to_string()) {
                return Some(p);
            }
        }
        None
    }

    fn find_path_in_subtree<'a>(&'a self, needle: &Requirement, path: &str) -> Option<String> {
        match self {
            Requirement::AnyOf {
                category,
                possibilities,
                ..
            } => {
                let child_path = if let Some(cat) = category.as_ref().filter(|c| !c.is_empty()) {
                    format!("{}|{}", path, slot_scope_slug(cat))
                } else {
                    path.to_string()
                };
                for (j, child) in possibilities.iter().enumerate() {
                    if child == needle {
                        return Some(format!("{}#{}", child_path, j));
                    }
                    if let Some(p) = child.find_path_in_subtree(needle, &format!("{}#{}", child_path, j)) {
                        return Some(p);
                    }
                }
                if Self::is_business_breadth_category(category.as_ref())
                    && needle == self
                {
                    return Some(child_path);
                }
            }
            Requirement::AllOf { requirements, .. }
            | Requirement::Concentration { requirements, .. } => {
                for (j, child) in requirements.iter().enumerate() {
                    if child == needle {
                        return Some(format!("{}#{}", path, j));
                    }
                    if let Some(p) = child.find_path_in_subtree(needle, &format!("{}#{}", path, j)) {
                        return Some(p);
                    }
                }
            }
            Requirement::DoubleCount {
                base_requirements,
                double_counting_requirements,
                ..
            } => {
                for (j, child) in base_requirements
                    .iter()
                    .chain(double_counting_requirements.iter())
                    .enumerate()
                {
                    if child == needle {
                        return Some(format!("{}#dc{}", path, j));
                    }
                    if let Some(p) = child.find_path_in_subtree(needle, &format!("{}#dc{}", path, j)) {
                        return Some(p);
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub fn matches_slot_id(&self, slot_id: &str) -> bool {
        if slot_id.starts_with("req:BB:") {
            if let Requirement::AnyOf { category, .. } = self {
                return category
                    .as_ref()
                    .map(|c| business_breadth_slot_id(c) == slot_id)
                    .unwrap_or(false);
            }
            return false;
        }
        if let Some(rest) = slot_id.strip_prefix("req:") {
            if let Some((scope, _fp)) = rest.split_once(":R:") {
                if !scope.is_empty() {
                    return self.requirement_slot_id(Some(scope)).as_deref() == Some(slot_id);
                }
            }
        }
        self.requirement_slot_id(None).as_deref() == Some(slot_id)
    }

    /// Find the nested requirement that owns a schedule slot id (e.g. inside AnyOf).
    pub fn find_for_slot_id<'a>(&'a self, slot_id: &str) -> Option<&'a Requirement> {
        if self.matches_slot_id(slot_id) {
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

    fn is_business_breadth_category(category: Option<&String>) -> bool {
        category
            .map(|c| c.to_lowercase().contains("business breadth"))
            .unwrap_or(false)
    }

    fn business_breadth_schedule_label(category: &str) -> String {
        if category.eq_ignore_ascii_case("Business Breadth") {
            "1 WH Business Breadth".to_string()
        } else {
            format!("1 WH {}", category)
        }
    }

    /// Business breadth slots use a short schedule label instead of dept-level restriction text.
    pub fn business_breadth_label_for_slot(&self, slot_id: &str) -> Option<String> {
        match self {
            Requirement::AnyOf { category, .. } => {
                let cat = category.as_deref()?;
                if !Self::is_business_breadth_category(category.as_ref()) {
                    return None;
                }
                if business_breadth_slot_id(cat) == slot_id {
                    return Some(Self::business_breadth_schedule_label(cat));
                }
                None
            }
            Requirement::AllOf { requirements, .. } | Requirement::Concentration { requirements, .. } => {
                for child in requirements {
                    if let Some(label) = child.business_breadth_label_for_slot(slot_id) {
                        return Some(label);
                    }
                }
                None
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
                    if let Some(label) = child.business_breadth_label_for_slot(slot_id) {
                        return Some(label);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn slot_label_for_id(&self, slot_id: &str) -> String {
        if let Some(label) = self.business_breadth_label_for_slot(slot_id) {
            return label;
        }
        self.find_for_slot_id(slot_id)
            .map(|r| r.create_requirement_description())
            .unwrap_or_else(|| "Open requirement".to_string())
    }

    pub fn requirement_slot_id(&self, scope: Option<&str>) -> Option<String> {
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
                let fingerprint = format!(
                    "R:{}:{}:{}:{}:{}:{}",
                    number, dept, lvl, attr_s, excl, school
                );
                Some(scoped_slot_id(scope, &fingerprint))
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

    pub fn suggest_for_requirement(
        &self,
        taken: &Vec<String>,
        attributes: &HashMap<String, Vec<String>>,
        cu_map: &HashMap<String, f64>,
        scope: Option<&str>,
    ) -> Option<Vec<String>> {
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
                if Self::is_business_breadth_category(category.as_ref()) {
                    if let Some(cat) = category.as_deref() {
                        return Some(vec![business_breadth_slot_id(cat)]);
                    }
                }
                for req in possibilities {
                    match req.suggest_for_requirement(taken, attributes, cu_map, scope) {
                        Some(val) => return Some(val),
                        None => {},
                    }
                }
                return None;
            },
            Requirement::AllOf { category, requirements } => {
                let mut suggested_courses = Vec::new();
                for req in requirements {
                    match req.suggest_for_requirement(taken, attributes, cu_map, scope) {
                        Some(mut val) => suggested_courses.append(&mut val),
                        None => return None,
                    }
                }
                return Some(suggested_courses);
            },
            Requirement::Concentration { category, number, requirements } => {
                let composite_requirement = &Requirement::AllOf { category: Some("Concentration".to_string()), requirements: requirements.clone() };
                composite_requirement.suggest_for_requirement(taken, attributes, cu_map, scope)
            },
            Requirement::Restriction { .. } => self
                .requirement_slot_id(scope)
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
                    if let Some(s) = req.suggest_for_requirement(taken, attributes, cu_map, scope) {
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
pub fn validate_courses_for_degree(
    requirements: Vec<Requirement>,
    taken: &Vec<String>,
    cu_map: &HashMap<String, f64>,
) -> (Vec<MappedRequirement>, Vec<MappedRequirement>) {
    let attributes = attributes_data::create_attributes();
    let mut fulfilled_requirements = Vec::new();
    let mut taken_mut = taken.clone();
    let mut requirements_not_fulfilled = Vec::new();

    // Preserve original major indices before sorting — identical requirements compare
    // equal and must not share one instance id.
    let mut indexed: Vec<(usize, Requirement)> = requirements.into_iter().enumerate().collect();
    indexed.sort_by_key(|(_, r)| r.specificity_score());

    for (orig_idx, req) in indexed {
        let instance_id = Some(orig_idx.to_string());

        match req {
            Requirement::DoubleCount {
                double_counting_requirements,
                base_requirements,
                ..
            } => {
                let mut base_courses: Vec<String> = Vec::new();
                for (bi, base_req) in base_requirements.into_iter().enumerate() {
                    let child_id = Some(format!("{}:b{}", orig_idx, bi));
                    if let Some(courses_fulfilling) =
                        base_req.fulfills_requirement(&taken_mut, &attributes, cu_map)
                    {
                        taken_mut.retain(|x| !courses_fulfilling.contains(x));
                        base_courses.extend(courses_fulfilling.clone());
                        fulfilled_requirements.push(MappedRequirement {
                            requirement: base_req,
                            course_ids: courses_fulfilling,
                            instance_id: child_id.clone(),
                        });
                    } else {
                        requirements_not_fulfilled.push(MappedRequirement {
                            requirement: base_req,
                            course_ids: vec![],
                            instance_id: child_id,
                        });
                    }
                }
                for (di, dc_req) in double_counting_requirements.into_iter().enumerate() {
                    let child_id = Some(format!("{}:d{}", orig_idx, di));
                    if let Some(courses_fulfilling) =
                        dc_req.fulfills_requirement(&base_courses, &attributes, cu_map)
                    {
                        fulfilled_requirements.push(MappedRequirement {
                            requirement: dc_req,
                            course_ids: courses_fulfilling,
                            instance_id: child_id,
                        });
                    } else if !base_courses.is_empty() {
                        requirements_not_fulfilled.push(MappedRequirement {
                            requirement: dc_req,
                            course_ids: vec![],
                            instance_id: child_id,
                        });
                    }
                }
            }
            _ => {
                if let Some(courses_fulfilling) = req.fulfills_requirement(&taken_mut, &attributes, cu_map) {
                    taken_mut.retain(|x| !courses_fulfilling.contains(x));

                    fulfilled_requirements.push(MappedRequirement {
                        requirement: req,
                        course_ids: courses_fulfilling,
                        instance_id,
                    });
                } else {
                    requirements_not_fulfilled.push(MappedRequirement {
                        requirement: req,
                        course_ids: vec![],
                        instance_id,
                    });
                }
            }
        }
    }

    (fulfilled_requirements, requirements_not_fulfilled)
}

fn course_department(course_id: &str) -> Option<String> {
    course_id
        .split_whitespace()
        .next()
        .map(|d| d.to_string())
}

fn requirement_matches_concentration(req: &Requirement, conc_name: &str) -> bool {
    let cat = req.get_category().to_lowercase();
    cat.contains(&format!("concentration - {}", conc_name.to_lowercase()))
}

fn is_business_breadth_requirement(req: &Requirement) -> bool {
    req.get_category()
        .to_lowercase()
        .contains("business breadth")
}

/// When a student has two Wharton concentrations, at most one business-breadth slot
/// may also count toward one concentration requirement; other BB slots may not overlap.
pub fn apply_wharton_double_concentration_bb_overlap(
    concentrations: &[String],
    fulfilled: &mut Vec<MappedRequirement>,
    unfulfilled: &mut Vec<MappedRequirement>,
) {
    if concentrations.len() < 2 {
        return;
    }

    // Prefer a BB fulfillment whose course dept matches one of the concentrations.
    for mapped in fulfilled.iter() {
        if !is_business_breadth_requirement(&mapped.requirement) {
            continue;
        }
        for bb_course in &mapped.course_ids {
            let Some(dept) = course_department(bb_course) else {
                continue;
            };
            if !concentrations.contains(&dept) {
                continue;
            }
            if let Some(idx) = unfulfilled.iter().position(|m| {
                requirement_matches_concentration(&m.requirement, &dept)
            }) {
                let mapped = unfulfilled.remove(idx);
                fulfilled.push(MappedRequirement {
                    requirement: mapped.requirement,
                    course_ids: vec![bb_course.clone()],
                    instance_id: mapped.instance_id,
                });
                return;
            }
        }
    }

    // Concentration matched first: credit at most one unfulfilled BB slot.
    for mapped in fulfilled.clone() {
        for course in &mapped.course_ids {
            let Some(dept) = course_department(course) else {
                continue;
            };
            if !concentrations.contains(&dept) {
                continue;
            }
            if !requirement_matches_concentration(&mapped.requirement, &dept) {
                continue;
            }
            if let Some(idx) = unfulfilled
                .iter()
                .position(|m| is_business_breadth_requirement(&m.requirement))
            {
                let mapped = unfulfilled.remove(idx);
                fulfilled.push(MappedRequirement {
                    requirement: mapped.requirement,
                    course_ids: vec![course.clone()],
                    instance_id: mapped.instance_id,
                });
                return;
            }
        }
    }
}

/// suggesting courses for certain requirements
pub fn suggest_courses_for_requirements(
    unfulfilled_requirements: &[MappedRequirement],
    taken: &Vec<String>,
    cu_map: &HashMap<String, f64>,
) -> Vec<MappedRequirement> {
    let attributes = attributes_data::create_attributes();
    let mut suggested_courses = Vec::new();
    for mapped in unfulfilled_requirements {
        let scope = mapped.instance_id.as_deref();
        match mapped
            .requirement
            .suggest_for_requirement(taken, &attributes, cu_map, scope)
        {
            Some(val) => {
                let course_ids = filter_schedule_suggestion_ids(val);
                if !course_ids.is_empty() {
                    suggested_courses.push(MappedRequirement {
                        requirement: mapped.requirement.clone(),
                        course_ids,
                        instance_id: mapped.instance_id.clone(),
                    });
                }
            }
            None => println!(
                "Unable to find a course to fulfill {}",
                mapped.requirement.get_category()
            ),
        }
    }

    suggested_courses
}

#[derive(Debug, Clone, Serialize)]
pub struct MappedRequirement {
    pub requirement: Requirement,
    pub course_ids: Vec<String>,
    /// Stable per-slot identity (major index or BB category slug), not the description text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
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
    selected_concentrations: &[String],
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
        return selected_concentrations
            .iter()
            .filter(|name| conc_map.contains_key(*name))
            .map(|name| ConcentrationInfo {
                name: name.clone(),
                is_core: true,
                requirements_total: 0,
                requirements_fulfilled: 0,
                requirement_descriptions: vec![],
                requirement_fulfilled: vec![],
                matched_courses: vec![],
            })
            .collect();
    }

    // Overlay-style: evaluate each selected concentration
    let mut results = Vec::new();
    let mut remaining_taken = taken.clone();

    for selected in selected_concentrations {
        let conc_reqs = match conc_map.get(selected) {
            Some(reqs) => reqs,
            None => continue,
        };

        let mut req_descriptions = Vec::new();
        let mut req_fulfilled = Vec::new();
        let mut matched_courses: Vec<Vec<String>> = Vec::new();

        for req in conc_reqs {
            let desc = req.create_requirement_description();
            let desc = if desc.is_empty() {
                req.get_category()
            } else {
                desc
            };
            req_descriptions.push(desc);

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

        results.push(ConcentrationInfo {
            name: selected.clone(),
            is_core: false,
            requirements_total: conc_reqs.len(),
            requirements_fulfilled: fulfilled_count,
            requirement_descriptions: req_descriptions,
            requirement_fulfilled: req_fulfilled,
            matched_courses,
        });
    }

    results
}