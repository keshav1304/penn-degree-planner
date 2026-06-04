use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::attributes_data;
use crate::course;
use crate::cross_degree::{
    self, CrossDegreeState, CrossDegreeSummary, detect_violations, is_graduate_degree,
    crosses_undergrad_grad, UNDERGRAD_GRAD_CU_LIMIT,
};

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

const CU_EPS: f64 = 0.001;

/// Required CU for a Restriction slot. `number` is whole credits (1 → 1.0 CU).
/// When `cu` is set it overrides as tenths (5 → 0.5 CU) for half-credit slots.
fn restriction_required_cu(number: i32, cu_field: &Option<i32>) -> f64 {
    if let Some(tenths) = cu_field {
        return (*tenths as f64) / 10.0;
    }
    number as f64
}

/// Fill order: SingleCourse first, then composites (incl. DoubleCount), then Restriction last
/// (among restrictions, smaller-CU slots are matched before larger ones).
fn requirement_fill_order_key(req: &Requirement) -> (u32, u32, usize) {
    match req {
        Requirement::SingleCourse { .. } => (0, 0, req.specificity_score()),
        Requirement::Restriction { number, cu, .. } => {
            let target = restriction_required_cu(*number, cu);
            let tenths = (target * 10.0).round() as u32;
            (2, tenths, req.specificity_score())
        }
        Requirement::DoubleCount { .. } => (1, 0, req.specificity_score()),
        _ => (1, 0, req.specificity_score()),
    }
}

fn is_restriction_requirement(req: &Requirement) -> bool {
    matches!(req, Requirement::Restriction { .. })
}

fn sorted_child_requirements<'a>(requirements: &'a [Requirement]) -> Vec<&'a Requirement> {
    let mut children: Vec<&Requirement> = requirements.iter().collect();
    children.sort_by_key(|r| requirement_fill_order_key(r));
    children
}

/// Greedily assign courses from `taken` to leaf slots inside a composite (best AnyOf branch).
/// Returns assigned courses and, for AnyOf roots, the committed branch index.
fn partial_fulfill_composite(
    req: &Requirement,
    taken: &[String],
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> (Vec<String>, Option<usize>) {
    match req {
        Requirement::SingleCourse { possibilities, .. } => {
            if let Some(course) = taken.iter().find(|c| possibilities.contains(c)) {
                (vec![course.clone()], None)
            } else {
                (vec![], None)
            }
        }
        Requirement::CourseGroup {
            possibilities, number, ..
        } => {
            let need = *number as usize;
            let matched: Vec<String> = taken
                .iter()
                .filter(|c| possibilities.contains(c))
                .take(need)
                .cloned()
                .collect();
            (matched, None)
        }
        Requirement::AllOf { requirements, .. } => {
            let mut pool: Vec<String> = taken.to_vec();
            let mut assigned = Vec::new();
            for child in sorted_child_requirements(requirements) {
                let (courses, _) =
                    partial_fulfill_composite(child, &pool, attributes, cu_map);
                for course in &courses {
                    pool.retain(|c| c != course);
                }
                assigned.extend(courses);
            }
            (assigned, None)
        }
        Requirement::AnyOf { possibilities, .. } => {
            let branch_idx = select_best_anyof_branch(possibilities, taken, attributes, cu_map);
            let branch = &possibilities[branch_idx];
            let (courses, _) = partial_fulfill_composite(branch, taken, attributes, cu_map);
            (courses, Some(branch_idx))
        }
        Requirement::Concentration { requirements, .. } => {
            let composite = Requirement::AllOf {
                category: Some("Concentration".to_string()),
                requirements: requirements.clone(),
            };
            partial_fulfill_composite(&composite, taken, attributes, cu_map)
        }
        _ => (vec![], None),
    }
}

/// Pick the AnyOf branch that best matches `taken`: prefer fully satisfiable, then most leaf matches.
fn select_best_anyof_branch(
    possibilities: &[Requirement],
    taken: &[String],
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> usize {
    let mut best_idx = 0usize;
    let mut best_key = (false, 0usize, 0usize);

    for (i, branch) in possibilities.iter().enumerate() {
        let taken_vec: Vec<String> = taken.to_vec();
        let can_fully = branch
            .fulfills_requirement(&taken_vec, attributes, cu_map)
            .is_some();
        let partial_count =
            partial_fulfill_composite(branch, taken, attributes, cu_map).0.len();
        let specificity = branch.specificity_score();
        let key = (can_fully, partial_count, usize::MAX.saturating_sub(specificity));
        if key > best_key {
            best_key = key;
            best_idx = i;
        }
    }

    best_idx
}

fn lookup_course_cu(cu_map: &HashMap<String, f64>, course: &str) -> f64 {
    *cu_map.get(course).unwrap_or(&1.0)
}

fn is_half_cu(cu: f64) -> bool {
    (cu - 0.5).abs() < CU_EPS
}

fn subset_has_mixed_cu_types(courses: &[(String, f64)]) -> bool {
    let mut saw_half = false;
    let mut saw_full = false;
    for (_, cu) in courses {
        if is_half_cu(*cu) {
            saw_half = true;
        } else {
            saw_full = true;
        }
    }
    saw_half && saw_full
}

/// Pick courses that reach `target_cu` with minimal waste; prefer exact fits and
/// avoid pairing 0.5 CU with 1.0 CU when a cleaner combination exists.
fn select_courses_for_cu_target(
    eligible: Vec<(String, f64)>,
    target_cu: f64,
) -> Option<Vec<String>> {
    if target_cu <= CU_EPS {
        return Some(vec![]);
    }
    if eligible.is_empty() {
        return None;
    }

    if (target_cu - 0.5).abs() < CU_EPS {
        return eligible
            .into_iter()
            .find(|(_, cu)| is_half_cu(*cu))
            .map(|(course, _)| vec![course]);
    }

    let max_bits = eligible.len().min(14);
    let items = &eligible[..max_bits];
    let mut best: Option<(Vec<String>, f64, usize, bool)> = None;

    for mask in 1u64..(1u64 << max_bits) {
        let mut picked: Vec<(String, f64)> = Vec::new();
        let mut sum = 0.0;
        for (i, (course, cu)) in items.iter().enumerate() {
            if mask & (1u64 << i) != 0 {
                sum += cu;
                picked.push((course.clone(), *cu));
            }
        }
        if sum + CU_EPS < target_cu {
            continue;
        }
        let overage = sum - target_cu;
        let mixed = subset_has_mixed_cu_types(&picked);
        let courses: Vec<String> = picked.into_iter().map(|(c, _)| c).collect();
        let count = courses.len();

        let is_better = match &best {
            None => true,
            Some((_, best_overage, best_count, best_mixed)) => {
                overage < *best_overage - CU_EPS
                    || ((overage - *best_overage).abs() < CU_EPS && count < *best_count)
                    || ((overage - *best_overage).abs() < CU_EPS
                        && count == *best_count
                        && !mixed
                        && *best_mixed)
            }
        };
        if is_better {
            best = Some((courses, overage, count, mixed));
        }
    }

    best.map(|(courses, _, _, _)| courses)
}

/// Courses from `taken` that satisfy a Restriction by accumulated catalog CU.
/// Two 0.5 CU courses can fulfill one 1.0 CU slot; a single 0.5 CU course cannot.
pub fn courses_fulfilling_restriction_cu(
    taken: &[String],
    department: &Option<Vec<String>>,
    level: &Option<i32>,
    attr: &Option<Vec<String>>,
    excluding: &Option<Vec<String>>,
    no_school: &Option<String>,
    target_cu: f64,
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> Option<Vec<String>> {
    let eligible: Vec<(String, f64)> = taken
        .iter()
        .filter(|course| {
            course_matches_restriction(
                course, department, level, attr, excluding, no_school, attributes,
            )
        })
        .map(|course| (course.clone(), lookup_course_cu(cu_map, course)))
        .collect();

    select_courses_for_cu_target(eligible, target_cu)
}

fn format_restriction_description(
    department: &Option<Vec<String>>,
    cu: &Option<i32>,
    level: &Option<i32>,
    attr: &Option<Vec<String>>,
    _excluding: &Option<Vec<String>>,
    number: &i32,
    no_school: &Option<String>,
) -> String {
    let target = restriction_required_cu(*number, cu);
    let mut response = if (target - target.round()).abs() < CU_EPS {
        format!("{} CU", target as i32)
    } else {
        format!("{target} CU")
    };
    if let Some(depts) = department {
        response.push_str(" from ");
        response.push_str(&depts.join("/"));
    }
    if let Some(min_level) = level {
        response.push_str(&format!(" min. level {}", min_level));
    }
    if let Some(attr_names) = attr {
        response.push_str(" from attribute ");
        response.push_str(&attr_names.join("/"));
    }
    // if let Some(excluded_courses) = excluding {
    //     response.push_str(" excluding ");
    //     response.push_str(&excluded_courses.join(", "));
    // }
    if let Some(no_school_name) = no_school {
        response.push_str(" not from ");
        response.push_str(no_school_name);
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
                for req in sorted_child_requirements(requirements) {
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
                let branch_idx = select_best_anyof_branch(possibilities, taken, attributes, cu_map);
                possibilities[branch_idx].fulfills_requirement(taken, attributes, cu_map)
            },
            Requirement::Concentration { category, number, requirements, .. } => {
                let composite_requirement = &Requirement::AllOf { category: Some("Concentration".to_string()), requirements: requirements.clone() };
                composite_requirement.fulfills_requirement(taken, attributes, cu_map)
            },
            Requirement::Restriction { category, department, cu, level, attr, excluding, no_school, number, .. } => {
                courses_fulfilling_restriction_cu(
                    taken,
                    department,
                    level,
                    attr,
                    excluding,
                    no_school,
                    restriction_required_cu(*number, cu),
                    attributes,
                    cu_map,
                )
            },
            Requirement::DoubleCount { category, double_counting_requirements, base_requirements } => {
                let mut taken_copy = taken.clone();
                let mut all_courses_fulfilled: Vec<String> = Vec::new();
                for req in sorted_child_requirements(base_requirements) {
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
        cross_filter: Option<(&CrossDegreeState, usize)>,
    ) -> Option<Vec<String>> {
        match self {
            Requirement::SingleCourse { category, possibilities } => {
                for course_code in possibilities {
                    if course_suggestable(course_code, taken, cross_filter, cu_map) {
                        return Some(vec![course_code.clone()]);
                    }
                }
                return None;
            },
            Requirement::CourseGroup { category, number, possibilities } => {
                let mut suggested_courses = Vec::new();
                for course_code in possibilities {
                    if course_suggestable(course_code, taken, cross_filter, cu_map) {
                        suggested_courses.push(course_code.clone());
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
                    match req.suggest_for_requirement(taken, attributes, cu_map, scope, cross_filter) {
                        Some(val) => return Some(val),
                        None => {},
                    }
                }
                return None;
            },
            Requirement::AllOf { category, requirements } => {
                let mut suggested_courses = Vec::new();
                for req in requirements {
                    match req.suggest_for_requirement(taken, attributes, cu_map, scope, cross_filter) {
                        Some(mut val) => suggested_courses.append(&mut val),
                        None => return None,
                    }
                }
                return Some(suggested_courses);
            },
            Requirement::Concentration { category, number, requirements } => {
                let composite_requirement = &Requirement::AllOf { category: Some("Concentration".to_string()), requirements: requirements.clone() };
                composite_requirement.suggest_for_requirement(taken, attributes, cu_map, scope, cross_filter)
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
                    if let Some(s) = req.suggest_for_requirement(taken, attributes, cu_map, scope, cross_filter) {
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
                let prefix = format!("{} CU from: ", number);
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
                let parts: Vec<String> = requirements
                    .iter()
                    .map(|r| r.create_requirement_description())
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.is_empty() {
                    format!("Complete all {} sub-requirements", requirements.len())
                } else {
                    parts.join(" + ")
                }
            }
            Requirement::Concentration { number, .. } => {
                format!("Concentration: {} CU", number)
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
    indexed.sort_by(|a, b| {
        requirement_fill_order_key(&a.1).cmp(&requirement_fill_order_key(&b.1))
    });

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
                    let courses = try_fulfill_or_partial_base(
                        &base_req,
                        &mut taken_mut,
                        &attributes,
                        cu_map,
                        child_id,
                        &mut fulfilled_requirements,
                        &mut requirements_not_fulfilled,
                    );
                    base_courses.extend(courses);
                }
                for (di, dc_req) in double_counting_requirements.into_iter().enumerate() {
                    let child_id = Some(format!("{}:d{}", orig_idx, di));
                    if let Some(courses_fulfilling) =
                        dc_req.fulfills_requirement(&base_courses, &attributes, cu_map)
                    {
                        fulfilled_requirements.push(new_mapped_requirement(
                            dc_req,
                            courses_fulfilling,
                            child_id,
                            &attributes,
                        ));
                    } else if !base_courses.is_empty() {
                        requirements_not_fulfilled.push(new_mapped_requirement(
                            dc_req,
                            vec![],
                            child_id,
                            &attributes,
                        ));
                    }
                }
            }
            ref composite
                if matches!(
                    composite,
                    Requirement::AnyOf { .. }
                        | Requirement::AllOf { .. }
                        | Requirement::Concentration { .. }
                ) =>
            {
                try_fulfill_or_partial_composite(
                    &req,
                    &mut taken_mut,
                    &attributes,
                    cu_map,
                    instance_id,
                    &mut fulfilled_requirements,
                    &mut requirements_not_fulfilled,
                );
            }
            _ => {
                if let Some(courses_fulfilling) = req.fulfills_requirement(&taken_mut, &attributes, cu_map) {
                    taken_mut.retain(|x| !courses_fulfilling.contains(x));

                    fulfilled_requirements.push(new_mapped_requirement(
                        req,
                        courses_fulfilling,
                        instance_id,
                        &attributes,
                    ));
                } else if is_restriction_requirement(&req) {
                    requirements_not_fulfilled.push(new_mapped_requirement(
                        req,
                        vec![],
                        instance_id,
                        &attributes,
                    ));
                } else {
                    requirements_not_fulfilled.push(new_mapped_requirement(
                        req,
                        vec![],
                        instance_id,
                        &attributes,
                    ));
                }
            }
        }
    }

    (fulfilled_requirements, requirements_not_fulfilled)
}

/// suggesting courses for certain requirements
pub fn suggest_courses_for_requirements(
    unfulfilled_requirements: &[MappedRequirement],
    taken: &Vec<String>,
    cu_map: &HashMap<String, f64>,
    cross_state: Option<&CrossDegreeState>,
    degree_idx: Option<usize>,
) -> Vec<MappedRequirement> {
    let attributes = attributes_data::create_attributes();
    let cross_filter = cross_state
        .zip(degree_idx)
        .map(|(state, idx)| (state, idx));
    let mut suggested_courses = Vec::new();
    for mapped in unfulfilled_requirements {
        let scope = mapped.instance_id.as_deref();
        match mapped
            .requirement
            .suggest_for_requirement(taken, &attributes, cu_map, scope, cross_filter)
        {
            Some(val) => {
                let course_ids = filter_schedule_suggestion_ids(val);
                if !course_ids.is_empty() {
                    suggested_courses.push(new_mapped_requirement(
                        mapped.requirement.clone(),
                        course_ids,
                        mapped.instance_id.clone(),
                        &attributes,
                    ));
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
pub struct AttributeFulfillment {
    pub attribute: String,
    pub course_ids: Vec<String>,
}

fn attribute_fulfillment_for_requirement(
    requirement: &Requirement,
    course_ids: &[String],
    attributes: &HashMap<String, Vec<String>>,
) -> Option<Vec<AttributeFulfillment>> {
    let Requirement::Restriction {
        department,
        level,
        attr,
        excluding,
        no_school,
        ..
    } = requirement
    else {
        return None;
    };
    let attr_names = attr.as_ref().filter(|names| !names.is_empty())?;
    let mut fulfillments = Vec::new();
    for attr_name in attr_names {
        let single_attr = Some(vec![attr_name.clone()]);
        let courses: Vec<String> = course_ids
            .iter()
            .filter(|course| {
                course_matches_restriction(
                    course,
                    department,
                    level,
                    &single_attr,
                    excluding,
                    no_school,
                    attributes,
                )
            })
            .cloned()
            .collect();
        if !courses.is_empty() {
            fulfillments.push(AttributeFulfillment {
                attribute: attr_name.clone(),
                course_ids: courses,
            });
        }
    }
    if fulfillments.is_empty() {
        None
    } else {
        Some(fulfillments)
    }
}

fn new_mapped_requirement(
    requirement: Requirement,
    course_ids: Vec<String>,
    instance_id: Option<String>,
    attributes: &HashMap<String, Vec<String>>,
) -> MappedRequirement {
    new_mapped_requirement_with_options(
        requirement,
        course_ids,
        instance_id,
        attributes,
        false,
        None,
    )
}

fn new_mapped_requirement_with_options(
    requirement: Requirement,
    course_ids: Vec<String>,
    instance_id: Option<String>,
    attributes: &HashMap<String, Vec<String>>,
    partial: bool,
    committed_anyof_branch: Option<usize>,
) -> MappedRequirement {
    let attribute_fulfillment =
        attribute_fulfillment_for_requirement(&requirement, &course_ids, attributes);
    MappedRequirement {
        requirement,
        course_ids,
        instance_id,
        attribute_fulfillment,
        partial,
        committed_anyof_branch,
    }
}

fn try_fulfill_or_partial_composite(
    req: &Requirement,
    taken: &mut Vec<String>,
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
    instance_id: Option<String>,
    fulfilled: &mut Vec<MappedRequirement>,
    unfulfilled: &mut Vec<MappedRequirement>,
) {
    if let Some(courses_fulfilling) = req.fulfills_requirement(taken, attributes, cu_map) {
        taken.retain(|x| !courses_fulfilling.contains(x));
        fulfilled.push(new_mapped_requirement(
            req.clone(),
            courses_fulfilling,
            instance_id,
            attributes,
        ));
        return;
    }

    let (partial_courses, branch) =
        partial_fulfill_composite(req, taken, attributes, cu_map);
    if partial_courses.is_empty() {
        unfulfilled.push(new_mapped_requirement(
            req.clone(),
            vec![],
            instance_id,
            attributes,
        ));
        return;
    }

    taken.retain(|x| !partial_courses.contains(x));
    unfulfilled.push(new_mapped_requirement_with_options(
        req.clone(),
        partial_courses,
        instance_id,
        attributes,
        true,
        branch,
    ));
}

fn try_fulfill_or_partial_base(
    base_req: &Requirement,
    taken: &mut Vec<String>,
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
    child_id: Option<String>,
    fulfilled: &mut Vec<MappedRequirement>,
    unfulfilled: &mut Vec<MappedRequirement>,
) -> Vec<String> {
    if let Some(courses_fulfilling) = base_req.fulfills_requirement(taken, attributes, cu_map) {
        taken.retain(|x| !courses_fulfilling.contains(x));
        let courses = courses_fulfilling.clone();
        fulfilled.push(new_mapped_requirement(
            base_req.clone(),
            courses_fulfilling,
            child_id,
            attributes,
        ));
        return courses;
    }

    let (partial_courses, branch) =
        partial_fulfill_composite(base_req, taken, attributes, cu_map);
    if partial_courses.is_empty() {
        unfulfilled.push(new_mapped_requirement(
            base_req.clone(),
            vec![],
            child_id,
            attributes,
        ));
        return vec![];
    }

    taken.retain(|x| !partial_courses.contains(x));
    let courses = partial_courses.clone();
    unfulfilled.push(new_mapped_requirement_with_options(
        base_req.clone(),
        partial_courses,
        child_id,
        attributes,
        true,
        branch,
    ));
    courses
}

#[derive(Debug, Clone, Serialize)]
pub struct MappedRequirement {
    pub requirement: Requirement,
    pub course_ids: Vec<String>,
    /// Stable per-slot identity (major index or BB category slug), not the description text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// For attribute-based restrictions: which attribute(s) were satisfied and by which courses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_fulfillment: Option<Vec<AttributeFulfillment>>,
    /// True when some courses are assigned but the requirement is not fully satisfied.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
    /// For partially fulfilled AnyOf: index of the committed branch in `possibilities`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committed_anyof_branch: Option<usize>,
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
    let has_core = requirements.iter().any(|r| matches!(r, Requirement::Concentration { .. }))
        || requirements
            .iter()
            .any(|r| r.get_category() == "Concentration");

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

fn course_suggestable(
    course_code: &str,
    taken: &[String],
    cross_filter: Option<(&CrossDegreeState, usize)>,
    cu_map: &HashMap<String, f64>,
) -> bool {
    if taken.contains(&course_code.to_string()) {
        return false;
    }
    if let Some((state, degree_idx)) = cross_filter {
        if course::is_valid_course_code(course_code) {
            return state.can_claim(course_code, degree_idx, cu_map).is_ok();
        }
    }
    true
}

fn fulfillment_score_for_course_in_degree(
    per_degree: &[(Vec<MappedRequirement>, Vec<MappedRequirement>)],
    course: &str,
    degree_idx: usize,
) -> usize {
    per_degree[degree_idx]
        .0
        .iter()
        .filter(|m| m.course_ids.contains(&course.to_string()))
        .count()
}

fn choose_best_two_degrees(
    course: &str,
    degree_indices: &[usize],
    per_degree: &[(Vec<MappedRequirement>, Vec<MappedRequirement>)],
) -> HashSet<usize> {
    if degree_indices.len() <= 2 {
        return degree_indices.iter().copied().collect();
    }

    let mut best: Option<(HashSet<usize>, usize)> = None;
    for i in 0..degree_indices.len() {
        for j in (i + 1)..degree_indices.len() {
            let pair = HashSet::from([degree_indices[i], degree_indices[j]]);
            let score = fulfillment_score_for_course_in_degree(per_degree, course, degree_indices[i])
                + fulfillment_score_for_course_in_degree(per_degree, course, degree_indices[j]);
            let replace = match &best {
                None => true,
                Some((_, best_score)) => score > *best_score,
            };
            if replace {
                best = Some((pair, score));
            }
        }
    }
    best.map(|(pair, _)| pair).unwrap_or_default()
}

fn remove_course_from_degree_result(
    per_degree: &mut [(Vec<MappedRequirement>, Vec<MappedRequirement>)],
    course: &str,
    degree_idx: usize,
) {
    let (fulfilled, unfulfilled) = &mut per_degree[degree_idx];
    let mut to_unfulfill = Vec::new();

    fulfilled.retain_mut(|mapped| {
        if mapped.course_ids.contains(&course.to_string()) {
            mapped.course_ids.retain(|c| c != course);
            if mapped.course_ids.is_empty() {
                to_unfulfill.push(mapped.clone());
                return false;
            }
        }
        true
    });

    for mapped in unfulfilled.iter_mut() {
        if mapped.partial && mapped.course_ids.contains(&course.to_string()) {
            mapped.course_ids.retain(|c| c != course);
            if mapped.course_ids.is_empty() {
                mapped.partial = false;
                mapped.committed_anyof_branch = None;
            }
        }
    }

    for mapped in to_unfulfill {
        if !unfulfilled
            .iter()
            .any(|u| u.instance_id == mapped.instance_id)
        {
            unfulfilled.push(mapped);
        }
    }
}

pub fn filter_mapped_requirements_by_allocation(
    mapped_list: &mut [MappedRequirement],
    degree_idx: usize,
    claims: &HashMap<String, HashSet<usize>>,
) {
    for mapped in mapped_list {
        mapped.course_ids.retain(|course_id| {
            if !course::is_valid_course_code(course_id) {
                return true;
            }
            claims
                .get(course_id)
                .map(|indices| indices.contains(&degree_idx))
                .unwrap_or(false)
        });
    }
}

pub fn build_allocations_from_fulfilled(
    per_degree: &[(Vec<MappedRequirement>, Vec<MappedRequirement>)],
) -> HashMap<String, HashSet<usize>> {
    let mut allocations: HashMap<String, HashSet<usize>> = HashMap::new();
    for (degree_idx, (fulfilled, unfulfilled)) in per_degree.iter().enumerate() {
        let mut record = |course: &String| {
            if course::is_valid_course_code(course) {
                allocations
                    .entry(course.clone())
                    .or_default()
                    .insert(degree_idx);
            }
        };
        for mapped in fulfilled {
            for course in &mapped.course_ids {
                record(course);
            }
        }
        for mapped in unfulfilled {
            if mapped.partial {
                for course in &mapped.course_ids {
                    record(course);
                }
            }
        }
    }
    allocations
}

pub fn resolve_cross_degree_conflicts(
    per_degree: &mut [(Vec<MappedRequirement>, Vec<MappedRequirement>)],
    degree_schools: &[String],
    degree_majors: &[String],
    cu_map: &HashMap<String, f64>,
) -> CrossDegreeSummary {
    let mut allocations = build_allocations_from_fulfilled(per_degree);

    loop {
        let violations = detect_violations(&allocations, degree_schools, cu_map);
        if violations.is_empty() {
            break;
        }

        let mut changed = false;

        for violation in &violations {
            match violation.kind {
                cross_degree::CrossDegreeViolationKind::TooManyDegrees => {
                    let course = &violation.course_id;
                    let indices: Vec<usize> = allocations
                        .get(course)
                        .map(|s| s.iter().copied().collect())
                        .unwrap_or_default();
                    let keep = choose_best_two_degrees(course, &indices, per_degree);
                    for idx in indices {
                        if !keep.contains(&idx) {
                            remove_course_from_degree_result(per_degree, course, idx);
                            changed = true;
                        }
                    }
                    if let Some(set) = allocations.get_mut(course) {
                        set.retain(|idx| keep.contains(idx));
                    }
                }
                cross_degree::CrossDegreeViolationKind::GradGradOverlap => {
                    let course = &violation.course_id;
                    let grad_indices = &violation.degree_indices;
                    if grad_indices.len() <= 1 {
                        continue;
                    }
                    let mut best_idx = grad_indices[0];
                    let mut best_score =
                        fulfillment_score_for_course_in_degree(per_degree, course, best_idx);
                    for &idx in &grad_indices[1..] {
                        let score =
                            fulfillment_score_for_course_in_degree(per_degree, course, idx);
                        if score > best_score {
                            best_score = score;
                            best_idx = idx;
                        }
                    }
                    for &idx in grad_indices {
                        if idx != best_idx {
                            remove_course_from_degree_result(per_degree, course, idx);
                            changed = true;
                        }
                    }
                    if let Some(set) = allocations.get_mut(course) {
                        let to_remove: Vec<usize> = set
                            .iter()
                            .copied()
                            .filter(|&idx| {
                                is_graduate_degree(&degree_schools[idx]) && idx != best_idx
                            })
                            .collect();
                        for idx in to_remove {
                            set.remove(&idx);
                        }
                    }
                }
                cross_degree::CrossDegreeViolationKind::UndergradGradCuCap => {
                    let mut shared: Vec<(String, f64, Vec<usize>)> = allocations
                        .iter()
                        .filter(|(course, indices)| {
                            course::is_valid_course_code(course)
                                && !course::is_graduate_level(course)
                                && crosses_undergrad_grad(course, indices, degree_schools)
                        })
                        .map(|(course, indices)| {
                            (
                                course.clone(),
                                lookup_course_cu(cu_map, course),
                                indices.iter().copied().collect(),
                            )
                        })
                        .collect();

                    shared.sort_by(|a, b| {
                        b.1.partial_cmp(&a.1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then_with(|| a.0.cmp(&b.0))
                    });

                    let mut used = shared.iter().map(|(_, cu, _)| cu).sum::<f64>();
                    while used > UNDERGRAD_GRAD_CU_LIMIT + CU_EPS {
                        let Some((course, cu, indices)) = shared.pop() else {
                            break;
                        };
                        let grad_indices: Vec<usize> = indices
                            .iter()
                            .copied()
                            .filter(|&i| is_graduate_degree(&degree_schools[i]))
                            .collect();
                        for idx in grad_indices {
                            remove_course_from_degree_result(per_degree, &course, idx);
                            if let Some(set) = allocations.get_mut(&course) {
                                set.remove(&idx);
                            }
                            changed = true;
                        }
                        used -= cu;
                    }
                }
            }
        }

        if !changed {
            break;
        }
        allocations = build_allocations_from_fulfilled(per_degree);
    }

    let mut state = CrossDegreeState::new(degree_schools.to_vec(), degree_majors.to_vec());
    state.rebuild_from_allocations(&allocations, cu_map);
    state.violations = detect_violations(&allocations, degree_schools, cu_map);
    state.to_summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_cu_restriction() -> Requirement {
        Requirement::Restriction {
            category: Some("Test restriction".to_string()),
            department: Some(vec!["TEST".to_string()]),
            cu: None,
            level: None,
            attr: None,
            number: 1,
            excluding: None,
            no_school: None,
        }
    }

    #[test]
    fn restriction_single_half_cu_does_not_fulfill_one_cu_slot() {
        let attributes = attributes_data::create_attributes();
        let mut cu_map = HashMap::new();
        cu_map.insert("TEST 1000".to_string(), 0.5);

        let taken = vec!["TEST 1000".to_string()];
        let req = one_cu_restriction();

        assert!(req
            .fulfills_requirement(&taken, &attributes, &cu_map)
            .is_none());
    }

    #[test]
    fn restriction_prefers_one_full_cu_over_half_plus_full() {
        let attributes = attributes_data::create_attributes();
        let mut cu_map = HashMap::new();
        cu_map.insert("TEST 1000".to_string(), 0.5);
        cu_map.insert("TEST 1001".to_string(), 1.0);
        cu_map.insert("TEST 1002".to_string(), 0.5);

        let taken = vec![
            "TEST 1000".to_string(),
            "TEST 1001".to_string(),
            "TEST 1002".to_string(),
        ];
        let req = one_cu_restriction();

        let fulfilled = req
            .fulfills_requirement(&taken, &attributes, &cu_map)
            .expect("1 CU slot should be filled by the 1 CU course alone");
        assert_eq!(fulfilled, vec!["TEST 1001".to_string()]);
    }

    #[test]
    fn restriction_half_cu_slot_uses_half_course_first() {
        let attributes = attributes_data::create_attributes();
        let mut cu_map = HashMap::new();
        cu_map.insert("TEST 1000".to_string(), 0.5);
        cu_map.insert("TEST 1001".to_string(), 1.0);

        let taken = vec!["TEST 1000".to_string(), "TEST 1001".to_string()];
        let req = Requirement::Restriction {
            category: Some("Half CU slot".to_string()),
            department: Some(vec!["TEST".to_string()]),
            cu: Some(5),
            level: None,
            attr: None,
            number: 1,
            excluding: None,
            no_school: None,
        };

        let fulfilled = req
            .fulfills_requirement(&taken, &attributes, &cu_map)
            .expect("0.5 CU slot should use the half-credit course");
        assert_eq!(fulfilled, vec!["TEST 1000".to_string()]);
    }

    #[test]
    fn validate_degree_fills_single_course_before_restriction() {
        let mut cu_map = HashMap::new();
        cu_map.insert("CIS 1200".to_string(), 1.0);

        let requirements = vec![
            Requirement::Restriction {
                category: Some("CIS elective".to_string()),
                department: Some(vec!["CIS".to_string()]),
                cu: None,
                level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::SingleCourse {
                category: Some("Required".to_string()),
                possibilities: vec!["CIS 1200".to_string()],
            },
        ];

        let taken = vec!["CIS 1200".to_string()];
        let (fulfilled, unfulfilled) =
            validate_courses_for_degree(requirements, &taken, &cu_map);

        assert_eq!(unfulfilled.len(), 1);
        assert_eq!(fulfilled.len(), 1);
        assert_eq!(fulfilled[0].course_ids, vec!["CIS 1200".to_string()]);
        assert!(matches!(
            fulfilled[0].requirement,
            Requirement::SingleCourse { .. }
        ));
    }

    #[test]
    fn validate_degree_fills_half_slot_before_one_cu_slot() {
        let attributes = attributes_data::create_attributes();
        let mut cu_map = HashMap::new();
        cu_map.insert("TEST 1000".to_string(), 0.5);
        cu_map.insert("TEST 1001".to_string(), 1.0);

        let requirements = vec![
            Requirement::Restriction {
                category: Some("One CU".to_string()),
                department: Some(vec!["TEST".to_string()]),
                cu: None,
                level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("Half CU".to_string()),
                department: Some(vec!["TEST".to_string()]),
                cu: Some(5),
                level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
        ];

        let taken = vec!["TEST 1000".to_string(), "TEST 1001".to_string()];
        let (fulfilled, unfulfilled) =
            validate_courses_for_degree(requirements, &taken, &cu_map);

        assert_eq!(unfulfilled.len(), 0);
        assert_eq!(fulfilled.len(), 2);

        let half = fulfilled
            .iter()
            .find(|m| m.requirement.get_category() == "Half CU")
            .expect("half CU requirement fulfilled");
        assert_eq!(half.course_ids, vec!["TEST 1000".to_string()]);

        let full = fulfilled
            .iter()
            .find(|m| m.requirement.get_category() == "One CU")
            .expect("one CU requirement fulfilled");
        assert_eq!(full.course_ids, vec!["TEST 1001".to_string()]);
    }

    #[test]
    fn restriction_two_half_cu_courses_fulfill_one_cu_slot() {
        let attributes = attributes_data::create_attributes();
        let mut cu_map = HashMap::new();
        cu_map.insert("TEST 1000".to_string(), 0.5);
        cu_map.insert("TEST 1001".to_string(), 0.5);

        let taken = vec!["TEST 1000".to_string(), "TEST 1001".to_string()];
        let req = one_cu_restriction();

        let fulfilled = req
            .fulfills_requirement(&taken, &attributes, &cu_map)
            .expect("two 0.5 CU courses should satisfy a 1 CU restriction");
        assert_eq!(
            fulfilled,
            vec!["TEST 1000".to_string(), "TEST 1001".to_string()]
        );
    }

    #[test]
    fn allof_description_joins_single_course_or_groups() {
        let req = Requirement::AllOf {
            category: None,
            requirements: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["MEAM 1100".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec![
                        "MEAM 1470".to_string(),
                        "BIOL 1124".to_string(),
                        "PHYS 0050".to_string(),
                        "CHEM 1101".to_string(),
                    ],
                },
            ],
        };

        assert_eq!(
            req.create_requirement_description(),
            "MEAM 1100 + One of: MEAM 1470, BIOL 1124, PHYS 0050, CHEM 1101"
        );
    }

    #[test]
    fn single_course_fulfills_with_half_cu_course() {
        let attributes = attributes_data::create_attributes();
        let mut cu_map = HashMap::new();
        cu_map.insert("TEST 1000".to_string(), 0.5);

        let taken = vec!["TEST 1000".to_string()];
        let req = Requirement::SingleCourse {
            category: None,
            possibilities: vec!["TEST 1000".to_string()],
        };

        assert_eq!(
            req.fulfills_requirement(&taken, &attributes, &cu_map),
            Some(vec!["TEST 1000".to_string()])
        );
    }

    #[test]
    fn validate_nested_single_course_reserved_before_restriction() {
        use crate::seas_grad_data;

        let major = seas_grad_data::create_ms_robo_major();
        let taken = vec!["CIS 5190".to_string()];
        let cu_map = HashMap::from([("CIS 5190".to_string(), 1.0)]);

        let (fulfilled, unfulfilled) =
            validate_courses_for_degree(major.requirements, &taken, &cu_map);

        let restriction_with_cis = fulfilled.iter().any(|m| {
            matches!(m.requirement, Requirement::Restriction { .. })
                && m.course_ids.contains(&"CIS 5190".to_string())
        });
        assert!(
            !restriction_with_cis,
            "CIS 5190 should not be allocated to a Restriction when it matches a foundational SingleCourse slot"
        );

        let foundational = unfulfilled
            .iter()
            .find(|m| m.requirement.get_category() == "Foundational Courses")
            .expect("foundational AnyOf should be present");
        assert!(foundational.partial);
        assert_eq!(foundational.course_ids, vec!["CIS 5190".to_string()]);
        assert!(foundational.committed_anyof_branch.is_some());
    }

    #[test]
    fn validate_allof_partial_single_course_not_stolen_by_restriction() {
        let mut cu_map = HashMap::new();
        cu_map.insert("MEAM 1100".to_string(), 1.0);
        cu_map.insert("CIS 4000".to_string(), 1.0);

        let requirements = vec![
            Requirement::AnyOf {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec![
                    Requirement::SingleCourse {
                        category: None,
                        possibilities: vec!["PHYS 0150".to_string()],
                    },
                    Requirement::AllOf {
                        category: None,
                        requirements: vec![
                            Requirement::SingleCourse {
                                category: None,
                                possibilities: vec!["MEAM 1100".to_string()],
                            },
                            Requirement::SingleCourse {
                                category: None,
                                possibilities: vec!["MEAM 1470".to_string()],
                            },
                        ],
                    },
                ],
            },
            Requirement::Restriction {
                category: Some("Technical Elective".to_string()),
                department: Some(vec!["CIS".to_string()]),
                cu: None,
                level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
        ];

        let taken = vec!["MEAM 1100".to_string(), "CIS 4000".to_string()];
        let (fulfilled, unfulfilled) =
            validate_courses_for_degree(requirements, &taken, &cu_map);

        let restriction_with_meam = fulfilled.iter().any(|m| {
            matches!(m.requirement, Requirement::Restriction { .. })
                && m.course_ids.contains(&"MEAM 1100".to_string())
        });
        assert!(!restriction_with_meam);

        let math_anyof = unfulfilled
            .iter()
            .find(|m| m.requirement.get_category() == "Math and Natural Science")
            .expect("AnyOf should be unfulfilled with partial progress");
        assert!(math_anyof.partial);
        assert_eq!(math_anyof.course_ids, vec!["MEAM 1100".to_string()]);

        let tech = fulfilled
            .iter()
            .find(|m| m.requirement.get_category() == "Technical Elective")
            .expect("CIS course should fill restriction");
        assert_eq!(tech.course_ids, vec!["CIS 4000".to_string()]);
    }

    #[test]
    fn resolve_keeps_best_two_degrees() {
        let schools = vec![
            "SEAS".to_string(),
            "WH".to_string(),
            "SEAS_MS".to_string(),
        ];
        let majors = vec!["CIS".to_string(), "WH_FL".to_string(), "MS_ROBO".to_string()];
        let cu_map = HashMap::from([("CIS 1200".to_string(), 1.0)]);

        let mapped = |course: &str| MappedRequirement {
            requirement: Requirement::SingleCourse {
                category: None,
                possibilities: vec![course.to_string()],
            },
            course_ids: vec![course.to_string()],
            instance_id: Some("0".to_string()),
            attribute_fulfillment: None,
            partial: false,
            committed_anyof_branch: None,
        };

        let mut per_degree = vec![
            (vec![mapped("CIS 1200"), mapped("CIS 1200")], vec![]),
            (vec![mapped("CIS 1200")], vec![]),
            (vec![mapped("CIS 1200")], vec![]),
        ];

        let summary = resolve_cross_degree_conflicts(&mut per_degree, &schools, &majors, &cu_map);
        let kept = summary
            .course_allocations
            .get("CIS 1200")
            .map(|v| v.len())
            .unwrap_or(0);
        assert!(kept <= 2);
        assert!(summary.violations.is_empty());
    }
}