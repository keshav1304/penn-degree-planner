use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;

use crate::attributes_data;
use crate::course;
use crate::cross_degree::{
    self, CrossDegreeState, CrossDegreeSummary, detect_violations, is_graduate_degree,
    crosses_undergrad_grad, UNDERGRAD_GRAD_CU_LIMIT,
};

/// A node in a degree's requirement tree. Variants compose via nesting (`AllOf`, `AnyOf`,
/// `CourseGroup`) or match individual courses (`SingleCourse`, `Restriction`).
///
/// `category`, when set, groups related slots in the requirements panel (e.g. "Foundational Courses").
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize)]
pub enum Requirement {
    /// **Pick one course** from a list of acceptable alternatives (OR within a single slot).
    ///
    /// Example: "One of: CIS 5190, CIS 5200, CIS 5210"
    ///
    /// - `category` — panel grouping label; on nested children, often the area name
    ///   (e.g. "Artificial Intelligence").
    /// - `possibilities` — course codes that satisfy this slot; exactly one is needed.
    SingleCourse {
        category: Option<String>,
        possibilities: Vec<String>,
    },

    /// **Complete N of M child areas** — each child is typically a `SingleCourse` representing
    /// one topical area; the student must fully satisfy `number` of them.
    ///
    /// Example: Robotics MSE foundational courses — 1 course from 3 of 4 areas.
    ///
    /// - `category` — panel grouping label for the whole group.
    /// - `number` — how many child areas must be satisfied (e.g. `3`).
    /// - `possibilities` — the M area requirements to choose from (e.g. 4 `SingleCourse` areas).
    CourseGroup {
        category: Option<String>,
        number: i32,
        possibilities: Vec<Requirement>,
    },

    /// **Pick one branch** from several alternative requirement paths (OR at the composite level).
    ///
    /// Example: "Take PHYS 0150" **or** "Take MEAM 1100 and MEAM 1470".
    /// Each entry in `possibilities` is a complete alternative (often an `AllOf`).
    ///
    /// Partial fulfillment commits to the best-matching branch (`committed_anyof_branch`).
    ///
    /// - `category` — panel grouping label.
    /// - `possibilities` — mutually exclusive alternative requirement subtrees.
    AnyOf {
        category: Option<String>,
        possibilities: Vec<Requirement>,
    },

    /// **Complete every child** requirement (AND semantics).
    ///
    /// Example: "Take MEAM 1100 and one of MEAM 1470, BIOL 1124, PHYS 0050".
    ///
    /// - `category` — panel grouping label.
    /// - `requirements` — child requirements that must all be satisfied.
    AllOf {
        category: Option<String>,
        requirements: Vec<Requirement>,
    },

    /// **Concentration block** — like `AllOf`, but tied to a student's chosen concentration.
    ///
    /// The `number` field records the concentration's credit requirement; child `requirements`
    /// are the specific courses/electives for that concentration path.
    ///
    /// - `category` — usually "Concentration" or the concentration name.
    /// - `number` — required CU for the concentration.
    /// - `requirements` — courses and sub-requirements within the concentration.
    Concentration {
        category: Option<String>,
        number: i32,
        requirements: Vec<Requirement>,
    },

    /// **Flexible elective slot** matched by course attributes rather than a fixed course list.
    ///
    /// Example: "1 CU from CIS/ESE/MEAM at min. level 5000" or "1 CU from attribute EMRT".
    ///
    /// - `category` — panel grouping label (e.g. "Technical Elective").
    /// - `department` — allowed departments (e.g. `["CIS", "ESE"]`); `None` = any.
    /// - `cu` — optional override in tenths (e.g. `5` → 0.5 CU); when `None`, `number` is whole CUs.
    /// - `level` — minimum course number (e.g. `5000` for graduate-level).
    /// - `attr` — course-attribute tags the course must carry (e.g. `["EMRT"]`).
    /// - `excluding` — course codes and/or attribute codes that cannot count toward this slot.
    /// - `number` — when `cu` is `None`, duplicate this slot `number` times at load time
    ///   (each copy requires 1 CU). When `cu` is set, `number` is ignored.
    /// - `no_school` — exclude courses from a given school.
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

    /// **Shared course pool** — fixed + flexible slots form a bucket; coverage constraints
    /// must be satisfied by courses in that bucket. A course may count toward at most two
    /// coverage constraints in the pool (double-counting); consumption groups still block
    /// reuse within the same group.
    CoursePool {
        category: Option<String>,
        /// Non-fungible slots (e.g. ECON 0100, specific major courses).
        fixed_slots: Vec<Requirement>,
        /// Number of generic 1-CU pool placeholders.
        flexible_slots: i32,
        /// Labeled coverage rules evaluated against the whole pool.
        constraints: Vec<PoolConstraint>,
    },
}

/// One coverage unit inside a [`Requirement::CoursePool`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct PoolConstraint {
    pub requirement: Requirement,
    pub count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumption_group: Option<String>,
}

const MAX_LISTED_COURSES: usize = 4;

/// Max coverage constraints a single pool course may satisfy (double-count limit).
const POOL_MAX_CONSTRAINT_USES_PER_COURSE: usize = 2;

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

/// Required CU for a Restriction slot. After [`expand_restriction_slots`], `number` is 1 when
/// `cu` is `None`. When `cu` is set it overrides as tenths (5 → 0.5 CU) for half-credit slots.
fn restriction_required_cu(number: i32, cu_field: &Option<i32>) -> f64 {
    if let Some(tenths) = cu_field {
        return (*tenths as f64) / 10.0;
    }
    number as f64
}

/// Fill order: SingleCourse first, then composites (incl. DoubleCount), then Restriction,
/// then Business Breadth last (category contains "business breadth").
/// Among restrictions, smaller-CU slots are matched before larger ones.
fn requirement_fill_order_key(req: &Requirement) -> (u32, u32, usize) {
    let cat = req.get_category();
    if Requirement::is_business_breadth_category((!cat.is_empty()).then_some(&cat)) {
        return (3, 0, req.specificity_score());
    }
    match req {
        Requirement::SingleCourse { .. } => (0, 0, req.specificity_score()),
        Requirement::Restriction { number, cu, .. } => {
            let target = restriction_required_cu(*number, cu);
            let tenths = (target * 10.0).round() as u32;
            (2, tenths, req.specificity_score())
        }
        Requirement::CoursePool { .. } => (1, 0, req.specificity_score()),
        _ => (1, 0, req.specificity_score()),
    }
}

fn is_restriction_requirement(req: &Requirement) -> bool {
    matches!(req, Requirement::Restriction { .. })
}

fn wrap_expanded_requirement_children(expanded: Vec<Requirement>) -> Requirement {
    match expanded.len() {
        0 => panic!("expand_restriction_slots produced an empty requirement list"),
        1 => expanded.into_iter().next().expect("single expanded requirement"),
        _ => Requirement::AllOf {
            category: None,
            requirements: expanded,
        },
    }
}

fn expand_pool_constraint(pc: PoolConstraint) -> Vec<PoolConstraint> {
    expand_restriction_slots(vec![pc.requirement])
        .into_iter()
        .map(|requirement| PoolConstraint {
            requirement,
            count: pc.count,
            consumption_group: pc.consumption_group.clone(),
        })
        .collect()
}

fn expand_restriction_slot(req: Requirement) -> Vec<Requirement> {
    match req {
        Requirement::Restriction {
            category,
            department,
            cu,
            level,
            attr,
            excluding,
            number,
            no_school,
        } if number > 1 && cu.is_none() => (0..number)
            .map(|_| Requirement::Restriction {
                category: category.clone(),
                department: department.clone(),
                cu: None,
                level: level.clone(),
                attr: attr.clone(),
                excluding: excluding.clone(),
                number: 1,
                no_school: no_school.clone(),
            })
            .collect(),
        Requirement::AllOf {
            category,
            requirements,
        } => vec![Requirement::AllOf {
            category,
            requirements: expand_restriction_slots(requirements),
        }],
        Requirement::AnyOf {
            category,
            possibilities,
        } => vec![Requirement::AnyOf {
            category,
            possibilities: possibilities
                .into_iter()
                .map(|child| wrap_expanded_requirement_children(expand_restriction_slot(child)))
                .collect(),
        }],
        Requirement::CourseGroup {
            category,
            number,
            possibilities,
        } => vec![Requirement::CourseGroup {
            category,
            number,
            possibilities: possibilities
                .into_iter()
                .map(|child| wrap_expanded_requirement_children(expand_restriction_slot(child)))
                .collect(),
        }],
        Requirement::Concentration {
            category,
            number,
            requirements,
        } => vec![Requirement::Concentration {
            category,
            number,
            requirements: expand_restriction_slots(requirements),
        }],
        Requirement::CoursePool {
            category,
            fixed_slots,
            flexible_slots,
            constraints,
        } => vec![Requirement::CoursePool {
            category,
            fixed_slots: expand_restriction_slots(fixed_slots),
            flexible_slots,
            constraints: constraints
                .into_iter()
                .flat_map(expand_pool_constraint)
                .collect(),
        }],
        other => vec![other],
    }
}

/// Expand [`Requirement::Restriction`] entries with `number > 1` and `cu: None` into that many
/// separate 1-CU slots. Composites are traversed recursively.
pub fn expand_restriction_slots(requirements: Vec<Requirement>) -> Vec<Requirement> {
    requirements
        .into_iter()
        .flat_map(expand_restriction_slot)
        .collect()
}

fn sorted_child_requirements<'a>(requirements: &'a [Requirement]) -> Vec<&'a Requirement> {
    let mut children: Vec<&Requirement> = requirements.iter().collect();
    children.sort_by_key(|r| requirement_fill_order_key(r));
    children
}

/// Expand [`Requirement::Concentration`] blocks inside pool fixed slots into their children.
fn expand_pool_fixed_slots(
    fixed_slots: Vec<Requirement>,
) -> Vec<(usize, usize, Requirement)> {
    let mut out = Vec::new();
    for (fi, slot_req) in fixed_slots.into_iter().enumerate() {
        match slot_req {
            Requirement::Concentration { requirements, .. } => {
                for (ci, child) in requirements.into_iter().enumerate() {
                    out.push((fi, ci, child));
                }
            }
            other => out.push((fi, 0, other)),
        }
    }
    out
}

fn flatten_pool_fixed_slot_refs(fixed_slots: &[Requirement]) -> Vec<&Requirement> {
    let mut out = Vec::new();
    for req in fixed_slots {
        match req {
            Requirement::Concentration { requirements, .. } => {
                out.extend(requirements.iter());
            }
            other => out.push(other),
        }
    }
    out
}

fn flatten_pool_fixed_slots_owned(fixed_slots: &[Requirement]) -> Vec<Requirement> {
    let mut out = Vec::new();
    for req in fixed_slots {
        match req {
            Requirement::Concentration { requirements, .. } => {
                out.extend(requirements.iter().cloned());
            }
            other => out.push(other.clone()),
        }
    }
    out
}

pub fn requirements_contain_concentration(requirements: &[Requirement]) -> bool {
    requirements
        .iter()
        .any(requirement_is_or_contains_concentration)
}

fn requirement_is_or_contains_concentration(req: &Requirement) -> bool {
    match req {
        Requirement::Concentration { .. } => true,
        Requirement::CoursePool { fixed_slots, .. } => fixed_slots
            .iter()
            .any(requirement_is_or_contains_concentration),
        Requirement::AllOf { requirements, .. } => requirements
            .iter()
            .any(requirement_is_or_contains_concentration),
        Requirement::AnyOf { possibilities, .. } | Requirement::CourseGroup { possibilities, .. } => {
            possibilities
                .iter()
                .any(requirement_is_or_contains_concentration)
        }
        _ => false,
    }
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
            possibilities, ..
        } => {
            let mut pool: Vec<String> = taken.to_vec();
            let mut assigned = Vec::new();
            for child in sorted_child_requirements(possibilities) {
                let (courses, _) =
                    partial_fulfill_composite(child, &pool, attributes, cu_map);
                if courses.is_empty() {
                    continue;
                }
                for course in &courses {
                    pool.retain(|c| c != course);
                }
                assigned.extend(courses);
            }
            (assigned, None)
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
    if let Some(excluding_items) = excluding {
        for ex in excluding_items {
            if crate::course::is_valid_course_code(ex) {
                if ex == course {
                    return false;
                }
            } else if attributes
                .get(ex)
                .is_some_and(|courses| courses.contains(&course.to_string()))
            {
                return false;
            }
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

/// Pool coverage constraints use instance ids like `"1:c0"` (not fixed slots `1:f0:c0` / `1:p0`).
pub fn is_pool_constraint_instance_id(instance_id: Option<&str>) -> bool {
    instance_id.is_some_and(|id| {
        let segments: Vec<&str> = id.split(':').collect();
        let has_fixed_or_flex = segments.iter().any(|seg| {
            seg.len() > 1
                && (seg.starts_with('f') || seg.starts_with('p'))
                && seg[1..].chars().all(|c| c.is_ascii_digit())
        });
        !has_fixed_or_flex
            && segments.iter().any(|seg| {
                seg.len() > 1
                    && seg.starts_with('c')
                    && seg[1..].chars().all(|c| c.is_ascii_digit())
            })
    })
}

/// Schedule slots scoped to a pool coverage constraint should not appear on the grid.
pub fn is_pool_constraint_slot_id(slot_id: &str) -> bool {
    if !is_requirement_slot_id(slot_id) {
        return false;
    }
    let rest = match slot_id.strip_prefix("req:") {
        Some(r) => r,
        None => return false,
    };
    let scope = rest.split(":R:").next().unwrap_or(rest);
    is_pool_constraint_instance_id(Some(scope))
}

/// Pool slot placeholders (fixed / flexible) that may be placed on the schedule.
pub fn is_schedulable_requirement_slot_id(slot_id: &str) -> bool {
    is_requirement_slot_id(slot_id) && !is_pool_constraint_slot_id(slot_id)
}

/// Generic 1-CU placeholder inside a course pool.
pub fn pool_flexible_slot_requirement(pool_category: &str, _index: usize) -> Requirement {
    Requirement::Restriction {
        category: Some(pool_category.to_string()),
        department: None,
        cu: None,
        level: None,
        attr: None,
        excluding: None,
        number: 1,
        no_school: None,
    }
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
            Requirement::CourseGroup { possibilities, .. } => {
                for (j, child) in possibilities.iter().enumerate() {
                    if child == needle {
                        return Some(format!("{}#{}", path, j));
                    }
                    if let Some(p) = child.find_path_in_subtree(needle, &format!("{}#{}", path, j)) {
                        return Some(p);
                    }
                }
            }
            Requirement::CoursePool {
                fixed_slots,
                constraints,
                ..
            } => {
                for (j, child) in fixed_slots.iter().enumerate() {
                    if child == needle {
                        return Some(format!("{}#f{}", path, j));
                    }
                    if let Some(p) = child.find_path_in_subtree(needle, &format!("{}#f{}", path, j)) {
                        return Some(p);
                    }
                }
                for (j, pc) in constraints.iter().enumerate() {
                    if &pc.requirement == needle {
                        return Some(format!("{}#c{}", path, j));
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
            Requirement::CourseGroup { possibilities, .. } => {
                for child in possibilities {
                    if let Some(found) = child.find_for_slot_id(slot_id) {
                        return Some(found);
                    }
                }
            }
            Requirement::CoursePool { fixed_slots, .. } => {
                for child in fixed_slots {
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
            Requirement::CoursePool {
                fixed_slots,
                ..
            } => {
                for child in fixed_slots {
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
            | Requirement::CoursePool { category, ..}
            | Requirement::AllOf { category, ..}
            | Requirement::AnyOf { category, ..} => category.clone().unwrap_or("".to_string()),
        }
    }

    /// Depth-first category list for UI grouping (skips generic pool wrapper labels).
    pub fn collect_category_order(&self, order: &mut Vec<String>) {
        match self {
            Requirement::CoursePool {
                category,
                fixed_slots,
                ..
            } => {
                let pool_cat = category.clone().unwrap_or_else(|| "Course Pool".to_string());
                if !order.contains(&pool_cat) {
                    order.push(pool_cat);
                }
                for req in fixed_slots {
                    req.collect_category_order(order);
                }
            }
            Requirement::AllOf { requirements, .. }
            | Requirement::Concentration { requirements, .. } => {
                let cat = self.get_category();
                if !cat.is_empty() && !order.contains(&cat) {
                    order.push(cat);
                }
                for req in requirements {
                    req.collect_category_order(order);
                }
            }
            Requirement::AnyOf { possibilities, .. }
            | Requirement::CourseGroup { possibilities, .. } => {
                let cat = self.get_category();
                if !cat.is_empty() && !order.contains(&cat) {
                    order.push(cat);
                }
                for req in possibilities {
                    req.collect_category_order(order);
                }
            }
            _ => {
                let cat = self.get_category();
                if !cat.is_empty() && !order.contains(&cat) {
                    order.push(cat);
                }
            }
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
            Requirement::CourseGroup { number, possibilities, .. } => {
                let need = *number as usize;
                let mut pool = taken.clone();
                let mut all_courses: Vec<String> = Vec::new();
                let mut fulfilled_count = 0usize;
                for child in sorted_child_requirements(possibilities) {
                    if fulfilled_count >= need {
                        break;
                    }
                    if let Some(mut courses) = child.fulfills_requirement(&pool, attributes, cu_map) {
                        pool.retain(|x| !courses.contains(x));
                        all_courses.append(&mut courses);
                        fulfilled_count += 1;
                    }
                }
                if fulfilled_count >= need {
                    Some(all_courses)
                } else {
                    None
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
            Requirement::CoursePool {
                category,
                fixed_slots,
                flexible_slots,
                constraints,
                ..
            } => {
                let pool_cat = category.as_deref().unwrap_or("Course Pool");
                let mut taken_copy = taken.clone();
                let mut pool_courses: Vec<String> = Vec::new();

                let flat_fixed = flatten_pool_fixed_slots_owned(fixed_slots);
                for req in sorted_child_requirements(&flat_fixed) {
                    if let Some(mut courses) = req.fulfills_requirement(&taken_copy, attributes, cu_map) {
                        taken_copy.retain(|x| !courses.contains(x));
                        pool_courses.append(&mut courses);
                    } else {
                        return None;
                    }
                }
                for pi in 0..(*flexible_slots).max(0) as usize {
                    let flex = pool_flexible_slot_requirement(pool_cat, pi);
                    if let Some(mut courses) = flex.fulfills_requirement(&taken_copy, attributes, cu_map) {
                        taken_copy.retain(|x| !courses.contains(x));
                        pool_courses.append(&mut courses);
                    } else {
                        return None;
                    }
                }

                let evaluations = evaluate_pool_constraints(
                    &pool_courses,
                    constraints,
                    attributes,
                    cu_map,
                );
                if evaluations.iter().all(|e| e.fulfilled) {
                    return Some(pool_courses);
                }
                None
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
            Requirement::CourseGroup { number, possibilities, .. } => {
                let need = *number as usize;
                let mut pool = taken.clone();
                let mut suggested_courses = Vec::new();
                let mut fulfilled_count = 0usize;
                for child in sorted_child_requirements(possibilities) {
                    if fulfilled_count >= need {
                        break;
                    }
                    if child.fulfills_requirement(&pool, &attributes, cu_map).is_some() {
                        fulfilled_count += 1;
                        continue;
                    }
                    if let Some(mut val) = child.suggest_for_requirement(
                        &pool,
                        &attributes,
                        cu_map,
                        scope,
                        cross_filter,
                    ) {
                        suggested_courses.append(&mut val);
                        fulfilled_count += 1;
                    } else {
                        return None;
                    }
                }
                if suggested_courses.is_empty() {
                    None
                } else {
                    Some(suggested_courses)
                }
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
            Requirement::CoursePool {
                category,
                fixed_slots,
                flexible_slots,
                ..
            } => {
                let pool_cat = category.as_deref().unwrap_or("Course Pool");
                let mut taken_copy = taken.clone();
                let mut unfulfilled_slots: Vec<Requirement> = Vec::new();

                for req in flatten_pool_fixed_slot_refs(fixed_slots) {
                    if req.fulfills_requirement(&taken_copy, attributes, cu_map).is_some() {
                        if let Some(courses) = req.fulfills_requirement(&taken_copy, attributes, cu_map) {
                            taken_copy.retain(|x| !courses.contains(x));
                        }
                    } else {
                        unfulfilled_slots.push(req.clone());
                    }
                }
                for pi in 0..(*flexible_slots).max(0) as usize {
                    let flex = pool_flexible_slot_requirement(pool_cat, pi);
                    if let Some(courses) = flex.fulfills_requirement(&taken_copy, attributes, cu_map) {
                        taken_copy.retain(|x| !courses.contains(x));
                    } else {
                        unfulfilled_slots.push(flex);
                    }
                }

                let mut suggestions: Vec<String> = Vec::new();
                for req in &unfulfilled_slots {
                    if let Some(s) = req.suggest_for_requirement(
                        taken,
                        attributes,
                        cu_map,
                        scope,
                        cross_filter,
                    ) {
                        suggestions.extend(s);
                    }
                }
                if suggestions.is_empty() {
                    None
                } else {
                    Some(suggestions)
                }
            },
        }
    }

    pub fn create_requirement_description(&self) -> String {
        match self {
            Requirement::SingleCourse { possibilities, .. } => {
                format_truncated_list(possibilities, "One of: ")
            }
            Requirement::CourseGroup { number, possibilities, .. } => {
                format!(
                    "Complete {} of {} areas",
                    number,
                    possibilities.len()
                )
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
            Requirement::CoursePool {
                category,
                fixed_slots,
                flexible_slots,
                constraints,
                ..
            } => {
                let name = category.clone().unwrap_or_else(|| "Course Pool".to_string());
                format!(
                    "{}: {} fixed + {} pool CU; {} coverage rules",
                    name,
                    fixed_slots.len(),
                    flexible_slots,
                    constraints.len()
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
                possibilities
                    .iter()
                    .map(|r| r.specificity_score())
                    .sum::<usize>()
                    .max(1)
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
            Requirement::CoursePool {
                fixed_slots,
                flexible_slots,
                ..
            } => {
                fixed_slots
                    .iter()
                    .map(|r| r.specificity_score())
                    .sum::<usize>()
                    .max(1)
                    + (*flexible_slots).max(0) as usize
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

/// Result of validating a degree's requirement tree.
#[derive(Debug, Clone)]
pub struct DegreeValidationResult {
    pub fulfilled: Vec<MappedRequirement>,
    pub unfulfilled: Vec<MappedRequirement>,
    pub pool_coverage_info: Vec<PoolCoverageInfo>,
}

/// One coverage constraint evaluated against a course pool.
#[derive(Debug, Clone)]
pub struct PoolConstraintEvaluation {
    pub requirement: Requirement,
    pub fulfilled: bool,
    pub course_ids: Vec<String>,
    pub consumption_group: String,
    pub label: String,
}

fn constraint_default_consumption_group(req: &Requirement, index: usize) -> String {
    let cat = req.get_category();
    if cat.starts_with("Foundational Approaches") {
        return "cas:fa".to_string();
    }
    if cat.starts_with("Sectors of Knowledge") {
        return "cas:sector".to_string();
    }
    if !cat.is_empty() {
        return format!("slot:{cat}");
    }
    format!("slot:{index}")
}

fn constraint_short_label(req: &Requirement) -> String {
    if let Requirement::Restriction {
        attr,
        no_school,
        department,
        ..
    } = req
    {
        if let Some(attrs) = attr {
            if attrs.len() > 1 {
                return attrs.join("/");
            }
            if let Some(a) = attrs.first() {
                return a.clone();
            }
        }
        if let Some(school) = no_school {
            return format!("non-{school}");
        }
        if let Some(depts) = department {
            return depts.join("/");
        }
    }
    let cat = req.get_category();
    if cat.is_empty() {
        "constraint".to_string()
    } else {
        cat
    }
}

fn expanded_pool_constraint_units(constraints: &[PoolConstraint]) -> Vec<(Requirement, String)> {
    let mut units = Vec::new();
    for (ci, pc) in constraints.iter().enumerate() {
        let group = pc
            .consumption_group
            .clone()
            .unwrap_or_else(|| constraint_default_consumption_group(&pc.requirement, ci));
        for _ in 0..pc.count.max(1) {
            units.push((pc.requirement.clone(), group.clone()));
        }
    }
    units
}

/// Expanded pool constraint units for overlap planning (one entry per coverage slot).
pub fn pool_constraint_units(constraints: &[PoolConstraint]) -> Vec<(Requirement, String)> {
    expanded_pool_constraint_units(constraints)
}

/// Attribute-specific pool units are matched before broad catch-alls (e.g. non-Wharton).
fn pool_constraint_unit_priority(req: &Requirement) -> u8 {
    match req {
        Requirement::Restriction {
            attr,
            department,
            no_school,
            ..
        } => {
            if attr.as_ref().is_some_and(|a| !a.is_empty()) {
                0
            } else if department.as_ref().is_some_and(|d| !d.is_empty()) {
                1
            } else if no_school.is_some() {
                3
            } else {
                2
            }
        }
        _ => 2,
    }
}

/// Match pool coverage constraints against courses assigned to pool slots.
/// Each course may satisfy at most [`POOL_MAX_CONSTRAINT_USES_PER_COURSE`] constraints
/// across the pool; [`PoolConstraint::consumption_group`] still prevents reuse within one group.
pub fn evaluate_pool_constraints(
    pool_courses: &[String],
    constraints: &[PoolConstraint],
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> Vec<PoolConstraintEvaluation> {
    let units = expanded_pool_constraint_units(constraints);
    let unit_count = units.len();
    let mut eval_order: Vec<usize> = (0..unit_count).collect();
    eval_order.sort_by_key(|&i| {
        (
            pool_constraint_unit_priority(&units[i].0),
            i,
        )
    });

    let mut blocked_by_group: HashMap<String, HashSet<String>> = HashMap::new();
    let mut course_constraint_uses: HashMap<String, usize> = HashMap::new();
    let mut results: Vec<Option<PoolConstraintEvaluation>> = vec![None; unit_count];

    for i in eval_order {
        let (req, group) = units[i].clone();
        let blocked = blocked_by_group.get(&group).cloned().unwrap_or_default();
        let available: Vec<String> = pool_courses
            .iter()
            .filter(|c| {
                !blocked.contains(*c)
                    && course_constraint_uses
                        .get(*c)
                        .copied()
                        .unwrap_or(0)
                        < POOL_MAX_CONSTRAINT_USES_PER_COURSE
            })
            .cloned()
            .collect();
        let label = constraint_short_label(&req);

        results[i] = Some(match req.fulfills_requirement(&available, attributes, cu_map) {
            Some(courses) => {
                blocked_by_group
                    .entry(group.clone())
                    .or_default()
                    .extend(courses.iter().cloned());
                for course in &courses {
                    *course_constraint_uses.entry(course.clone()).or_insert(0) += 1;
                }
                PoolConstraintEvaluation {
                    requirement: req,
                    fulfilled: true,
                    course_ids: courses,
                    consumption_group: group,
                    label,
                }
            }
            None => PoolConstraintEvaluation {
                requirement: req,
                fulfilled: false,
                course_ids: vec![],
                consumption_group: group,
                label,
            },
        });
    }

    results.into_iter().map(|r| r.expect("all units evaluated")).collect()
}

fn pool_fill_hint(evaluations: &[PoolConstraintEvaluation]) -> Option<String> {
    let open: Vec<&PoolConstraintEvaluation> =
        evaluations.iter().filter(|e| !e.fulfilled).collect();
    if open.is_empty() {
        return None;
    }
    let open_count = open.len();
    if open_count <= 3 {
        return Some(format!(
            "{} open coverage requirement{} remaining",
            open_count,
            if open_count == 1 { "" } else { "s" }
        ));
    }
    Some(format!("{open_count} open coverage requirements remaining"))
}

fn pop_group_unit(by_group: &mut HashMap<String, Vec<String>>, group: &str) -> Option<String> {
    let units = by_group.get_mut(group)?;
    units.pop()
}

fn push_group_unit(by_group: &mut HashMap<String, Vec<String>>, group: &str, label: String) {
    by_group.entry(group.to_string()).or_default().push(label);
}

fn try_pop_priority_pair(
    by_group: &mut HashMap<String, Vec<String>>,
    g1: &str,
    g2: &str,
) -> Option<String> {
    let a = pop_group_unit(by_group, g1)?;
    if let Some(b) = pop_group_unit(by_group, g2) {
        return Some(format!("{a} + {b} (double-count)"));
    }
    push_group_unit(by_group, g1, a);
    None
}

fn try_pop_any_pair(by_group: &mut HashMap<String, Vec<String>>) -> Option<String> {
    let groups: Vec<String> = by_group
        .iter()
        .filter(|(_, units)| !units.is_empty())
        .map(|(g, _)| g.clone())
        .collect();
    for i in 0..groups.len() {
        for j in (i + 1)..groups.len() {
            if let Some(hint) = try_pop_priority_pair(by_group, &groups[i], &groups[j]) {
                return Some(hint);
            }
        }
    }
    None
}

fn try_pop_single(by_group: &mut HashMap<String, Vec<String>>) -> Option<String> {
    for group in by_group.keys().cloned().collect::<Vec<_>>() {
        if let Some(label) = pop_group_unit(by_group, &group) {
            return Some(label);
        }
    }
    None
}

/// Greedy suggested allocation of open coverage units across unfilled flexible pool slots.
pub fn plan_pool_slot_hints(
    evaluations: &[PoolConstraintEvaluation],
    flexible_slots_total: i32,
    flexible_slots_filled: i32,
) -> Vec<String> {
    let open_flex = (flexible_slots_total - flexible_slots_filled).max(0) as usize;
    if open_flex == 0 {
        return vec![];
    }

    let mut by_group: HashMap<String, Vec<String>> = HashMap::new();
    for eval in evaluations.iter().filter(|e| !e.fulfilled) {
        by_group
            .entry(eval.consumption_group.clone())
            .or_default()
            .push(eval.label.clone());
    }

    let pair_priority: [(&str, &str); 6] = [
        ("wh:wufl", "wh:mt_las"),
        ("wh:cc_fl", "wh:ssh"),
        ("wh:cross_cultural", "wh:ssh"),
        ("wh:cc_fl", "wh:non_wh"),
        ("wh:cross_cultural", "wh:non_wh"),
        ("cas:fa", "cas:sector"),
    ];

    let mut hints = Vec::with_capacity(open_flex);
    for _ in 0..open_flex {
        let mut assigned = false;
        for (g1, g2) in pair_priority {
            if let Some(hint) = try_pop_priority_pair(&mut by_group, g1, g2) {
                hints.push(hint);
                assigned = true;
                break;
            }
        }
        if assigned {
            continue;
        }
        if let Some(hint) = try_pop_any_pair(&mut by_group) {
            hints.push(hint);
            continue;
        }
        if let Some(label) = try_pop_single(&mut by_group) {
            hints.push(label);
        } else {
            hints.push("Open pool elective".to_string());
        }
    }

    hints
}

fn build_pool_coverage_info(
    pool_index: usize,
    category: Option<String>,
    pool_courses: Vec<String>,
    fixed_slots_total: i32,
    fixed_slots_filled: i32,
    flexible_slots_total: i32,
    flexible_slots_filled: i32,
    evaluations: &[PoolConstraintEvaluation],
) -> PoolCoverageInfo {
    PoolCoverageInfo {
        pool_index,
        category: category.unwrap_or_else(|| "Course Pool".to_string()),
        pool_courses,
        fixed_slots_total,
        fixed_slots_filled,
        flexible_slots_total,
        flexible_slots_filled,
        constraints: evaluations
            .iter()
            .map(|e| PoolConstraintStatus {
                label: e.label.clone(),
                description: e.requirement.create_requirement_description(),
                fulfilled: e.fulfilled,
                matched_courses: e.course_ids.clone(),
                consumption_group: e.consumption_group.clone(),
            })
            .collect(),
        fill_hint: pool_fill_hint(evaluations),
        slot_hints: plan_pool_slot_hints(
            evaluations,
            flexible_slots_total,
            flexible_slots_filled,
        ),
    }
}

fn count_pool_prefix_slots_filled(
    pool_idx: usize,
    prefix: char,
    total: i32,
    fulfilled: &[MappedRequirement],
    unfulfilled: &[MappedRequirement],
) -> i32 {
    let mut count = 0i32;
    for pi in 0..total.max(0) {
        let child_id = format!("{pool_idx}:{prefix}{pi}");
        let filled = fulfilled.iter().chain(unfulfilled.iter()).any(|m| {
            m.instance_id.as_deref() == Some(child_id.as_str()) && !m.course_ids.is_empty()
        });
        if filled {
            count += 1;
        }
    }
    count
}

fn collect_pool_courses_for_block(
    pool_idx: usize,
    fulfilled: &[MappedRequirement],
    unfulfilled: &[MappedRequirement],
) -> Vec<String> {
    let prefix = format!("{pool_idx}:");
    let mut courses = Vec::new();
    for mapped in fulfilled
        .iter()
        .chain(unfulfilled.iter().filter(|m| m.partial))
    {
        if let Some(id) = &mapped.instance_id {
            if id.starts_with(&prefix) && !is_pool_constraint_instance_id(Some(id)) {
                courses.extend(mapped.course_ids.clone());
            }
        }
    }
    courses
}

/// Coverage status for each course pool in a degree.
pub fn pool_coverage_info_from_degree_requirements(
    requirements: &[Requirement],
    fulfilled: &[MappedRequirement],
    unfulfilled: &[MappedRequirement],
    cu_map: &HashMap<String, f64>,
) -> Vec<PoolCoverageInfo> {
    let attributes = attributes_data::create_attributes();
    let mut result = Vec::new();

    for (pool_idx, req) in requirements.iter().enumerate() {
        if let Requirement::CoursePool {
            category,
            fixed_slots,
            flexible_slots,
            constraints,
        } = req
        {
            let pool_courses =
                collect_pool_courses_for_block(pool_idx, fulfilled, unfulfilled);
            let evaluations = evaluate_pool_constraints(
                &pool_courses,
                constraints,
                &attributes,
                cu_map,
            );
            let fixed_filled = count_pool_prefix_slots_filled(
                pool_idx,
                'f',
                fixed_slots.len() as i32,
                fulfilled,
                unfulfilled,
            );
            let flex_filled = count_pool_prefix_slots_filled(
                pool_idx,
                'p',
                *flexible_slots,
                fulfilled,
                unfulfilled,
            );
            result.push(build_pool_coverage_info(
                pool_idx,
                category.clone(),
                pool_courses,
                fixed_slots.len() as i32,
                fixed_filled,
                *flexible_slots,
                flex_filled,
                &evaluations,
            ));
        }
    }

    result
}

impl DegreeValidationResult {
    /// Recompute pool coverage after fulfilled/unfulfilled lists change.
    pub fn refresh_pool_coverage_info(
        &mut self,
        requirements: &[Requirement],
        cu_map: &HashMap<String, f64>,
    ) {
        self.pool_coverage_info = pool_coverage_info_from_degree_requirements(
            requirements,
            &self.fulfilled,
            &self.unfulfilled,
            cu_map,
        );
    }
}

/// finding whether taken fulfills degree and to what extent
pub fn validate_courses_for_degree(
    requirements: Vec<Requirement>,
    taken: &Vec<String>,
    cu_map: &HashMap<String, f64>,
) -> DegreeValidationResult {
    let attributes = attributes_data::create_attributes();
    let requirements = expand_restriction_slots(requirements);
    let root_requirements = requirements.clone();
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
            Requirement::CoursePool {
                category,
                fixed_slots,
                flexible_slots,
                constraints,
            } => {
                let pool_cat = category.clone().unwrap_or_else(|| "Course Pool".to_string());
                let mut pool_courses: Vec<String> = Vec::new();

                for (fi, ci, slot_req) in expand_pool_fixed_slots(fixed_slots) {
                    let child_id = Some(format!("{}:f{}:c{}", orig_idx, fi, ci));
                    let courses = try_fulfill_or_partial_base(
                        &slot_req,
                        &mut taken_mut,
                        &attributes,
                        cu_map,
                        child_id,
                        &mut fulfilled_requirements,
                        &mut requirements_not_fulfilled,
                    );
                    pool_courses.extend(courses);
                }

                for pi in 0..flexible_slots.max(0) as usize {
                    let flex_req = pool_flexible_slot_requirement(&pool_cat, pi);
                    let child_id = Some(format!("{}:p{}", orig_idx, pi));
                    let courses = try_fulfill_or_partial_base(
                        &flex_req,
                        &mut taken_mut,
                        &attributes,
                        cu_map,
                        child_id,
                        &mut fulfilled_requirements,
                        &mut requirements_not_fulfilled,
                    );
                    pool_courses.extend(courses);
                }

                let constraint_evals = evaluate_pool_constraints(
                    &pool_courses,
                    &constraints,
                    &attributes,
                    cu_map,
                );
                for (ci, eval) in constraint_evals.iter().enumerate() {
                    let child_id = Some(format!("{}:c{}", orig_idx, ci));
                    let mapped = new_mapped_requirement(
                        eval.requirement.clone(),
                        eval.course_ids.clone(),
                        child_id,
                        &attributes,
                    );
                    if eval.fulfilled {
                        fulfilled_requirements.push(mapped);
                    } else {
                        requirements_not_fulfilled.push(mapped);
                    }
                }
            }
            ref composite
                if matches!(
                    composite,
                    Requirement::AnyOf { .. }
                        | Requirement::AllOf { .. }
                        | Requirement::Concentration { .. }
                        | Requirement::CourseGroup { .. }
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

    let pool_coverage_info = pool_coverage_info_from_degree_requirements(
        &root_requirements,
        &fulfilled_requirements,
        &requirements_not_fulfilled,
        cu_map,
    );

    DegreeValidationResult {
        fulfilled: fulfilled_requirements,
        unfulfilled: requirements_not_fulfilled,
        pool_coverage_info,
    }
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
        // Pool coverage constraints are not schedulable CU slots.
        if is_pool_constraint_instance_id(mapped.instance_id.as_deref()) {
            continue;
        }
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
pub struct PoolConstraintStatus {
    pub label: String,
    pub description: String,
    pub fulfilled: bool,
    pub matched_courses: Vec<String>,
    pub consumption_group: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PoolCoverageInfo {
    pub pool_index: usize,
    pub category: String,
    pub pool_courses: Vec<String>,
    pub fixed_slots_total: i32,
    pub fixed_slots_filled: i32,
    pub flexible_slots_total: i32,
    pub flexible_slots_filled: i32,
    pub constraints: Vec<PoolConstraintStatus>,
    pub fill_hint: Option<String>,
    /// Suggested coverage allocation for each unfilled flexible pool slot (in order).
    pub slot_hints: Vec<String>,
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

fn embedded_concentration_category(conc_name: &str) -> String {
    format!("Concentration - {conc_name}")
}

fn concentration_slot_category_matches(conc_name: &str, category: &str) -> bool {
    category == conc_name || category == embedded_concentration_category(conc_name)
}

fn requirement_includes_concentration_name(req: &Requirement, conc_name: &str) -> bool {
    match req {
        Requirement::Concentration { category, .. } => category.as_deref() == Some(conc_name),
        Requirement::CoursePool { fixed_slots, .. } => fixed_slots
            .iter()
            .any(|r| requirement_includes_concentration_name(r, conc_name)),
        Requirement::AllOf { requirements, .. } => requirements
            .iter()
            .any(|r| requirement_includes_concentration_name(r, conc_name)),
        Requirement::AnyOf { possibilities, .. } | Requirement::CourseGroup { possibilities, .. } => {
            possibilities
                .iter()
                .any(|r| requirement_includes_concentration_name(r, conc_name))
        }
        _ => concentration_slot_category_matches(conc_name, &req.get_category()),
    }
}

fn requirements_include_embedded_concentration(
    requirements: &[Requirement],
    conc_name: &str,
) -> bool {
    requirements
        .iter()
        .any(|r| requirement_includes_concentration_name(r, conc_name))
}

fn collect_embedded_concentration_slots<'a>(
    validation: &'a DegreeValidationResult,
    conc_name: &str,
) -> Vec<&'a MappedRequirement> {
    let mut slots: Vec<_> = validation
        .fulfilled
        .iter()
        .chain(validation.unfulfilled.iter())
        .filter(|m| concentration_slot_category_matches(conc_name, &m.requirement.get_category()))
        .collect();
    slots.sort_by(|a, b| a.instance_id.cmp(&b.instance_id));
    slots
}

fn extract_embedded_concentration_from_validation(
    validation: &DegreeValidationResult,
    conc_name: &str,
    conc_reqs: &[Requirement],
) -> ConcentrationInfo {
    let slots = collect_embedded_concentration_slots(validation, conc_name);
    let req_descriptions: Vec<String> = conc_reqs
        .iter()
        .map(|req| {
            let desc = req.create_requirement_description();
            if desc.is_empty() {
                req.get_category()
            } else {
                desc
            }
        })
        .collect();

    let mut matched_courses = Vec::new();
    let mut req_fulfilled = Vec::new();
    for i in 0..conc_reqs.len() {
        if let Some(mapped) = slots.get(i) {
            matched_courses.push(mapped.course_ids.clone());
            req_fulfilled.push(!mapped.course_ids.is_empty());
        } else {
            matched_courses.push(vec![]);
            req_fulfilled.push(false);
        }
    }

    ConcentrationInfo {
        name: conc_name.to_string(),
        is_core: false,
        requirements_total: conc_reqs.len(),
        requirements_fulfilled: req_fulfilled.iter().filter(|&&x| x).count(),
        requirement_descriptions: req_descriptions,
        requirement_fulfilled: req_fulfilled,
        matched_courses,
    }
}

fn fill_overlay_concentration_greedy(
    conc_reqs: &[Requirement],
    pool: &mut Vec<String>,
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> (Vec<String>, Vec<bool>, Vec<Vec<String>>) {
    let mut req_descriptions = Vec::new();
    let mut req_fulfilled = Vec::new();
    let mut matched_courses = Vec::new();

    for req in conc_reqs {
        let desc = req.create_requirement_description();
        let desc = if desc.is_empty() {
            req.get_category()
        } else {
            desc
        };
        req_descriptions.push(desc);

        if let Some(courses) = req.fulfills_requirement(pool, attributes, cu_map) {
            pool.retain(|x| !courses.contains(x));
            req_fulfilled.push(true);
            matched_courses.push(courses);
        } else {
            req_fulfilled.push(false);
            matched_courses.push(vec![]);
        }
    }

    (req_descriptions, req_fulfilled, matched_courses)
}

/// Extract concentration progress info for overlay-style concentrations.
/// For core concentrations (Requirement::Concentration in requirements), only name + is_core are populated.
pub fn extract_concentration_info(
    requirements: &Vec<Requirement>,
    concentrations: &Option<std::collections::BTreeMap<String, Vec<Requirement>>>,
    selected_concentrations: &[String],
    taken: &Vec<String>,
    cu_map: &HashMap<String, f64>,
    validation: Option<&DegreeValidationResult>,
) -> Vec<ConcentrationInfo> {
    let attributes = attributes_data::create_attributes();

    // Check if this major has a core concentration (Requirement::Concentration in requirements)
    let has_core = requirements_contain_concentration(requirements)
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

    let mut results = Vec::new();
    let mut remaining_taken = taken.clone();

    for selected in selected_concentrations {
        let conc_reqs = match conc_map.get(selected) {
            Some(reqs) => reqs,
            None => continue,
        };

        if validation.is_some()
            && requirements_include_embedded_concentration(requirements, selected)
        {
            results.push(extract_embedded_concentration_from_validation(
                validation.unwrap(),
                selected,
                conc_reqs,
            ));
            continue;
        }

        let (req_descriptions, req_fulfilled, matched_courses) =
            fill_overlay_concentration_greedy(conc_reqs, &mut remaining_taken, &attributes, cu_map);

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
    per_degree: &[DegreeValidationResult],
    course: &str,
    degree_idx: usize,
    conc_contexts: Option<&[DegreeConcentrationContext]>,
    taken: Option<&[String]>,
    cu_map: &HashMap<String, f64>,
) -> usize {
    let mut score = per_degree[degree_idx]
        .fulfilled
        .iter()
        .filter(|m| m.course_ids.contains(&course.to_string()))
        .count();

    if let (Some(contexts), Some(taken)) = (conc_contexts, taken) {
        if let Some(ctx) = contexts.get(degree_idx) {
            score += CONCENTRATION_PRIORITY_WEIGHT
                * concentration_slots_for_course(
                    ctx,
                    course,
                    taken,
                    cu_map,
                    per_degree.get(degree_idx),
                );
        }
    }

    score
}

fn choose_best_two_degrees(
    course: &str,
    degree_indices: &[usize],
    per_degree: &[DegreeValidationResult],
    conc_contexts: Option<&[DegreeConcentrationContext]>,
    taken: Option<&[String]>,
    cu_map: &HashMap<String, f64>,
    degree_schools: &[String],
) -> HashSet<usize> {
    if degree_indices.len() <= 2 {
        return degree_indices.iter().copied().collect();
    }

    let mut best: Option<(HashSet<usize>, usize, bool)> = None;
    for i in 0..degree_indices.len() {
        for j in (i + 1)..degree_indices.len() {
            let pair = HashSet::from([degree_indices[i], degree_indices[j]]);
            let score = fulfillment_score_for_course_in_degree(
                per_degree,
                course,
                degree_indices[i],
                conc_contexts,
                taken,
                cu_map,
            ) + fulfillment_score_for_course_in_degree(
                per_degree,
                course,
                degree_indices[j],
                conc_contexts,
                taken,
                cu_map,
            );
            let has_undergrad = [degree_indices[i], degree_indices[j]]
                .iter()
                .any(|&idx| !is_graduate_degree(&degree_schools[idx]));
            let replace = match &best {
                None => true,
                Some((_, best_score, best_has_undergrad)) => {
                    score > *best_score
                        || (score == *best_score && has_undergrad && !*best_has_undergrad)
                }
            };
            if replace {
                best = Some((pair, score, has_undergrad));
            }
        }
    }
    best.map(|(pair, _, _)| pair).unwrap_or_default()
}

fn remove_course_from_degree_result(
    per_degree: &mut [DegreeValidationResult],
    course: &str,
    degree_idx: usize,
) {
    let (fulfilled, unfulfilled) = (
        &mut per_degree[degree_idx].fulfilled,
        &mut per_degree[degree_idx].unfulfilled,
    );
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

fn course_allocated_to_degree(
    course_id: &str,
    degree_idx: usize,
    claims: &HashMap<String, HashSet<usize>>,
) -> bool {
    if !course::is_valid_course_code(course_id) {
        return true;
    }
    claims
        .get(course_id)
        .map(|indices| indices.contains(&degree_idx))
        .unwrap_or(false)
}

pub fn filter_mapped_requirements_by_allocation(
    mapped_list: &mut [MappedRequirement],
    degree_idx: usize,
    claims: &HashMap<String, HashSet<usize>>,
) {
    for mapped in mapped_list {
        mapped.course_ids.retain(|course_id| {
            course_allocated_to_degree(course_id, degree_idx, claims)
        });
        if let Some(ref mut attr_rows) = mapped.attribute_fulfillment {
            for row in attr_rows.iter_mut() {
                row.course_ids.retain(|course_id| {
                    course_allocated_to_degree(course_id, degree_idx, claims)
                });
            }
            attr_rows.retain(|row| !row.course_ids.is_empty());
            if attr_rows.is_empty() {
                mapped.attribute_fulfillment = None;
            }
        }
    }
}

const CONCENTRATION_PRIORITY_WEIGHT: usize = 100;

#[derive(Debug, Clone)]
pub struct DegreeConcentrationContext {
    pub is_overlay: bool,
    pub selected_concentrations: Vec<String>,
    pub concentration_requirements: BTreeMap<String, Vec<Requirement>>,
    pub degree_requirements: Vec<Requirement>,
}

pub fn degree_concentration_context_from_major(
    requirements: &[Requirement],
    concentrations: &Option<BTreeMap<String, Vec<Requirement>>>,
    selected: &[String],
) -> DegreeConcentrationContext {
    let has_core = requirements_contain_concentration(requirements)
        || requirements
            .iter()
            .any(|r| r.get_category() == "Concentration");
    let is_overlay = !has_core && concentrations.is_some();
    DegreeConcentrationContext {
        is_overlay,
        selected_concentrations: selected.to_vec(),
        concentration_requirements: concentrations.clone().unwrap_or_default(),
        degree_requirements: requirements.to_vec(),
    }
}

fn overlay_concentration_courses_for_degree(
    ctx: &DegreeConcentrationContext,
    taken: &[String],
    cu_map: &HashMap<String, f64>,
    validation: Option<&DegreeValidationResult>,
) -> HashSet<String> {
    if !ctx.is_overlay {
        return HashSet::new();
    }

    let infos = extract_concentration_info(
        &ctx.degree_requirements,
        &Some(ctx.concentration_requirements.clone()),
        &ctx.selected_concentrations,
        &taken.to_vec(),
        cu_map,
        validation,
    );

    infos
        .iter()
        .flat_map(|ci| ci.matched_courses.iter().flatten())
        .cloned()
        .collect()
}

pub fn concentration_slots_for_course(
    ctx: &DegreeConcentrationContext,
    course: &str,
    taken: &[String],
    cu_map: &HashMap<String, f64>,
    validation: Option<&DegreeValidationResult>,
) -> usize {
    if !ctx.is_overlay {
        return 0;
    }

    extract_concentration_info(
        &ctx.degree_requirements,
        &Some(ctx.concentration_requirements.clone()),
        &ctx.selected_concentrations,
        &taken.to_vec(),
        cu_map,
        validation,
    )
    .iter()
    .flat_map(|ci| ci.matched_courses.iter().flatten())
    .filter(|c| *c == course)
    .count()
}

pub fn build_ug_concentration_claims(
    conc_contexts: &[DegreeConcentrationContext],
    degree_schools: &[String],
    per_degree: &[DegreeValidationResult],
    taken: &[String],
    cu_map: &HashMap<String, f64>,
) -> HashMap<String, HashSet<usize>> {
    let mut claims: HashMap<String, HashSet<usize>> = HashMap::new();

    for (degree_idx, ctx) in conc_contexts.iter().enumerate() {
        if is_graduate_degree(&degree_schools[degree_idx]) {
            continue;
        }
        let validation = per_degree.get(degree_idx);
        for course in overlay_concentration_courses_for_degree(ctx, taken, cu_map, validation) {
            claims.entry(course).or_default().insert(degree_idx);
        }
    }

    claims
}

pub fn merge_concentration_claims_into(
    allocations: &mut HashMap<String, HashSet<usize>>,
    ug_conc_claims: &HashMap<String, HashSet<usize>>,
) {
    for (course, indices) in ug_conc_claims {
        allocations
            .entry(course.clone())
            .or_default()
            .extend(indices.iter().copied());
    }
}

pub fn filter_concentration_info_by_claims(
    conc_info: &mut [ConcentrationInfo],
    degree_idx: usize,
    claims: &HashMap<String, HashSet<usize>>,
) {
    for ci in conc_info.iter_mut() {
        if ci.is_core {
            continue;
        }

        for (slot_idx, courses) in ci.matched_courses.iter_mut().enumerate() {
            courses.retain(|course_id| {
                claims
                    .get(course_id)
                    .map(|indices| indices.contains(&degree_idx))
                    .unwrap_or(false)
            });
            if let Some(fulfilled) = ci.requirement_fulfilled.get_mut(slot_idx) {
                *fulfilled = !courses.is_empty();
            }
        }

        ci.requirements_fulfilled = ci.requirement_fulfilled.iter().filter(|&&x| x).count();
    }
}

fn merge_concentration_into_allocations(
    allocations: &mut HashMap<String, HashSet<usize>>,
    conc_contexts: &[DegreeConcentrationContext],
    degree_schools: &[String],
    per_degree: &[DegreeValidationResult],
    taken: &[String],
    cu_map: &HashMap<String, f64>,
) {
    let ug_conc_claims = build_ug_concentration_claims(
        conc_contexts,
        degree_schools,
        per_degree,
        taken,
        cu_map,
    );
    merge_concentration_claims_into(allocations, &ug_conc_claims);
}

fn ug_concentration_priority(
    course: &str,
    ug_conc_claims: &HashMap<String, HashSet<usize>>,
) -> usize {
    ug_conc_claims
        .get(course)
        .map(|set| set.len())
        .unwrap_or(0)
}

pub fn build_allocations_from_fulfilled(
    per_degree: &[DegreeValidationResult],
) -> HashMap<String, HashSet<usize>> {
    let mut allocations: HashMap<String, HashSet<usize>> = HashMap::new();
    for (degree_idx, validation) in per_degree.iter().enumerate() {
        let mut record = |course: &String| {
            if course::is_valid_course_code(course) {
                allocations
                    .entry(course.clone())
                    .or_default()
                    .insert(degree_idx);
            }
        };
        for mapped in &validation.fulfilled {
            for course in &mapped.course_ids {
                record(course);
            }
        }
        for mapped in &validation.unfulfilled {
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
    per_degree: &mut [DegreeValidationResult],
    degree_schools: &[String],
    degree_majors: &[String],
    cu_map: &HashMap<String, f64>,
    conc_contexts: Option<&[DegreeConcentrationContext]>,
    taken: Option<&[String]>,
) -> CrossDegreeSummary {
    let mut allocations = build_allocations_from_fulfilled(per_degree);
    if let (Some(contexts), Some(taken)) = (conc_contexts, taken) {
        merge_concentration_into_allocations(
            &mut allocations,
            contexts,
            degree_schools,
            per_degree,
            taken,
            cu_map,
        );
    }
    let ug_conc_claims = match (conc_contexts, taken) {
        (Some(contexts), Some(taken)) => {
            build_ug_concentration_claims(contexts, degree_schools, per_degree, taken, cu_map)
        }
        _ => HashMap::new(),
    };

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
                    let keep = choose_best_two_degrees(
                        course,
                        &indices,
                        per_degree,
                        conc_contexts,
                        taken,
                        cu_map,
                        degree_schools,
                    );
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
                    let mut best_score = fulfillment_score_for_course_in_degree(
                        per_degree,
                        course,
                        best_idx,
                        conc_contexts,
                        taken,
                        cu_map,
                    );
                    for &idx in &grad_indices[1..] {
                        let score = fulfillment_score_for_course_in_degree(
                            per_degree,
                            course,
                            idx,
                            conc_contexts,
                            taken,
                            cu_map,
                        );
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
                        ug_concentration_priority(&a.0, &ug_conc_claims)
                            .cmp(&ug_concentration_priority(&b.0, &ug_conc_claims))
                            .then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
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
        if let (Some(contexts), Some(taken)) = (conc_contexts, taken) {
            merge_concentration_into_allocations(
                &mut allocations,
                contexts,
                degree_schools,
                per_degree,
                taken,
                cu_map,
            );
        }
    }

    let mut state = CrossDegreeState::new(degree_schools.to_vec(), degree_majors.to_vec());
    state.rebuild_from_allocations(&allocations, cu_map);
    state.ug_concentration_courses = ug_conc_claims;
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
    fn restriction_number_expands_to_duplicate_slots() {
        let attributes = attributes_data::create_attributes();
        let mut cu_map = HashMap::new();
        cu_map.insert("TEST 1000".to_string(), 1.0);
        cu_map.insert("TEST 1001".to_string(), 1.0);
        cu_map.insert("TEST 1002".to_string(), 1.0);
        cu_map.insert("TEST 1003".to_string(), 1.0);

        let slot = || Requirement::Restriction {
            category: Some("Test elective".to_string()),
            department: Some(vec!["TEST".to_string()]),
            cu: None,
            level: None,
            attr: None,
            excluding: None,
            number: 1,
            no_school: None,
        };

        let expanded = expand_restriction_slots(vec![Requirement::Restriction {
            category: Some("Test elective".to_string()),
            department: Some(vec!["TEST".to_string()]),
            cu: None,
            level: None,
            attr: None,
            excluding: None,
            number: 4,
            no_school: None,
        }]);
        let explicit = vec![slot(), slot(), slot(), slot()];

        let taken = vec![
            "TEST 1000".to_string(),
            "TEST 1001".to_string(),
            "TEST 1002".to_string(),
            "TEST 1003".to_string(),
        ];

        let expanded_validation =
            validate_courses_for_degree(expanded, &taken, &cu_map);
        let explicit_validation =
            validate_courses_for_degree(explicit, &taken, &cu_map);

        assert_eq!(
            expanded_validation.fulfilled.len(),
            explicit_validation.fulfilled.len()
        );
        assert_eq!(
            expanded_validation.unfulfilled.len(),
            explicit_validation.unfulfilled.len()
        );
    }

    #[test]
    fn restriction_excluding_attribute_blocks_matching_courses() {
        let attributes = attributes_data::create_attributes();
        // AFRC 0030 is tagged AIRE and AUFS; AUFS exclusion should win.
        assert!(!course_matches_restriction(
            "AFRC 0030",
            &None,
            &None,
            &Some(vec!["AIRE".to_string()]),
            &Some(vec!["AUFS".to_string()]),
            &None,
            &attributes,
        ));
        // BEPP 2010 is AIRE but not AUFS.
        assert!(course_matches_restriction(
            "BEPP 2010",
            &None,
            &None,
            &Some(vec!["AIRE".to_string()]),
            &Some(vec!["AUFS".to_string()]),
            &None,
            &attributes,
        ));
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
    fn validate_degree_fills_restriction_before_business_breadth() {
        let mut cu_map = HashMap::new();
        cu_map.insert("FNCE 2030".to_string(), 1.0);

        let requirements = vec![
            Requirement::AnyOf {
                category: Some("Business Breadth".to_string()),
                possibilities: vec![Requirement::Restriction {
                    category: None,
                    department: Some(vec!["FNCE".to_string()]),
                    cu: None,
                    level: None,
                    attr: None,
                    excluding: None,
                    number: 1,
                    no_school: None,
                }],
            },
            Requirement::Restriction {
                category: Some("WUCP elective".to_string()),
                department: Some(vec!["FNCE".to_string()]),
                cu: None,
                level: None,
                attr: None,
                excluding: None,
                number: 1,
                no_school: None,
            },
        ];

        let taken = vec!["FNCE 2030".to_string()];
        let validation = validate_courses_for_degree(requirements, &taken, &cu_map);
        let fulfilled = validation.fulfilled;
        let unfulfilled = validation.unfulfilled;

        assert_eq!(unfulfilled.len(), 1);
        assert_eq!(fulfilled.len(), 1);
        assert_eq!(fulfilled[0].course_ids, vec!["FNCE 2030".to_string()]);
        assert_eq!(fulfilled[0].requirement.get_category(), "WUCP elective");
        assert!(
            unfulfilled[0]
                .requirement
                .get_category()
                .to_lowercase()
                .contains("business breadth")
        );
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
        let validation = validate_courses_for_degree(requirements, &taken, &cu_map);
        let fulfilled = validation.fulfilled;
        let unfulfilled = validation.unfulfilled;

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
        let validation = validate_courses_for_degree(requirements, &taken, &cu_map);
        let fulfilled = validation.fulfilled;
        let unfulfilled = validation.unfulfilled;

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

        let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
        let fulfilled = validation.fulfilled;
        let unfulfilled = validation.unfulfilled;

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
            .expect("foundational CourseGroup should be present");
        assert!(foundational.partial);
        assert_eq!(foundational.course_ids, vec!["CIS 5190".to_string()]);
        assert!(foundational.committed_anyof_branch.is_none());
    }

    #[test]
    fn course_group_fulfills_three_of_four_foundational_areas() {
        use crate::seas_grad_data;

        let major = seas_grad_data::create_ms_robo_major();
        let taken = vec![
            "CIS 5190".to_string(),
            "MEAM 5200".to_string(),
            "ESE 5000".to_string(),
        ];
        let cu_map = HashMap::from([
            ("CIS 5190".to_string(), 1.0),
            ("MEAM 5200".to_string(), 1.0),
            ("ESE 5000".to_string(), 1.0),
        ]);

        let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
        let fulfilled = validation.fulfilled;
        let unfulfilled = validation.unfulfilled;

        let foundational = fulfilled
            .iter()
            .find(|m| m.requirement.get_category() == "Foundational Courses")
            .expect("foundational CourseGroup should be fulfilled");
        assert!(!foundational.partial);
        assert_eq!(foundational.course_ids.len(), 3);
        assert!(unfulfilled
            .iter()
            .all(|m| m.requirement.get_category() != "Foundational Courses"));
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
        let validation = validate_courses_for_degree(requirements, &taken, &cu_map);
        let fulfilled = validation.fulfilled;
        let unfulfilled = validation.unfulfilled;

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
            DegreeValidationResult {
                fulfilled: vec![mapped("CIS 1200"), mapped("CIS 1200")],
                unfulfilled: vec![],
                pool_coverage_info: vec![],
            },
            DegreeValidationResult {
                fulfilled: vec![mapped("CIS 1200")],
                unfulfilled: vec![],
                pool_coverage_info: vec![],
            },
            DegreeValidationResult {
                fulfilled: vec![mapped("CIS 1200")],
                unfulfilled: vec![],
                pool_coverage_info: vec![],
            },
        ];

        let summary = resolve_cross_degree_conflicts(
            &mut per_degree,
            &schools,
            &majors,
            &cu_map,
            None,
            None,
        );
        let kept = summary
            .course_allocations
            .get("CIS 1200")
            .map(|v| v.len())
            .unwrap_or(0);
        assert!(kept <= 2);
        assert!(summary.violations.is_empty());
    }

    #[test]
    fn ug_concentration_course_claimed_for_undergrad_degree() {
        use crate::major::resolve_major;

        let mut cu_map = HashMap::new();
        cu_map.insert("MEAM 5200".to_string(), 1.0);
        cu_map.insert("ESE 4210".to_string(), 1.0);

        let ee = resolve_major("SEAS", "EE", &["Robotics".to_string()]).expect("EE major");
        let ms_robo = resolve_major("SEAS_MS", "MS_ROBO", &[]).expect("MS Robotics");

        let taken = vec!["MEAM 5200".to_string(), "ESE 4210".to_string()];
        let ee_validation =
            validate_courses_for_degree(ee.requirements.clone(), &taken, &cu_map);
        let ms_validation =
            validate_courses_for_degree(ms_robo.requirements.clone(), &taken, &cu_map);

        let conc_contexts = vec![
            degree_concentration_context_from_major(
                &ee.requirements,
                &ee.concentrations,
                &["Robotics".to_string()],
            ),
            degree_concentration_context_from_major(
                &ms_robo.requirements,
                &ms_robo.concentrations,
                &[],
            ),
        ];

        let mut per_degree = vec![ee_validation, ms_validation];
        let schools = vec!["SEAS".to_string(), "SEAS_MS".to_string()];
        let majors = vec!["EE".to_string(), "MS_ROBO".to_string()];

        let ug_conc = build_ug_concentration_claims(
            &conc_contexts,
            &schools,
            &per_degree,
            &taken,
            &cu_map,
        );
        assert!(
            ug_conc
                .get("MEAM 5200")
                .map(|s| s.contains(&0))
                .unwrap_or(false),
            "MEAM 5200 should count toward UG EE via Robotics concentration"
        );

        resolve_cross_degree_conflicts(
            &mut per_degree,
            &schools,
            &majors,
            &cu_map,
            Some(&conc_contexts),
            Some(&taken),
        );

        let mut allocations = build_allocations_from_fulfilled(&per_degree);
        merge_concentration_claims_into(&mut allocations, &ug_conc);
        assert!(
            allocations
                .get("MEAM 5200")
                .map(|s| s.contains(&0))
                .unwrap_or(false),
            "MEAM 5200 should remain allocated to UG EE after conflict resolution"
        );
    }

    #[test]
    fn concentration_info_hides_courses_not_on_degree() {
        let mut conc_info = vec![ConcentrationInfo {
            name: "Robotics".to_string(),
            is_core: false,
            requirements_total: 1,
            requirements_fulfilled: 1,
            requirement_descriptions: vec!["Robotics elective".to_string()],
            requirement_fulfilled: vec![true],
            matched_courses: vec![vec!["MEAM 5200".to_string()]],
        }];

        let mut claims = HashMap::new();
        claims.insert("MEAM 5200".to_string(), HashSet::from([1]));

        filter_concentration_info_by_claims(&mut conc_info, 0, &claims);

        assert_eq!(conc_info[0].requirements_fulfilled, 0);
        assert!(conc_info[0].matched_courses[0].is_empty());
    }

    #[test]
    fn cas_pool_blocks_triple_constraint_reuse() {
        let mut attributes = attributes_data::create_attributes();
        let cu_map = HashMap::from([("MULTI 0100".to_string(), 1.0)]);
        let pool = vec!["MULTI 0100".to_string()];
        let constraints = vec![
            PoolConstraint {
                requirement: Requirement::Restriction {
                    category: Some("Group A".to_string()),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(vec!["ATTR_A".to_string()]),
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                count: 1,
                consumption_group: Some("group:a".to_string()),
            },
            PoolConstraint {
                requirement: Requirement::Restriction {
                    category: Some("Group B".to_string()),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(vec!["ATTR_B".to_string()]),
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                count: 1,
                consumption_group: Some("group:b".to_string()),
            },
            PoolConstraint {
                requirement: Requirement::Restriction {
                    category: Some("Group C".to_string()),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(vec!["ATTR_C".to_string()]),
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                count: 1,
                consumption_group: Some("group:c".to_string()),
            },
        ];

        for attr in ["ATTR_A", "ATTR_B", "ATTR_C"] {
            attributes
                .entry(attr.to_string())
                .or_default()
                .push("MULTI 0100".to_string());
        }

        let evaluations = evaluate_pool_constraints(&pool, &constraints, &attributes, &cu_map);
        let fulfilled = evaluations.iter().filter(|e| e.fulfilled).count();
        assert_eq!(
            fulfilled, 2,
            "one course may satisfy at most two pool constraints"
        );
    }

    #[test]
    fn cas_pool_allows_fa_sector_overlap() {
        let attributes = attributes_data::create_attributes();
        let cu_map = HashMap::from([("COLL 0200".to_string(), 1.0)]);
        let pool = vec!["COLL 0200".to_string()];
        let constraints = vec![
            PoolConstraint {
                requirement: Requirement::Restriction {
                    category: Some("Foundational Approaches — Quantitative Data Analysis".to_string()),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(vec!["AUQD".to_string()]),
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                count: 1,
                consumption_group: Some("cas:fa".to_string()),
            },
            PoolConstraint {
                requirement: Requirement::Restriction {
                    category: Some(
                        "Sectors of Knowledge — VII — Natural Sciences Across Disciplines".to_string(),
                    ),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(vec!["AUNM".to_string()]),
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                count: 1,
                consumption_group: Some("cas:sector".to_string()),
            },
        ];

        let evaluations = evaluate_pool_constraints(&pool, &constraints, &attributes, &cu_map);

        assert!(evaluations[0].fulfilled, "AUQD via COLL 0200");
        assert!(
            evaluations[1].fulfilled,
            "AUNM should reuse COLL 0200 across FA/Sector groups"
        );
        assert_eq!(evaluations[0].course_ids, evaluations[1].course_ids);
    }

    #[test]
    fn cas_pool_blocks_fa_fa_overlap() {
        let attributes = attributes_data::create_attributes();
        let cu_map = HashMap::from([("COLL 0200".to_string(), 1.0)]);

        let pool = vec!["COLL 0200".to_string()];
        let constraints = vec![
            PoolConstraint {
                requirement: Requirement::Restriction {
                    category: Some("Foundational Approaches - Quantitative Data Analysis".to_string()),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(vec!["AUQD".to_string()]),
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                count: 1,
                consumption_group: Some("cas:fa".to_string()),
            },
            PoolConstraint {
                requirement: Requirement::Restriction {
                    category: Some("Foundational Approaches - Formal Reasoning & Analysis".to_string()),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(vec!["AUFR".to_string()]),
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                count: 1,
                consumption_group: Some("cas:fa".to_string()),
            },
        ];

        let evaluations = evaluate_pool_constraints(&pool, &constraints, &attributes, &cu_map);

        assert!(evaluations[0].fulfilled);
        assert!(
            !evaluations[1].fulfilled,
            "one course cannot satisfy two Foundational Approaches"
        );
    }

    #[test]
    fn validate_emits_pool_coverage_for_cas_econ() {
        use crate::college_data::{self};

        let major = college_data::create_econ_major();
        let cu_map = HashMap::from([("WRIT 0100".to_string(), 1.0)]);
        let taken = vec!["WRIT 0100".to_string()];

        let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
        assert_eq!(validation.pool_coverage_info.len(), 1);
        assert_eq!(
            validation.pool_coverage_info[0].category,
            "General Education"
        );
    }

    #[test]
    fn wh_fl_pool_blocks_wufl_wucn_same_course() {
        use crate::wharton_data;

        let major = wharton_data::create_wh_fl_major(vec!["FNCE".to_string()]);
        let pool_req = major
            .requirements
            .iter()
            .find(|r| matches!(r, Requirement::CoursePool { .. }))
            .expect("WH_FL LAS pool");
        let Requirement::CoursePool { constraints, .. } = pool_req else {
            panic!("expected CoursePool");
        };

        let mut attributes = attributes_data::create_attributes();
        for attr in ["WUFL", "WUCN"] {
            attributes
                .entry(attr.to_string())
                .or_default()
                .push("SPAN 0100".to_string());
        }
        let cu_map = HashMap::from([("SPAN 0100".to_string(), 1.0)]);
        let pool = vec!["SPAN 0100".to_string()];

        let evaluations = evaluate_pool_constraints(&pool, constraints, &attributes, &cu_map);
        let cc_fl_fulfilled = evaluations
            .iter()
            .filter(|e| e.consumption_group == "wh:cc_fl" && e.fulfilled)
            .count();
        assert_eq!(
            cc_fl_fulfilled, 1,
            "WUFL and WUCN share wh:cc_fl — one course satisfies at most one slot"
        );
    }

    #[test]
    fn wh_fl_pool_allows_wucn_wuhm_overlap() {
        use crate::wharton_data;

        let major = wharton_data::create_wh_fl_major(vec!["FNCE".to_string()]);
        let pool_req = major
            .requirements
            .iter()
            .find(|r| matches!(r, Requirement::CoursePool { .. }))
            .expect("WH_FL LAS pool");
        let Requirement::CoursePool { constraints, .. } = pool_req else {
            panic!("expected CoursePool");
        };

        let mut attributes = attributes_data::create_attributes();
        for attr in ["WUCN", "WUHM"] {
            attributes
                .entry(attr.to_string())
                .or_default()
                .push("ANTH 0001".to_string());
        }
        let cu_map = HashMap::from([("ANTH 0001".to_string(), 1.0)]);
        let pool = vec!["ANTH 0001".to_string()];

        let evaluations = evaluate_pool_constraints(&pool, constraints, &attributes, &cu_map);
        assert!(
            evaluations
                .iter()
                .any(|e| e.consumption_group == "wh:cc_fl" && e.fulfilled && e.label == "WUCN"),
            "cross-cultural slot"
        );
        assert!(
            evaluations
                .iter()
                .any(|e| e.consumption_group == "wh:ssh" && e.fulfilled && e.label == "WUHM"),
            "humanities slot via CC double-count"
        );
    }

    #[test]
    fn wh_fl_pool_blocks_wuhm_wuss_same_course() {
        use crate::wharton_data;

        let major = wharton_data::create_wh_fl_major(vec!["FNCE".to_string()]);
        let pool_req = major
            .requirements
            .iter()
            .find(|r| matches!(r, Requirement::CoursePool { .. }))
            .expect("WH_FL LAS pool");
        let Requirement::CoursePool { constraints, .. } = pool_req else {
            panic!("expected CoursePool");
        };

        let mut attributes = attributes_data::create_attributes();
        for attr in ["WUHM", "WUSS"] {
            attributes
                .entry(attr.to_string())
                .or_default()
                .push("TEST 0001".to_string());
        }
        let cu_map = HashMap::from([("TEST 0001".to_string(), 1.0)]);
        let pool = vec!["TEST 0001".to_string()];

        let evaluations = evaluate_pool_constraints(&pool, constraints, &attributes, &cu_map);
        let ssh_fulfilled = evaluations
            .iter()
            .filter(|e| e.consumption_group == "wh:ssh" && e.fulfilled)
            .count();
        assert_eq!(
            ssh_fulfilled, 1,
            "WUHM and WUSS share wh:ssh — one course satisfies at most one SSH slot"
        );
    }

    #[test]
    fn collect_category_order_flattens_cas_pool_children() {
        use crate::college_data;

        let major = college_data::create_econ_major();
        let mut order = Vec::new();
        for req in &major.requirements {
            req.collect_category_order(&mut order);
        }
        assert_eq!(order.first().map(String::as_str), Some("Writing Seminar"));
        assert!(order.iter().any(|c| c == "General Education"));
        assert!(order.iter().any(|c| c == "Introductory Economics"));
        assert!(!order.iter().any(|c| c.starts_with("Sectors of Knowledge —")));
    }

    #[test]
    fn pool_constraint_instance_id_distinguishes_fixed_slots() {
        assert!(is_pool_constraint_instance_id(Some("1:c0")));
        assert!(is_pool_constraint_instance_id(Some("1:c12")));
        assert!(!is_pool_constraint_instance_id(Some("1:f0:c0")));
        assert!(!is_pool_constraint_instance_id(Some("1:f4:c0")));
        assert!(!is_pool_constraint_instance_id(Some("1:p0")));
        assert!(!is_pool_constraint_instance_id(Some("0")));
    }

    #[test]
    fn suggest_skips_pool_constraint_slots() {
        use crate::college_data;

        let major = college_data::create_econ_major();
        let cu_map = HashMap::new();
        let taken: Vec<String> = vec![];

        let validation = validate_courses_for_degree(major.requirements, &taken, &cu_map);
        let suggested = suggest_courses_for_requirements(
            &validation.unfulfilled,
            &taken,
            &cu_map,
            None,
            None,
        );

        assert!(
            validation
                .unfulfilled
                .iter()
                .any(|m| is_pool_constraint_instance_id(m.instance_id.as_deref())),
            "expected unfulfilled pool constraint rows for requirements panel"
        );
        assert!(
            suggested.iter().all(|m| {
                !is_pool_constraint_instance_id(m.instance_id.as_deref())
            }),
            "pool constraints must not become schedule slots"
        );
        assert!(
            suggested.iter().all(|m| {
                m.course_ids
                    .iter()
                    .all(|id| !is_pool_constraint_slot_id(id))
            }),
            "suggested slot ids must be pool slots only"
        );
    }
}