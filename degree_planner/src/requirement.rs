use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::Serialize;

use crate::course;
use crate::cross_degree::{
    self, CrossDegreeState, CrossDegreeSummary, detect_violations, is_graduate_degree,
    crosses_undergrad_grad, UNDERGRAD_GRAD_CU_LIMIT,
};
use crate::major::Major;
use crate::penn_data::{attributes_data, college_data};
use crate::penn_data::college_data::{
    CAS_DEGREE_CU, CAS_GENED_POOL_CATEGORY, CAS_UNRESTRICTED_ELECTIVES_CATEGORY,
};
use crate::penn_data::requirement_builders::unrestricted_elective;

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
    /// - `max_level` — maximum course number; defaults to 9000 when unset.
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
        max_level: Option<i32>,
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
const MAX_SCHEDULE_LISTED_COURSES: usize = 2;

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

/// Schedule grid label for a multi-option SingleCourse dashed requirement block.
pub fn format_schedule_single_course_label(possibilities: &[String]) -> String {
    if possibilities.is_empty() {
        return "(options not specified)".to_string();
    }
    if possibilities.len() == 1 {
        return possibilities[0].clone();
    }
    if possibilities.len() <= MAX_SCHEDULE_LISTED_COURSES {
        return possibilities.join("/");
    }
    let shown: Vec<String> = possibilities
        .iter()
        .take(MAX_SCHEDULE_LISTED_COURSES)
        .cloned()
        .collect();
    let more = possibilities.len() - MAX_SCHEDULE_LISTED_COURSES;
    format!("{} (+{more})", shown.join("/"))
}

fn format_schedule_level_clause(level: Option<i32>, max_level: Option<i32>) -> String {
    match (level, max_level) {
        (Some(min), Some(max)) if max != RESTRICTION_DEFAULT_MAX_LEVEL => {
            format!("{min}–{max}")
        }
        (Some(min), _) => format!("min level {min}"),
        (None, Some(max)) => format!("max level {max}"),
        (None, None) => String::new(),
    }
}

/// Compact restriction label for 1-CU schedule slots (no redundant CU prefix).
pub fn format_schedule_restriction_description(
    department: &Option<Vec<String>>,
    cu: &Option<i32>,
    level: &Option<i32>,
    max_level: &Option<i32>,
    attr: &Option<Vec<String>>,
    number: &i32,
    no_school: &Option<String>,
) -> String {
    let target_cu = restriction_required_cu(*number, cu);
    let mut parts: Vec<String> = Vec::new();

    if (target_cu - 1.0).abs() >= CU_EPS {
        parts.push(if (target_cu - target_cu.round()).abs() < CU_EPS {
            format!("{} CU", target_cu as i32)
        } else {
            format!("{target_cu} CU")
        });
    }

    if let Some(depts) = department {
        if !depts.is_empty() {
            parts.push(depts.join("/"));
        }
    }
    if let Some(attrs) = attr {
        if !attrs.is_empty() {
            parts.push(attrs.join("/"));
        }
    }

    let level_clause = format_schedule_level_clause(*level, *max_level);
    if !level_clause.is_empty() {
        parts.push(level_clause);
    }
    if let Some(school) = no_school {
        parts.push(format!("excl. {school}"));
    }

    if parts.is_empty() {
        "Unrestricted".to_string()
    } else {
        parts.join(" ")
    }
}

/// SingleCourse on the schedule: one possibility → concrete course; many → scoped placeholder slot.
pub fn normalize_suggested_schedule_ids(mapped: &mut MappedRequirement) {
    let Requirement::SingleCourse { possibilities, .. } = &mapped.requirement else {
        return;
    };
    match possibilities.len() {
        0 => {}
        1 => {
            mapped.course_ids = vec![possibilities[0].clone()];
        }
        _ => {
            if let Some(instance_id) = mapped.instance_id.as_deref() {
                if let Some(slot_id) = mapped
                    .requirement
                    .schedulable_placeholder_id(Some(instance_id))
                {
                    mapped.course_ids = vec![slot_id];
                }
            }
        }
    }
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
    if let Requirement::Restriction { department, .. } = req {
        if department
            .as_ref()
            .is_some_and(|d| d.iter().any(|dept| dept == "WRIT"))
        {
            // CAS writing is siloed — satisfy before pools can absorb WRIT courses.
            return (0, 0, req.specificity_score());
        }
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
            max_level,
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
                max_level: max_level.clone(),
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
pub(crate) fn expand_pool_fixed_slots(
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
    max_level: &Option<i32>,
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
                course, department, level, max_level, attr, excluding, no_school, attributes,
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
    max_level: &Option<i32>,
    attr: &Option<Vec<String>>,
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
    let level_clause = format_schedule_level_clause(*level, *max_level);
    if !level_clause.is_empty() {
        response.push(' ');
        response.push_str(&level_clause);
    }
    if let Some(attr_names) = attr {
        response.push_str(" from attribute ");
        response.push_str(&attr_names.join("/"));
    }
    if let Some(no_school_name) = no_school {
        response.push_str(" not from ");
        response.push_str(no_school_name);
    }
    response
}

/// Default upper bound for [`Requirement::Restriction`] course numbers when `max_level` is unset.
pub const RESTRICTION_DEFAULT_MAX_LEVEL: i32 = 9000;

pub fn effective_restriction_max_level(max_level: Option<i32>) -> i32 {
    max_level.unwrap_or(RESTRICTION_DEFAULT_MAX_LEVEL)
}

/// Whether a catalog course code satisfies a Restriction requirement.
pub fn course_matches_restriction(
    course: &str,
    department: &Option<Vec<String>>,
    level: &Option<i32>,
    max_level: &Option<i32>,
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
            "CAS" => vec![],
            "NURS" => vec!["NURS"],
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
        let course_level = course_id.parse::<i32>().unwrap_or(0);
        if course_level < *min_level {
            return false;
        }
        let cap = effective_restriction_max_level(*max_level);
        if course_level > cap {
            return false;
        }
    } else if let Some(max_only) = max_level {
        let course_level = course_id.parse::<i32>().unwrap_or(0);
        if course_level > *max_only {
            return false;
        }
    } else {
        let course_level = course_id.parse::<i32>().unwrap_or(0);
        if course_level > RESTRICTION_DEFAULT_MAX_LEVEL {
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

/// Map a pool coverage constraint instance (`29:c1`) to a schedulable flex slot (`29:p1`).
pub fn pool_constraint_to_flex_slot_key(
    constraint_key: &str,
    pool: &PoolCoverageInfo,
) -> Option<String> {
    if !is_pool_constraint_instance_id(Some(constraint_key)) {
        return None;
    }
    let mut parts = constraint_key.split(':');
    let pool_idx = parts.next()?.parse::<usize>().ok()?;
    if pool_idx != pool.pool_index {
        return None;
    }
    let ci = parts
        .next()
        .and_then(|s| s.strip_prefix('c'))
        .and_then(|n| n.parse::<i32>().ok())
        .unwrap_or(0);
    let flex_total = pool.flexible_slots_total.max(0);
    if flex_total == 0 {
        return None;
    }
    let flex_idx = (ci % flex_total).min(flex_total - 1);
    Some(format!("{pool_idx}:p{flex_idx}"))
}

/// Schedule/overlap label for a pool coverage constraint — always the pool category, not the constraint name.
pub fn pool_overlap_display_label(
    slot_key: &str,
    pool_coverage: &[PoolCoverageInfo],
) -> Option<String> {
    if !is_pool_constraint_instance_id(Some(slot_key)) {
        return None;
    }
    let pool_idx: usize = slot_key.split(':').next()?.parse().ok()?;
    let pool = pool_coverage.iter().find(|p| p.pool_index == pool_idx)?;
    Some(pool.category.clone())
}

/// Resolve an open-slot key to the schedulable pool flex slot when it is a coverage constraint.
pub fn effective_pool_schedule_slot_key(
    slot_key: &str,
    pool_coverage: &[PoolCoverageInfo],
) -> String {
    for pool in pool_coverage {
        if let Some(flex) = pool_constraint_to_flex_slot_key(slot_key, pool) {
            return flex;
        }
    }
    slot_key.to_string()
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
        max_level: None,
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

const BB_SLOT_FINGERPRINT: &str = "BB:Business_Breadth";

/// Business-breadth AnyOf slots use `req:{scope}:BB:Business_Breadth` when scoped.
pub fn business_breadth_slot_id(scope: Option<&str>) -> String {
    scoped_slot_id(scope, BB_SLOT_FINGERPRINT)
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
        if let Requirement::AnyOf { category, .. } = self {
            if Self::is_business_breadth_category(category.as_ref()) {
                if slot_id == business_breadth_slot_id(None) {
                    return true;
                }
                if let Some(rest) = slot_id.strip_prefix("req:") {
                    if let Some((scope, fp)) = rest.split_once(":BB:") {
                        if !scope.is_empty() && fp == "Business_Breadth" {
                            return self
                                .schedulable_placeholder_id(Some(scope))
                                .as_deref()
                                == Some(slot_id);
                        }
                    }
                }
                return false;
            }
        }
        if let Some(rest) = slot_id.strip_prefix("req:") {
            if let Some((scope, _fp)) = rest.split_once(":R:") {
                if !scope.is_empty() {
                    return self.requirement_slot_id(Some(scope)).as_deref() == Some(slot_id);
                }
            }
            if let Some((scope, _fp)) = rest.split_once(":S:") {
                if !scope.is_empty() {
                    return self
                        .schedulable_placeholder_id(Some(scope))
                        .as_deref()
                        == Some(slot_id);
                }
            }
            if let Some((scope, _fp)) = rest.split_once(":A:") {
                if !scope.is_empty() {
                    return self
                        .schedulable_placeholder_id(Some(scope))
                        .as_deref()
                        == Some(slot_id);
                }
            }
        }
        self.schedulable_placeholder_id(None).as_deref() == Some(slot_id)
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

    fn business_breadth_schedule_label() -> String {
        "1 WH Business Breadth".to_string()
    }

    /// Business breadth slots use a short schedule label instead of dept-level restriction text.
    pub fn business_breadth_label_for_slot(&self, slot_id: &str) -> Option<String> {
        match self {
            Requirement::AnyOf { category, .. } => {
                if !Self::is_business_breadth_category(category.as_ref()) {
                    return None;
                }
                if self.matches_slot_id(slot_id) {
                    return Some(Self::business_breadth_schedule_label());
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

    pub fn schedule_label_for_requirement(&self) -> String {
        match self {
            Requirement::SingleCourse { possibilities, .. } => {
                format_schedule_single_course_label(possibilities)
            }
            Requirement::Restriction {
                department,
                cu,
                level,
                max_level,
                attr,
                number,
                no_school,
                ..
            } => format_schedule_restriction_description(
                department,
                cu,
                level,
                max_level,
                attr,
                number,
                no_school,
            ),
            _ => self.create_requirement_description(),
        }
    }

    pub fn slot_label_for_id(&self, slot_id: &str) -> String {
        if let Some(label) = self.business_breadth_label_for_slot(slot_id) {
            return label;
        }
        if let Some(matched) = self.find_for_slot_id(slot_id) {
            return matched.schedule_label_for_requirement();
        }
        "Open requirement".to_string()
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

    /// Schedule placeholder for open requirements that would otherwise suggest a concrete course.
    pub fn schedulable_placeholder_id(&self, scope: Option<&str>) -> Option<String> {
        match self {
            Requirement::Restriction { .. } => self.requirement_slot_id(scope),
            Requirement::SingleCourse { possibilities, .. } => {
                let fp = format!(
                    "S:{}",
                    possibilities
                        .iter()
                        .map(|p| slot_scope_slug(p))
                        .collect::<Vec<_>>()
                        .join("/")
                );
                Some(scoped_slot_id(scope, &fp))
            }
            Requirement::AnyOf { category, .. } => {
                if Self::is_business_breadth_category(category.as_ref()) {
                    return Some(business_breadth_slot_id(scope));
                }
                let cat = category.as_deref().filter(|c| !c.is_empty())?;
                let fp = format!("A:{}", slot_scope_slug(cat));
                Some(scoped_slot_id(scope, &fp))
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
            Requirement::Restriction { category, department, cu, level, max_level, attr, excluding, no_school, number, .. } => {
                courses_fulfilling_restriction_cu(
                    taken,
                    department,
                    level,
                    max_level,
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
                    return Some(vec![business_breadth_slot_id(scope)]);
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
                max_level,
                attr,
                number,
                no_school,
                ..
            } => format_restriction_description(
                department, cu, level, max_level, attr, number, no_school,
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

/// CAS gen-ed coverage: FA constraints may use major courses freely.
/// Sector constraints may use at most **one** course that also fulfills a major requirement;
/// additional sector slots must use non-major courses.
/// Writing must already be excluded from both course lists by the caller.
pub fn evaluate_cas_pool_constraints(
    fa_courses: &[String],
    sector_courses: &[String],
    major_courses: &HashSet<String>,
    constraints: &[PoolConstraint],
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> Vec<PoolConstraintEvaluation> {
    let units = expanded_pool_constraint_units(constraints);
    let unit_count = units.len();
    let mut eval_order: Vec<usize> = (0..unit_count).collect();
    eval_order.sort_by_key(|&i| (pool_constraint_unit_priority(&units[i].0), i));

    let mut blocked_by_group: HashMap<String, HashSet<String>> = HashMap::new();
    let mut course_constraint_uses: HashMap<String, usize> = HashMap::new();
    let mut major_sector_double_counts: usize = 0;
    let mut results: Vec<Option<PoolConstraintEvaluation>> = vec![None; unit_count];

    for i in eval_order {
        let (req, group) = units[i].clone();
        let is_sector = group == "cas:sector" || group.starts_with("cas:sector");
        let blocked = blocked_by_group.get(&group).cloned().unwrap_or_default();
        let source = if is_sector {
            sector_courses
        } else {
            fa_courses
        };
        let mut available: Vec<String> = source
            .iter()
            .filter(|c| {
                if blocked.contains(*c) {
                    return false;
                }
                if course_constraint_uses.get(*c).copied().unwrap_or(0)
                    >= POOL_MAX_CONSTRAINT_USES_PER_COURSE
                {
                    return false;
                }
                // After one major↔sector double-count, further sectors need non-major courses.
                if is_sector
                    && major_sector_double_counts >= 1
                    && major_courses.contains(*c)
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        if is_sector {
            // Prefer non-major courses so the single double-count slot is used only if needed.
            available.sort_by_key(|c| major_courses.contains(c));
        }
        let label = constraint_short_label(&req);

        results[i] = Some(match req.fulfills_requirement(&available, attributes, cu_map) {
            Some(courses) => {
                blocked_by_group
                    .entry(group.clone())
                    .or_default()
                    .extend(courses.iter().cloned());
                for course in &courses {
                    *course_constraint_uses.entry(course.clone()).or_insert(0) += 1;
                    if is_sector && major_courses.contains(course) {
                        major_sector_double_counts += 1;
                    }
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

pub(crate) fn course_improves_cas_pool_coverage(
    course: &str,
    fa_courses: &[String],
    sector_courses: &[String],
    major_courses: &HashSet<String>,
    constraints: &[PoolConstraint],
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> bool {
    let before = evaluate_cas_pool_constraints(
        fa_courses,
        sector_courses,
        major_courses,
        constraints,
        attributes,
        cu_map,
    )
    .iter()
    .filter(|e| e.fulfilled)
    .count();
    let mut fa = fa_courses.to_vec();
    fa.push(course.to_string());
    let mut sector = sector_courses.to_vec();
    sector.push(course.to_string());
    let after = evaluate_cas_pool_constraints(
        &fa,
        &sector,
        major_courses,
        constraints,
        attributes,
        cu_map,
    )
    .iter()
    .filter(|e| e.fulfilled)
    .count();
    after > before
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

pub(crate) fn build_pool_coverage_info(
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
                requirement: e.requirement.clone(),
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

fn pool_slot_has_valid_course(mapped: &MappedRequirement) -> bool {
    mapped
        .course_ids
        .iter()
        .any(|c| course::is_valid_course_code(c))
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
            m.instance_id.as_deref() == Some(child_id.as_str()) && pool_slot_has_valid_course(m)
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
                courses.extend(
                    mapped
                        .course_ids
                        .iter()
                        .filter(|c| course::is_valid_course_code(c))
                        .cloned(),
                );
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
    let attributes = attributes_data::attributes();
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

    pub fn mapped_for_instance(&self, instance_id: &str) -> Option<&MappedRequirement> {
        self.unfulfilled
            .iter()
            .find(|m| m.instance_id.as_deref() == Some(instance_id))
            .or_else(|| {
                self.fulfilled
                    .iter()
                    .find(|m| m.partial && m.instance_id.as_deref() == Some(instance_id))
            })
    }
}

/// True when `course` is a named option on a SingleCourse (possibly under AnyOf), not via attributes.
pub fn requirement_explicitly_lists_course(req: &Requirement, course: &str) -> bool {
    match req {
        Requirement::SingleCourse { possibilities, .. } => {
            possibilities.iter().any(|p| p == course)
        }
        Requirement::AnyOf { possibilities, .. } => possibilities
            .iter()
            .any(|child| requirement_explicitly_lists_course(child, course)),
        _ => false,
    }
}

/// Whether a shared course can count toward this requirement in a cross-degree overlap.
/// Accepts explicit lists plus attribute/department restrictions that include the course.
pub fn requirement_accepts_shared_course(req: &Requirement, course: &str) -> bool {
    if requirement_explicitly_lists_course(req, course) {
        return true;
    }
    let attributes = attributes_data::attributes();
    match req {
        Requirement::Restriction {
            department,
            level,
            max_level,
            attr,
            excluding,
            no_school,
            ..
        } => course_matches_restriction(
            course,
            department,
            level,
            max_level,
            attr,
            excluding,
            no_school,
            &attributes,
        ),
        Requirement::AnyOf { possibilities, .. } => possibilities
            .iter()
            .any(|child| requirement_accepts_shared_course(child, course)),
        Requirement::AllOf { requirements, .. } => requirements
            .iter()
            .any(|child| requirement_accepts_shared_course(child, course)),
        Requirement::CourseGroup { possibilities, .. } => possibilities
            .iter()
            .any(|child| requirement_accepts_shared_course(child, course)),
        _ => false,
    }
}

/// finding whether taken fulfills degree and to what extent
pub fn validate_courses_for_degree(
    requirements: Vec<Requirement>,
    taken: &Vec<String>,
    cu_map: &HashMap<String, f64>,
) -> DegreeValidationResult {
    let attributes = attributes_data::attributes();
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
                    let courses = try_fulfill_pool_flexible_slot(
                        &flex_req,
                        &constraints,
                        &pool_courses,
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
    let attributes = attributes_data::attributes();
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
        max_level,
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
                    max_level,
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

pub(crate) fn new_mapped_requirement(
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

pub(crate) fn try_fulfill_or_partial_base(
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

fn course_fills_unmet_pool_need(
    course: &str,
    pool_courses: &[String],
    constraints: &[PoolConstraint],
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> bool {
    let fulfilled_before = evaluate_pool_constraints(pool_courses, constraints, attributes, cu_map)
        .iter()
        .filter(|e| e.fulfilled)
        .count();
    let mut with_course = pool_courses.to_vec();
    with_course.push(course.to_string());
    let fulfilled_after = evaluate_pool_constraints(&with_course, constraints, attributes, cu_map)
        .iter()
        .filter(|e| e.fulfilled)
        .count();
    fulfilled_after > fulfilled_before
}

/// Pool flexible slots only absorb courses that satisfy an unmet pool coverage need.
/// Other courses remain for unrestricted electives outside the pool.
fn try_fulfill_pool_flexible_slot(
    flex_req: &Requirement,
    constraints: &[PoolConstraint],
    pool_courses: &[String],
    taken: &mut Vec<String>,
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
    child_id: Option<String>,
    fulfilled: &mut Vec<MappedRequirement>,
    unfulfilled: &mut Vec<MappedRequirement>,
) -> Vec<String> {
    if constraints.is_empty() {
        unfulfilled.push(new_mapped_requirement(
            flex_req.clone(),
            vec![],
            child_id,
            attributes,
        ));
        return vec![];
    }

    if let Some(course_idx) = taken.iter().position(|c| {
        course_fills_unmet_pool_need(c, pool_courses, constraints, attributes, cu_map)
    }) {
        let course = taken.remove(course_idx);
        fulfilled.push(new_mapped_requirement(
            flex_req.clone(),
            vec![course.clone()],
            child_id,
            attributes,
        ));
        return vec![course];
    }

    unfulfilled.push(new_mapped_requirement(
        flex_req.clone(),
        vec![],
        child_id,
        attributes,
    ));
    vec![]
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
    pub requirement: Requirement,
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
    let attributes = attributes_data::attributes();

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

/// Formerly patched independent per-major CAS validation. College-wide assignment is now
/// handled by [`assign_cas_college`]; this is a no-op kept for call-site compatibility in
/// older tests.
pub fn reconcile_cas_college_double_major_claims(
    _per_degree: &mut [DegreeValidationResult],
    _degree_schools: &[String],
) {
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

/// Drop courses from minor fulfillment that are allocated to graduate degree(s) only.
pub fn filter_minor_mapped_requirements(
    mapped_list: &mut [MappedRequirement],
    major_claims: &HashMap<String, HashSet<usize>>,
    major_degree_schools: &[String],
) {
    for mapped in mapped_list {
        mapped.course_ids.retain(|course_id| {
            cross_degree::course_may_count_toward_minor(
                course_id,
                major_claims,
                major_degree_schools,
            )
        });
        if let Some(ref mut attr_rows) = mapped.attribute_fulfillment {
            for row in attr_rows.iter_mut() {
                row.course_ids.retain(|course_id| {
                    cross_degree::course_may_count_toward_minor(
                        course_id,
                        major_claims,
                        major_degree_schools,
                    )
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
    excluded_degree_indices: Option<&HashSet<usize>>,
) -> HashMap<String, HashSet<usize>> {
    let mut allocations: HashMap<String, HashSet<usize>> = HashMap::new();
    for (degree_idx, validation) in per_degree.iter().enumerate() {
        if excluded_degree_indices.is_some_and(|ex| ex.contains(&degree_idx)) {
            continue;
        }
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
    excluded_degree_indices: Option<&HashSet<usize>>,
) -> CrossDegreeSummary {
    let mut allocations =
        build_allocations_from_fulfilled(per_degree, excluded_degree_indices);
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
        allocations = build_allocations_from_fulfilled(per_degree, excluded_degree_indices);
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

// ── College-wide CAS requirement assignment ─────────────────────────────────
//
// Order: Writing (exclusive) → Majors (shared bag, dual-major overlap) → Gen-Ed
// (FA may use major courses; remaining sectors may not) → residual Unrestricted to 36 CU.

pub struct CasCollegeAssignResult {
    pub per_degree: Vec<(usize, DegreeValidationResult)>,
    pub primary_flexible_slots: i32,
    pub primary_unrestricted_count: i32,
    pub college_auto_sectors: Vec<String>,
    pub college_constraints: Vec<PoolConstraint>,
}

fn cas_pool_fixed_slots(major: &Major) -> Option<&[Requirement]> {
    let (_, _) = college_data::cas_gened_pool(major)?;
    major.requirements.iter().find_map(|req| {
        if let Requirement::CoursePool {
            category,
            fixed_slots,
            ..
        } = req
        {
            if category.as_deref() == Some(CAS_GENED_POOL_CATEGORY) {
                return Some(fixed_slots.as_slice());
            }
        }
        None
    })
}

fn fulfill_from_available(
    req: &Requirement,
    available: &[String],
    attributes: &HashMap<String, Vec<String>>,
    cu_map: &HashMap<String, f64>,
) -> Option<Vec<String>> {
    req.fulfills_requirement(&available.to_vec(), attributes, cu_map)
}

/// Assign CAS college requirements for one or more CAS majors (primary first).
/// `concentrations[i]` matches `majors[i]` for sector auto-completion overrides.
pub fn assign_cas_college(
    majors: &[(usize, &Major)],
    concentrations: &[&[String]],
    taken: &[String],
    cu_map: &HashMap<String, f64>,
) -> CasCollegeAssignResult {
    assert!(!majors.is_empty(), "assign_cas_college requires at least one CAS major");
    let attributes = attributes_data::attributes();
    let mut bag: Vec<String> = taken.to_vec();

    let major_refs: Vec<&Major> = majors.iter().map(|(_, m)| *m).collect();
    let mut seen = BTreeSet::new();
    let mut college_auto_sectors = Vec::new();
    for (i, (_, major)) in majors.iter().enumerate() {
        let conc = concentrations
            .get(i)
            .and_then(|c| c.first())
            .map(|s| s.as_str());
        for attr in college_data::cas_auto_completed_sectors_for(&major.short_name, conc) {
            if seen.insert(attr.clone()) {
                college_auto_sectors.push(attr);
            }
        }
    }
    let college_constraints = college_data::cas_pool_constraints(&college_auto_sectors);

    // ── 1. Writing (exclusive, primary only) ────────────────────────────────
    let writing_req = majors[0]
        .1
        .requirements
        .first()
        .cloned()
        .expect("CAS major has writing requirement");
    let mut writing_mapped: Option<MappedRequirement> = None;
    if let Some(courses) = writing_req.fulfills_requirement(&bag, &attributes, cu_map) {
        bag.retain(|c| !courses.contains(c));
        writing_mapped = Some(new_mapped_requirement(
            writing_req.clone(),
            courses,
            Some("0".to_string()),
            &attributes,
        ));
    }
    let writing_unfulfilled = writing_mapped.is_none().then(|| {
        new_mapped_requirement(writing_req, vec![], Some("0".to_string()), &attributes)
    });

    // ── 2. Majors (shared bag; overlap courses kept on both) ────────────────
    let mut shared_major_courses: HashSet<String> = HashSet::new();
    let mut per_major_fulfilled: Vec<Vec<MappedRequirement>> = vec![Vec::new(); majors.len()];
    let mut per_major_unfulfilled: Vec<Vec<MappedRequirement>> = vec![Vec::new(); majors.len()];
    let mut per_major_course_sets: Vec<HashSet<String>> = vec![HashSet::new(); majors.len()];

    for (mi, (_, major)) in majors.iter().enumerate() {
        let Some(fixed_slots) = cas_pool_fixed_slots(major) else {
            continue;
        };
        for (fi, ci, slot_req) in expand_pool_fixed_slots(fixed_slots.to_vec()) {
            let child_id = Some(format!("1:f{fi}:c{ci}"));

            if let Some(courses) = fulfill_from_available(&slot_req, &bag, &attributes, cu_map) {
                bag.retain(|c| !courses.contains(c));
                for c in &courses {
                    shared_major_courses.insert(c.clone());
                    per_major_course_sets[mi].insert(c.clone());
                }
                per_major_fulfilled[mi].push(new_mapped_requirement(
                    slot_req,
                    courses,
                    child_id,
                    &attributes,
                ));
                continue;
            }

            // Cross-major overlap only: a course already claimed by *this* major must not
            // fill another of its slots (no within-major double-counting in CAS).
            let other_majors: Vec<String> = shared_major_courses
                .iter()
                .filter(|c| !per_major_course_sets[mi].contains(*c))
                .cloned()
                .collect();
            if let Some(courses) =
                fulfill_from_available(&slot_req, &other_majors, &attributes, cu_map)
            {
                for c in &courses {
                    per_major_course_sets[mi].insert(c.clone());
                }
                per_major_fulfilled[mi].push(new_mapped_requirement(
                    slot_req,
                    courses,
                    child_id,
                    &attributes,
                ));
                continue;
            }

            let courses = try_fulfill_or_partial_base(
                &slot_req,
                &mut bag,
                &attributes,
                cu_map,
                child_id,
                &mut per_major_fulfilled[mi],
                &mut per_major_unfulfilled[mi],
            );
            for c in &courses {
                if course::is_valid_course_code(c) {
                    shared_major_courses.insert(c.clone());
                    per_major_course_sets[mi].insert(c.clone());
                }
            }
        }
    }

    // Overlap savings: courses claimed by 2+ majors.
    let mut course_major_count: HashMap<String, usize> = HashMap::new();
    for set in &per_major_course_sets {
        for c in set {
            *course_major_count.entry(c.clone()).or_insert(0) += 1;
        }
    }
    let overlap_savings = course_major_count.values().filter(|&&n| n >= 2).count() as i32;
    let effective_major_cu =
        college_data::cas_effective_combined_major_cu(&major_refs, overlap_savings);

    // ── 3. Gen-Ed coverage ──────────────────────────────────────────────────
    // FA + sector candidates start as major courses. At most one major course may
    // also satisfy a sector; prefer non-major (bag) courses for additional sectors.
    let major_course_vec: Vec<String> = shared_major_courses.iter().cloned().collect();
    let mut fa_courses = major_course_vec.clone();
    let mut sector_courses = major_course_vec.clone();

    let mut flex_fulfilled: Vec<MappedRequirement> = Vec::new();
    let mut absorbed: Vec<String> = Vec::new();
    loop {
        let Some(idx) = bag.iter().position(|c| {
            course_improves_cas_pool_coverage(
                c,
                &fa_courses,
                &sector_courses,
                &shared_major_courses,
                &college_constraints,
                &attributes,
                cu_map,
            )
        }) else {
            break;
        };
        let course = bag.remove(idx);
        absorbed.push(course.clone());
        fa_courses.push(course.clone());
        sector_courses.push(course.clone());
        let pi = flex_fulfilled.len();
        let flex_req = pool_flexible_slot_requirement(CAS_GENED_POOL_CATEGORY, pi);
        flex_fulfilled.push(new_mapped_requirement(
            flex_req,
            vec![course],
            Some(format!("1:p{pi}")),
            &attributes,
        ));
    }

    let evaluations = evaluate_cas_pool_constraints(
        &fa_courses,
        &sector_courses,
        &shared_major_courses,
        &college_constraints,
        &attributes,
        cu_map,
    );

    let open_coverage = evaluations.iter().filter(|e| !e.fulfilled).count() as i32;
    let flex_filled = flex_fulfilled.len() as i32;
    let flex_total = flex_filled + open_coverage;
    let remaining_after_major = (CAS_DEGREE_CU - 1 - effective_major_cu).max(0);
    let primary_flexible_slots = flex_total.min(remaining_after_major);
    // If CU budget is tighter than open coverage, drop excess open flex slots.
    let open_flex_slots = (primary_flexible_slots - flex_filled).max(0);
    let primary_unrestricted_count =
        (remaining_after_major - primary_flexible_slots).max(0);

    let mut flex_unfulfilled: Vec<MappedRequirement> = Vec::new();
    for i in 0..open_flex_slots as usize {
        let pi = flex_filled as usize + i;
        let flex_req = pool_flexible_slot_requirement(CAS_GENED_POOL_CATEGORY, pi);
        flex_unfulfilled.push(new_mapped_requirement(
            flex_req,
            vec![],
            Some(format!("1:p{pi}")),
            &attributes,
        ));
    }

    let mut constraint_fulfilled = Vec::new();
    let mut constraint_unfulfilled = Vec::new();
    for (ci, eval) in evaluations.iter().enumerate() {
        let child_id = Some(format!("1:c{ci}"));
        let mapped = new_mapped_requirement(
            eval.requirement.clone(),
            eval.course_ids.clone(),
            child_id,
            &attributes,
        );
        if eval.fulfilled {
            constraint_fulfilled.push(mapped);
        } else {
            constraint_unfulfilled.push(mapped);
        }
    }

    // ── 4. Unrestricted residual ────────────────────────────────────────────
    let unrest_start_idx = 2usize; // writing=0, pool=1
    let mut unrest_fulfilled = Vec::new();
    let mut unrest_unfulfilled = Vec::new();
    for i in 0..primary_unrestricted_count as usize {
        let instance_id = Some((unrest_start_idx + i).to_string());
        let req = unrestricted_elective(CAS_UNRESTRICTED_ELECTIVES_CATEGORY);
        if let Some(courses) = req.fulfills_requirement(&bag, &attributes, cu_map) {
            bag.retain(|c| !courses.contains(c));
            unrest_fulfilled.push(new_mapped_requirement(
                req,
                courses,
                instance_id,
                &attributes,
            ));
        } else {
            unrest_unfulfilled.push(new_mapped_requirement(
                req,
                vec![],
                instance_id,
                &attributes,
            ));
        }
    }

    // Pool courses for coverage info = major + absorbed gen-ed flex.
    let mut pool_courses = major_course_vec;
    pool_courses.extend(absorbed.iter().cloned());

    let pool_coverage = vec![build_pool_coverage_info(
        1,
        Some(CAS_GENED_POOL_CATEGORY.to_string()),
        pool_courses,
        cas_pool_fixed_slots(majors[0].1)
            .map(|s| expand_pool_fixed_slots(s.to_vec()).len() as i32)
            .unwrap_or(0),
        per_major_fulfilled[0]
            .iter()
            .filter(|m| !m.course_ids.is_empty())
            .count() as i32,
        primary_flexible_slots,
        flex_filled,
        &evaluations,
    )];

    // ── Assemble per-degree results ─────────────────────────────────────────
    let mut per_degree = Vec::new();
    for (mi, (degree_idx, _)) in majors.iter().enumerate() {
        let is_primary = mi == 0;
        let mut fulfilled = Vec::new();
        let mut unfulfilled = Vec::new();

        if is_primary {
            if let Some(w) = writing_mapped.clone() {
                fulfilled.push(w);
            } else if let Some(w) = writing_unfulfilled.clone() {
                unfulfilled.push(w);
            }
        }

        fulfilled.extend(per_major_fulfilled[mi].clone());
        unfulfilled.extend(per_major_unfulfilled[mi].clone());

        if is_primary {
            fulfilled.extend(flex_fulfilled.clone());
            unfulfilled.extend(flex_unfulfilled.clone());
            fulfilled.extend(constraint_fulfilled.clone());
            unfulfilled.extend(constraint_unfulfilled.clone());
            fulfilled.extend(unrest_fulfilled.clone());
            unfulfilled.extend(unrest_unfulfilled.clone());
        }

        let coverage = if is_primary {
            pool_coverage.clone()
        } else {
            Vec::new()
        };

        per_degree.push((
            *degree_idx,
            DegreeValidationResult {
                fulfilled,
                unfulfilled,
                pool_coverage_info: coverage,
            },
        ));
    }

    CasCollegeAssignResult {
        per_degree,
        primary_flexible_slots,
        primary_unrestricted_count,
        college_auto_sectors,
        college_constraints,
    }
}

