use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::course;
use crate::cross_degree::{cross_degree_optimizer_applicable, CrossDegreeState};
use crate::major::Major;
use crate::penn_data::college_data;
use crate::penn_data::{attributes_data, courses_data};
use crate::requirement::{
    course_matches_restriction, DegreeValidationResult, MappedRequirement, Requirement,
};

// ── Catalog index (fast restriction candidate lookup) ─────────────────────────

/// Inverted catalog indexes for fast restriction candidate lookup.
#[derive(Debug, Clone)]
struct CatalogIndex {
    all_courses: Vec<String>,
    courses_by_attr: HashMap<String, HashSet<String>>,
    courses_by_dept: HashMap<String, HashSet<String>>,
}

impl CatalogIndex {
    fn build() -> Self {
        let attributes = attributes_data::create_attributes();
        let mut courses_by_attr: HashMap<String, HashSet<String>> = HashMap::new();
        for (attr, courses) in &attributes {
            courses_by_attr
                .entry(attr.clone())
                .or_default()
                .extend(courses.iter().cloned());
        }

        let mut courses_by_dept: HashMap<String, HashSet<String>> = HashMap::new();
        let mut all_courses = Vec::new();
        for c in courses_data::all_courses() {
            if !course::is_valid_course_code(&c.course_code) {
                continue;
            }
            all_courses.push(c.course_code.clone());
            courses_by_dept
                .entry(c.dept_code.clone())
                .or_default()
                .insert(c.course_code.clone());
        }
        all_courses.sort();

        Self {
            all_courses,
            courses_by_attr,
            courses_by_dept,
        }
    }

    fn unrestricted_undergrad_set(&self) -> HashSet<String> {
        self.all_courses
            .iter()
            .filter(|c| !course::is_graduate_level(c))
            .cloned()
            .collect()
    }

    fn candidates_for_restriction(
        &self,
        department: &Option<Vec<String>>,
        level: &Option<i32>,
        attr: &Option<Vec<String>>,
        excluding: &Option<Vec<String>>,
        no_school: &Option<String>,
        taken: &HashSet<String>,
    ) -> HashSet<String> {
        let attributes = attributes_data::create_attributes();
        let mut base: Option<HashSet<String>> = None;

        if let Some(attrs) = attr.as_ref().filter(|a| !a.is_empty()) {
            let mut union = HashSet::new();
            for name in attrs {
                if let Some(set) = self.courses_by_attr.get(name) {
                    union.extend(set.iter().cloned());
                }
            }
            base = Some(union);
        } else if let Some(depts) = department.as_ref().filter(|d| !d.is_empty()) {
            let mut union = HashSet::new();
            for dept in depts {
                if let Some(set) = self.courses_by_dept.get(dept) {
                    union.extend(set.iter().cloned());
                }
            }
            base = Some(union);
        } else if no_school.is_some() {
            base = Some(self.unrestricted_undergrad_set());
        } else {
            base = Some(self.unrestricted_undergrad_set());
        }

        let mut out = base.unwrap_or_default();
        out.retain(|c| {
            !taken.contains(c)
                && course_matches_restriction(
                    c,
                    department,
                    level,
                    attr,
                    excluding,
                    no_school,
                    &attributes,
                )
        });
        out
    }

    fn candidates_for_one_of(
        &self,
        possibilities: &[String],
        taken: &HashSet<String>,
    ) -> HashSet<String> {
        possibilities
            .iter()
            .filter(|c| course::is_valid_course_code(c) && !taken.contains(*c))
            .cloned()
            .collect()
    }
}

// ── Course matcher (requirement slot → catalog predicate) ─────────────────────

/// Compiled predicate: which catalog courses can satisfy a single open slot.
#[derive(Debug, Clone)]
enum CourseMatcher {
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
    fn specificity_score(&self) -> usize {
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

fn compile_matcher(req: &Requirement, committed_branch: Option<usize>) -> CourseMatcher {
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

fn course_satisfies_matcher(
    matcher: &CourseMatcher,
    course: &str,
    attributes: &HashMap<String, Vec<String>>,
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
        CourseMatcher::Unrestricted => course::is_valid_course_code(course),
    }
}

fn candidates_for_matcher(
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

// ── Cross-degree overlap discovery ───────────────────────────────────────────

const MAX_SUGGESTED_COURSES: usize = 12;
const MAX_CANDIDATES_PER_SLOT: usize = 800;
/// SingleCourse slots with at most this many named options participate in overlap discovery.
const MAX_EXPLICIT_ONEOF_OVERLAP: usize = 40;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OverlapSlotRef {
    pub degree_index: usize,
    pub slot_key: String,
    pub label: String,
    pub school: String,
    pub major: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlapOpportunity {
    pub slots: Vec<OverlapSlotRef>,
    pub suggested_courses: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlapPair {
    pub slots: Vec<OverlapSlotRef>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlapPlan {
    pub opportunities: Vec<OverlapOpportunity>,
    /// `"{degree_index}:{slot_key}"` → overlap course suggestions for UI hover.
    pub hints_by_slot: HashMap<String, Vec<String>>,
    /// Slot pairs to show as one grouped requirement block on the schedule (no auto course).
    pub pairs: Vec<OverlapPair>,
    /// `"{degree_index}:{slot_key}"` → why overlap applies on this row.
    pub slot_explanations: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct OpenSlot {
    degree_index: usize,
    slot_key: String,
    label: String,
    matcher: CourseMatcher,
    consumption_group: Option<String>,
    gened_attr: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlapScheduleGroupMember {
    pub schedule_slot_id: String,
    pub label: String,
    pub degree_index: usize,
    pub school: String,
    pub major: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlapScheduleGroup {
    pub group_id: String,
    pub members: Vec<OverlapScheduleGroupMember>,
    pub explanation: String,
}

pub fn overlap_group_schedule_id(slots: &[OverlapSlotRef]) -> String {
    let mut parts: Vec<String> = slots
        .iter()
        .map(|s| format!("{}@{}", s.degree_index, s.slot_key))
        .collect();
    parts.sort();
    format!("req:overlap:{}", parts.join("+"))
}

pub fn hint_key(degree_index: usize, slot_key: &str) -> String {
    format!("{degree_index}:{slot_key}")
}

pub fn is_overlap_schedule_group_id(slot_id: &str) -> bool {
    slot_id.starts_with("req:overlap:")
}

fn attr_from_requirement(req: &Requirement) -> Option<String> {
    if let Requirement::Restriction { attr, .. } = req {
        attr.as_ref().and_then(|a| a.first().cloned())
    } else {
        None
    }
}

fn slot_label(req: &Requirement) -> String {
    let cat = req.get_category();
    if cat.is_empty() {
        req.create_requirement_description()
    } else {
        cat
    }
}

fn push_open_from_mapped(slots: &mut Vec<OpenSlot>, degree_index: usize, mapped: &MappedRequirement) {
    if !mapped.partial
        && mapped
            .course_ids
            .iter()
            .any(|c| course::is_valid_course_code(c))
    {
        return;
    }
    let Some(instance_id) = mapped.instance_id.as_ref() else {
        return;
    };
    let matcher = compile_matcher(&mapped.requirement, mapped.committed_anyof_branch);
    slots.push(OpenSlot {
        degree_index,
        slot_key: instance_id.clone(),
        label: slot_label(&mapped.requirement),
        matcher,
        consumption_group: consumption_group_for_requirement(&mapped.requirement),
        gened_attr: attr_from_requirement(&mapped.requirement),
    });
}

fn consumption_group_for_requirement(req: &Requirement) -> Option<String> {
    if let Requirement::Restriction { department, .. } = req {
        if department
            .as_ref()
            .is_some_and(|d| d.iter().any(|dept| dept == "WRIT"))
        {
            return Some("university:writ".to_string());
        }
    }
    let cat = req.get_category();
    if cat.starts_with("Foundational Approaches") {
        return Some("cas:fa".to_string());
    }
    if cat.starts_with("Sectors of Knowledge") {
        return Some("cas:sector".to_string());
    }
    if cat.to_lowercase().contains("non-wharton") {
        return Some("wh:non_wh".to_string());
    }
    if cat.contains("Cross-Cultural") {
        return Some("wh:cross_cultural".to_string());
    }
    if cat.contains("WUHM") || cat.contains("WUSS") || cat.contains("WUNM") {
        return Some("wh:ssh".to_string());
    }
    if let Requirement::Restriction { attr, .. } = req {
        if let Some(attrs) = attr {
            if attrs
                .iter()
                .any(|a| a == "WUHM" || a == "WUSS" || a == "WUNM")
            {
                return Some("wh:ssh".to_string());
            }
            if attrs.iter().any(|a| a == "WUCN" || a == "WUCU") {
                return Some("wh:cross_cultural".to_string());
            }
        }
    }
    None
}

fn is_writ_department_restriction(matcher: &CourseMatcher) -> bool {
    matches!(
        matcher,
        CourseMatcher::Restriction { department, .. }
            if department
                .as_ref()
                .is_some_and(|d| d.iter().any(|dept| dept == "WRIT"))
    )
}

/// Pool constraints (`:c`), flex pool slots (`:p`), and finite SingleCourse lists participate
/// in cross-degree overlap. Mega-opportunities are avoided by pairing exactly one slot per
/// degree (max double-count).
fn matcher_cross_degree_overlap_eligible(matcher: &CourseMatcher, in_anyof: bool) -> bool {
    match matcher {
        CourseMatcher::OneOf(possibilities) => {
            if possibilities.is_empty() || possibilities.len() > MAX_EXPLICIT_ONEOF_OVERLAP {
                return false;
            }
            !in_anyof || possibilities.len() > 1
        }
        CourseMatcher::Restriction {
            department,
            attr,
            no_school,
            ..
        } => {
            if department
                .as_ref()
                .is_some_and(|d| d.iter().any(|dept| dept == "WRIT"))
            {
                return true;
            }
            if department.as_ref().is_some_and(|d| !d.is_empty()) {
                return false;
            }
            attr.as_ref().is_some_and(|a| !a.is_empty()) || no_school.is_some()
        }
        CourseMatcher::Unrestricted => false,
        CourseMatcher::AnyOf(children) => children
            .iter()
            .any(|child| matcher_cross_degree_overlap_eligible(child, true)),
        CourseMatcher::AllOf(children) => children
            .iter()
            .any(|child| matcher_cross_degree_overlap_eligible(child, true)),
    }
}

fn cross_degree_overlap_eligible(slot: &OpenSlot, degree_schools: &[String]) -> bool {
    if college_data::is_cas_college_double_major(degree_schools) {
        return college_data::is_cas_major_overlap_slot_key(&slot.slot_key);
    }
    if slot.slot_key.contains(":c") || slot.slot_key.contains(":p") {
        return true;
    }
    if is_writ_department_restriction(&slot.matcher) {
        return true;
    }
    if slot.slot_key == "0" || slot.slot_key.starts_with("1:f") {
        return false;
    }
    if matches!(&slot.matcher, CourseMatcher::Unrestricted) {
        return slot.slot_key.contains(":p");
    }
    matcher_cross_degree_overlap_eligible(&slot.matcher, false)
}

fn opportunity_is_valid_pair(slots: &[OverlapSlotRef]) -> bool {
    if slots.len() != 2 {
        return false;
    }
    let degrees: HashSet<usize> = slots.iter().map(|s| s.degree_index).collect();
    degrees.len() == 2
}

/// When a course matches many flex slots on one degree, pair using one flex representative
/// per degree so we don't explode pair count (assignment still consumes one slot at a time).
fn trim_slots_for_pairing(
    by_degree: HashMap<usize, Vec<usize>>,
    open_slots: &[OpenSlot],
) -> HashMap<usize, Vec<usize>> {
    let mut trimmed = HashMap::new();
    for (deg, indices) in by_degree {
        let mut flex_pick: Option<usize> = None;
        let mut constrained = Vec::new();
        for idx in &indices {
            if open_slots[*idx].slot_key.contains(":p") {
                if flex_pick.is_none() {
                    flex_pick = Some(*idx);
                }
            } else {
                constrained.push(*idx);
            }
        }
        let mut slots = constrained;
        if let Some(f) = flex_pick {
            slots.push(f);
        }
        if !slots.is_empty() {
            trimmed.insert(deg, slots);
        }
    }
    trimmed
}

/// Expand a course's matching slots into cross-degree pairs (one slot per degree).
fn cross_degree_slot_pairs(
    slot_indices: &[usize],
    open_slots: &[OpenSlot],
    eligible_indices: &HashSet<usize>,
) -> Vec<Vec<usize>> {
    let mut by_degree: HashMap<usize, Vec<usize>> = HashMap::new();
    for &idx in slot_indices {
        if !eligible_indices.contains(&idx) {
            continue;
        }
        by_degree
            .entry(open_slots[idx].degree_index)
            .or_default()
            .push(idx);
    }
    by_degree = trim_slots_for_pairing(by_degree, open_slots);
    if by_degree.len() < 2 {
        return vec![];
    }

    let mut degrees: Vec<usize> = by_degree.keys().copied().collect();
    degrees.sort_unstable();
    let mut pairs = Vec::new();
    let base_degree = degrees[0];
    let base_slots = &by_degree[&base_degree];

    for &other_degree in &degrees[1..] {
        for &a in base_slots {
            for &b in &by_degree[&other_degree] {
                let pair_refs = vec![&open_slots[a], &open_slots[b]];
                if slots_share_consumption_group(&pair_refs) {
                    continue;
                }
                let mut pair = vec![a, b];
                pair.sort_unstable();
                pairs.push(pair);
            }
        }
    }
    pairs
}

fn course_is_explicit_option(course: &str, matcher: &CourseMatcher) -> bool {
    matches!(matcher, CourseMatcher::OneOf(v) if v.iter().any(|c| c == course))
}

/// Lower is better. Prefer courses named directly on every open slot over broad attribute matches.
fn course_overlap_quality_score(course: &str, slot_refs: &[&OpenSlot]) -> usize {
    let all_explicit = slot_refs
        .iter()
        .all(|s| course_is_explicit_option(course, &s.matcher));
    if all_explicit {
        return 0;
    }

    let base = slot_refs
        .iter()
        .map(|s| s.matcher.specificity_score())
        .min()
        .unwrap_or(usize::MAX);

    if slot_refs
        .iter()
        .any(|s| course_is_explicit_option(course, &s.matcher))
    {
        return base / 4;
    }

    base
}

fn format_opportunity_explanation(slots: &[OverlapSlotRef]) -> String {
    let parts: Vec<String> = slots
        .iter()
        .map(|s| format!("{} — {}", s.major, s.label))
        .collect();
    format!("One course can satisfy: {}", parts.join(" + "))
}


pub fn extract_open_slots(
    per_degree: &[DegreeValidationResult],
    majors: &[&Major],
) -> Vec<OpenSlot> {
    let mut slots = Vec::new();

    for (degree_index, validation) in per_degree.iter().enumerate() {
        for mapped in &validation.unfulfilled {
            push_open_from_mapped(&mut slots, degree_index, mapped);
        }
        for mapped in validation.fulfilled.iter().filter(|m| m.partial) {
            push_open_from_mapped(&mut slots, degree_index, mapped);
        }

        for pool in &validation.pool_coverage_info {
            for pi in 0..(pool.flexible_slots_total - pool.flexible_slots_filled).max(0) as usize {
                slots.push(OpenSlot {
                    degree_index,
                    slot_key: format!("{}:p{}", pool.pool_index, pi),
                    label: format!("1 CU from {}", pool.category),
                    matcher: CourseMatcher::Unrestricted,
                    consumption_group: None,
                    gened_attr: None,
                });
            }
        }

        let _ = &majors[degree_index];
    }

    for (degree_index, major) in majors.iter().enumerate() {
        for (pool_idx, pool_req) in major.requirements.iter().enumerate() {
            if let Requirement::CoursePool { constraints, .. } = pool_req {
                let units = crate::requirement::pool_constraint_units(constraints);
                let validation = &per_degree[degree_index];
                let pool_info = validation
                    .pool_coverage_info
                    .iter()
                    .find(|p| p.pool_index == pool_idx);
                for (ci, (req, group)) in units.into_iter().enumerate() {
                    let fulfilled = pool_info
                        .and_then(|p| p.constraints.get(ci))
                        .map(|c| c.fulfilled)
                        .unwrap_or(false);
                    if fulfilled {
                        continue;
                    }
                    let slot_key = format!("{pool_idx}:c{ci}");
                    if slots
                        .iter()
                        .any(|s| s.degree_index == degree_index && s.slot_key == slot_key)
                    {
                        if let Some(slot) = slots
                            .iter_mut()
                            .find(|s| s.degree_index == degree_index && s.slot_key == slot_key)
                        {
                            slot.matcher = compile_matcher(&req, None);
                            slot.consumption_group = Some(group);
                            slot.label = slot_label(&req);
                            slot.gened_attr = attr_from_requirement(&req);
                        }
                    } else {
                        slots.push(OpenSlot {
                            degree_index,
                            slot_key,
                            label: slot_label(&req),
                            matcher: compile_matcher(&req, None),
                            consumption_group: Some(group),
                            gened_attr: attr_from_requirement(&req),
                        });
                    }
                }
            }
        }
    }

    slots
}

fn slots_share_consumption_group(slots: &[&OpenSlot]) -> bool {
    let mut seen: HashSet<(usize, String)> = HashSet::new();
    for slot in slots {
        if let Some(g) = &slot.consumption_group {
            if !seen.insert((slot.degree_index, g.clone())) {
                return true;
            }
        }
    }
    false
}

fn can_claim_all(
    course: &str,
    slots: &[&OpenSlot],
    cross_state: &CrossDegreeState,
    cu_map: &HashMap<String, f64>,
) -> bool {
    slots.iter().all(|slot| {
        cross_state
            .can_claim(course, slot.degree_index, cu_map)
            .is_ok()
    })
}

fn register_hints(
    hints_by_slot: &mut HashMap<String, Vec<String>>,
    slot_explanations: &mut HashMap<String, String>,
    slot: &OpenSlot,
    suggested: &[String],
    explanation: &str,
) {
    let key = hint_key(slot.degree_index, &slot.slot_key);
    hints_by_slot.entry(key.clone()).or_default().extend(suggested.iter().cloned());
    slot_explanations
        .entry(key)
        .or_insert_with(|| explanation.to_string());
    if let Some(attr) = &slot.gened_attr {
        let gened_key = hint_key(slot.degree_index, &format!("gened:{attr}"));
        hints_by_slot
            .entry(gened_key.clone())
            .or_default()
            .extend(suggested.iter().cloned());
        slot_explanations
            .entry(gened_key)
            .or_insert_with(|| explanation.to_string());
    }
}

fn dedupe_hints(hints_by_slot: &mut HashMap<String, Vec<String>>) {
    for courses in hints_by_slot.values_mut() {
        courses.sort();
        courses.dedup();
        if courses.len() > MAX_SUGGESTED_COURSES {
            courses.truncate(MAX_SUGGESTED_COURSES);
        }
    }
}

fn select_overlap_pairs(opportunities: &[OverlapOpportunity]) -> Vec<OverlapPair> {
    let mut used_slots: HashSet<(usize, String)> = HashSet::new();
    let mut pairs = Vec::new();

    for opp in opportunities {
        if !opportunity_is_valid_pair(&opp.slots) {
            continue;
        }
        let slot_keys: Vec<(usize, String)> = opp
            .slots
            .iter()
            .map(|s| (s.degree_index, s.slot_key.clone()))
            .collect();
        if slot_keys.iter().any(|k| used_slots.contains(k)) {
            continue;
        }
        for k in &slot_keys {
            used_slots.insert(k.clone());
        }
        pairs.push(OverlapPair {
            slots: opp.slots.clone(),
            explanation: opp.explanation.clone(),
        });
    }

    pairs
}

fn explicit_oneof_courses(slots: &[OpenSlot], eligible: &HashSet<usize>) -> Vec<String> {
    let mut courses = HashSet::new();
    for idx in eligible {
        if let CourseMatcher::OneOf(list) = &slots[*idx].matcher {
            for c in list {
                if course::is_valid_course_code(c) {
                    courses.insert(c.clone());
                }
            }
        }
    }
    let mut out: Vec<String> = courses.into_iter().collect();
    out.sort();
    out
}

fn index_explicit_courses_to_slots(
    course_to_slots: &mut HashMap<String, Vec<usize>>,
    open_slots: &[OpenSlot],
    eligible_indices: &HashSet<usize>,
    taken: &HashSet<String>,
    attributes: &HashMap<String, Vec<String>>,
) {
    for course in explicit_oneof_courses(open_slots, eligible_indices) {
        if taken.contains(&course) {
            continue;
        }
        for &slot_idx in eligible_indices {
            let slot = &open_slots[slot_idx];
            if course_satisfies_matcher(&slot.matcher, &course, attributes) {
                let entry = course_to_slots.entry(course.clone()).or_default();
                if !entry.contains(&slot_idx) {
                    entry.push(slot_idx);
                }
            }
        }
    }
}

/// Inverted-index overlap discovery: O(sum of candidate set sizes), not O(slots²).
pub fn compute_overlap_plan(
    per_degree: &[DegreeValidationResult],
    majors: &[&Major],
    degree_schools: &[String],
    degree_majors: &[String],
    taken: &HashSet<String>,
    cross_state: &CrossDegreeState,
    cu_map: &HashMap<String, f64>,
) -> OverlapPlan {
    if !cross_degree_optimizer_applicable(degree_schools) {
        return OverlapPlan::empty();
    }

    let index = CatalogIndex::build();
    let open_slots = extract_open_slots(per_degree, majors);
    if open_slots.len() < 2 {
        return OverlapPlan::empty();
    }

    let eligible_indices: HashSet<usize> = open_slots
        .iter()
        .enumerate()
        .filter(|(_, s)| cross_degree_overlap_eligible(s, degree_schools))
        .map(|(i, _)| i)
        .collect();

    if eligible_indices.len() < 2 {
        return OverlapPlan::empty();
    }

    let attributes = crate::penn_data::attributes_data::create_attributes();

    let slot_candidates: Vec<Option<HashSet<String>>> = open_slots
        .iter()
        .enumerate()
        .map(|(slot_idx, slot)| {
            if !eligible_indices.contains(&slot_idx) {
                return None;
            }
            let mut set = candidates_for_matcher(&slot.matcher, &index, taken)?;
            if set.len() > MAX_CANDIDATES_PER_SLOT {
                let mut v: Vec<String> = set.into_iter().collect();
                v.sort();
                v.truncate(MAX_CANDIDATES_PER_SLOT);
                set = v.into_iter().collect();
            }
            Some(set)
        })
        .collect();

    let mut course_to_slots: HashMap<String, Vec<usize>> = HashMap::new();
    for (slot_idx, slot) in open_slots.iter().enumerate() {
        if !eligible_indices.contains(&slot_idx) {
            continue;
        }
        if let Some(candidates) = &slot_candidates[slot_idx] {
            for course in candidates {
                if course_satisfies_matcher(&slot.matcher, course, &attributes) {
                    course_to_slots
                        .entry(course.clone())
                        .or_default()
                        .push(slot_idx);
                }
            }
        }
    }

    index_explicit_courses_to_slots(
        &mut course_to_slots,
        &open_slots,
        &eligible_indices,
        taken,
        &attributes,
    );

    for slot_idx in &eligible_indices {
        if slot_candidates[*slot_idx].is_some() {
            continue;
        }
        let slot = &open_slots[*slot_idx];
        for (peer_idx, peer_set) in slot_candidates.iter().enumerate() {
            if !eligible_indices.contains(&peer_idx) || *slot_idx == peer_idx {
                continue;
            }
            let Some(peer) = peer_set else {
                continue;
            };
            for course in peer {
                if course_satisfies_matcher(&slot.matcher, course, &attributes) {
                    let entry = course_to_slots.entry(course.clone()).or_default();
                    if !entry.contains(slot_idx) {
                        entry.push(*slot_idx);
                    }
                }
            }
        }
    }

    let mut group_courses: HashMap<Vec<usize>, Vec<(String, usize)>> = HashMap::new();

    for (course, slot_indices) in course_to_slots {
        let mut unique: Vec<usize> = slot_indices;
        unique.sort_unstable();
        unique.dedup();
        if unique.len() < 2 {
            continue;
        }

        for pair in cross_degree_slot_pairs(&unique, &open_slots, &eligible_indices) {
            let slot_refs: Vec<&OpenSlot> = pair.iter().map(|&i| &open_slots[i]).collect();
            if !can_claim_all(&course, &slot_refs, cross_state, cu_map) {
                continue;
            }
            let specificity = pair
                .iter()
                .map(|&i| open_slots[i].matcher.specificity_score())
                .min()
                .unwrap_or(usize::MAX);
            let quality = course_overlap_quality_score(&course, &slot_refs);
            let score = specificity.saturating_add(quality);
            group_courses
                .entry(pair)
                .or_default()
                .push((course.clone(), score));
        }
    }

    let mut scored_opportunities: Vec<(usize, OverlapOpportunity)> = Vec::new();
    let mut hints_by_slot: HashMap<String, Vec<String>> = HashMap::new();
    let mut slot_explanations: HashMap<String, String> = HashMap::new();

    for (slot_indices, mut courses) in group_courses {
        courses.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        courses.dedup_by(|a, b| a.0 == b.0);
        let best_score = courses.first().map(|(_, score)| *score).unwrap_or(usize::MAX);
        let suggested: Vec<String> = courses
            .iter()
            .take(MAX_SUGGESTED_COURSES)
            .map(|(c, _)| c.clone())
            .collect();
        if suggested.is_empty() {
            continue;
        }

        let slots: Vec<OverlapSlotRef> = slot_indices
            .iter()
            .map(|&i| {
                let s = &open_slots[i];
                OverlapSlotRef {
                    degree_index: s.degree_index,
                    slot_key: s.slot_key.clone(),
                    label: s.label.clone(),
                    school: degree_schools[s.degree_index].clone(),
                    major: degree_majors[s.degree_index].clone(),
                }
            })
            .collect();

        let explanation = format_opportunity_explanation(&slots);

        for &i in &slot_indices {
            register_hints(
                &mut hints_by_slot,
                &mut slot_explanations,
                &open_slots[i],
                &suggested,
                &explanation,
            );
        }

        scored_opportunities.push((
            best_score,
            OverlapOpportunity {
                slots,
                suggested_courses: suggested,
                explanation,
            },
        ));
    }

    scored_opportunities.retain(|(_, o)| opportunity_is_valid_pair(&o.slots));

    dedupe_hints(&mut hints_by_slot);

    scored_opportunities.sort_by(|(score_a, opp_a), (score_b, opp_b)| {
        score_a
            .cmp(score_b)
            .then_with(|| opp_a.suggested_courses.len().cmp(&opp_b.suggested_courses.len()))
            .then_with(|| opp_a.explanation.cmp(&opp_b.explanation))
    });
    let opportunities: Vec<OverlapOpportunity> = scored_opportunities
        .into_iter()
        .map(|(_, opp)| opp)
        .collect();

    let pairs = select_overlap_pairs(&opportunities);

    OverlapPlan {
        opportunities,
        hints_by_slot,
        pairs,
        slot_explanations,
    }
}

impl OverlapPlan {
    fn empty() -> Self {
        Self {
            opportunities: vec![],
            hints_by_slot: HashMap::new(),
            pairs: vec![],
            slot_explanations: HashMap::new(),
        }
    }
}

