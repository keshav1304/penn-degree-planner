use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::catalog_index::CatalogIndex;
use crate::course;
use crate::course_matcher::{
    candidates_for_matcher, compile_matcher, course_satisfies_matcher, CourseMatcher,
};
use crate::cross_degree::CrossDegreeState;
use crate::major::Major;
use crate::requirement::{DegreeValidationResult, MappedRequirement, Requirement};

const MAX_SUGGESTED_COURSES: usize = 12;
const MAX_CANDIDATES_PER_SLOT: usize = 800;

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
pub struct OverlapAssignment {
    pub course: String,
    pub slots: Vec<OverlapSlotRef>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlapPlan {
    pub opportunities: Vec<OverlapOpportunity>,
    /// `"{degree_index}:{slot_key}"` → overlap course suggestions for UI hover.
    pub hints_by_slot: HashMap<String, Vec<String>>,
    /// Courses chosen to satisfy multiple degrees (applied before scheduling).
    pub assignments: Vec<OverlapAssignment>,
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

pub fn hint_key(degree_index: usize, slot_key: &str) -> String {
    format!("{degree_index}:{slot_key}")
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
    None
}

/// Only broad, shareable pool constraints participate in cross-degree overlap.
/// Flex pool slots (`:p`) are excluded — they match too many courses and create
/// unusable mega-opportunities spanning dozens of slots.
fn cross_degree_overlap_eligible(slot: &OpenSlot) -> bool {
    if slot.slot_key.contains(":c") {
        return true;
    }
    if slot.slot_key.contains(":p") || slot.slot_key == "0" || slot.slot_key.starts_with("1:f") {
        return false;
    }
    match &slot.matcher {
        CourseMatcher::OneOf(_) => false,
        CourseMatcher::Restriction {
            department,
            attr,
            no_school,
            ..
        } => {
            if department.as_ref().is_some_and(|d| !d.is_empty()) {
                return false;
            }
            attr.as_ref().is_some_and(|a| !a.is_empty()) || no_school.is_some()
        }
        CourseMatcher::Unrestricted => false,
        CourseMatcher::AnyOf(_) | CourseMatcher::AllOf(_) => false,
    }
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

fn course_overlap_quality_score(course: &str, slot_refs: &[&OpenSlot]) -> usize {
    let base = slot_refs
        .iter()
        .map(|s| s.matcher.specificity_score())
        .min()
        .unwrap_or(usize::MAX);
    let mut penalty = 0usize;

    for prefix in ["NRSC", "BIOL", "CIS", "ESE", "BE", "MEAM", "NETS", "CBE"] {
        if course.starts_with(&format!("{prefix} ")) {
            penalty += 800;
            break;
        }
    }
    for prefix in ["HIST", "ANTH", "PSYC", "SOCI", "PHIL", "ENGL", "ARTH", "LING", "REL", "PPE"] {
        if course.starts_with(&format!("{prefix} ")) {
            penalty = penalty.saturating_sub(200);
            break;
        }
    }
    base + penalty
}

fn format_opportunity_explanation(slots: &[OverlapSlotRef]) -> String {
    let parts: Vec<String> = slots
        .iter()
        .map(|s| format!("{} — {}", s.major, s.label))
        .collect();
    format!("One course can satisfy: {}", parts.join(" + "))
}

fn format_assignment_explanation(course: &str, slots: &[OverlapSlotRef]) -> String {
    let parts: Vec<String> = slots
        .iter()
        .map(|s| format!("{} ({})", s.label, s.major))
        .collect();
    format!("{course} satisfies {}", parts.join(" and "))
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

fn can_claim_all_refs(
    course: &str,
    slots: &[OverlapSlotRef],
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

fn select_overlap_assignments(
    opportunities: &[OverlapOpportunity],
    open_slots: &[OpenSlot],
    cross_state: &CrossDegreeState,
    cu_map: &HashMap<String, f64>,
) -> Vec<OverlapAssignment> {
    let mut used_slots: HashSet<(usize, String)> = HashSet::new();
    let mut used_courses: HashSet<String> = HashSet::new();
    let mut assignments = Vec::new();

    let slot_lookup: HashMap<(usize, String), usize> = open_slots
        .iter()
        .enumerate()
        .map(|(i, s)| ((s.degree_index, s.slot_key.clone()), i))
        .collect();

    for opp in opportunities {
        if opp.slots.len() < 2 {
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

        let slot_refs: Vec<&OpenSlot> = slot_keys
            .iter()
            .filter_map(|k| slot_lookup.get(k).map(|&i| &open_slots[i]))
            .collect();
        if slot_refs.len() != slot_keys.len() {
            continue;
        }

        let mut best_course: Option<String> = None;
        let mut best_score = usize::MAX;
        for course in &opp.suggested_courses {
            if used_courses.contains(course) {
                continue;
            }
            if !can_claim_all_refs(course, &opp.slots, cross_state, cu_map) {
                continue;
            }
            let score = course_overlap_quality_score(course, &slot_refs);
            if score < best_score {
                best_score = score;
                best_course = Some(course.clone());
            }
        }

        if let Some(course) = best_course {
            for k in &slot_keys {
                used_slots.insert(k.clone());
            }
            used_courses.insert(course.clone());
            assignments.push(OverlapAssignment {
                explanation: format_assignment_explanation(&course, &opp.slots),
                course,
                slots: opp.slots.clone(),
            });
        }
    }

    assignments
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
    index: &CatalogIndex,
) -> OverlapPlan {
    if per_degree.len() < 2 {
        return OverlapPlan::empty();
    }

    let open_slots = extract_open_slots(per_degree, majors);
    if open_slots.len() < 2 {
        return OverlapPlan::empty();
    }

    let eligible_indices: HashSet<usize> = open_slots
        .iter()
        .enumerate()
        .filter(|(_, s)| cross_degree_overlap_eligible(s))
        .map(|(i, _)| i)
        .collect();

    if eligible_indices.len() < 2 {
        return OverlapPlan::empty();
    }

    let attributes = crate::attributes_data::create_attributes();

    let slot_candidates: Vec<Option<HashSet<String>>> = open_slots
        .iter()
        .enumerate()
        .map(|(slot_idx, slot)| {
            if !eligible_indices.contains(&slot_idx) {
                return None;
            }
            let mut set = candidates_for_matcher(&slot.matcher, index, taken)?;
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
            group_courses
                .entry(pair)
                .or_default()
                .push((course.clone(), specificity + quality));
        }
    }

    let mut opportunities = Vec::new();
    let mut hints_by_slot: HashMap<String, Vec<String>> = HashMap::new();
    let mut slot_explanations: HashMap<String, String> = HashMap::new();

    for (slot_indices, mut courses) in group_courses {
        courses.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        courses.dedup_by(|a, b| a.0 == b.0);
        let suggested: Vec<String> = courses
            .into_iter()
            .take(MAX_SUGGESTED_COURSES)
            .map(|(c, _)| c)
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

        opportunities.push(OverlapOpportunity {
            slots,
            suggested_courses: suggested,
            explanation,
        });
    }

    dedupe_hints(&mut hints_by_slot);

    opportunities.sort_by(|a, b| {
        b.slots
            .len()
            .cmp(&a.slots.len())
            .then_with(|| a.suggested_courses.len().cmp(&b.suggested_courses.len()))
    });

    let assignments = select_overlap_assignments(&opportunities, &open_slots, cross_state, cu_map);

    OverlapPlan {
        opportunities,
        hints_by_slot,
        assignments,
        slot_explanations,
    }
}

impl OverlapPlan {
    fn empty() -> Self {
        Self {
            opportunities: vec![],
            hints_by_slot: HashMap::new(),
            assignments: vec![],
            slot_explanations: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::major::resolve_major;

    #[test]
    fn neur_wh_nofl_has_cross_degree_overlap_hints() {
        let neur = resolve_major("CAS", "NEUR", &[]).expect("NEUR");
        let wh = resolve_major("WH", "WH_NOFL", &["FNCE".to_string()]).expect("WH_NOFL");
        let cu_map: HashMap<String, f64> = crate::courses_data::all_courses()
            .iter()
            .map(|c| (c.course_code.clone(), c.cu))
            .collect();
        let taken: HashSet<String> = HashSet::new();

        let neur_v = crate::requirement::validate_courses_for_degree(
            neur.requirements.clone(),
            &vec![],
            &cu_map,
        );
        let wh_v = crate::requirement::validate_courses_for_degree(
            wh.requirements.clone(),
            &vec![],
            &cu_map,
        );
        let per_degree = vec![neur_v, wh_v];
        let schools = vec!["CAS".to_string(), "WH".to_string()];
        let majors_code = vec!["NEUR".to_string(), "WH_NOFL".to_string()];
        let cross_state = CrossDegreeState::new(schools.clone(), majors_code.clone());
        let index = CatalogIndex::build();

        let plan = compute_overlap_plan(
            &per_degree,
            &[&neur, &wh],
            &schools,
            &majors_code,
            &taken,
            &cross_state,
            &cu_map,
            &index,
        );

        assert!(
            !plan.opportunities.is_empty(),
            "expected cross-degree overlap opportunities for NEUR + WH_NOFL"
        );
        assert!(
            !plan.hints_by_slot.is_empty(),
            "expected per-slot overlap hints"
        );
        assert!(
            !plan.assignments.is_empty(),
            "expected overlap assignments for NEUR + WH_NOFL"
        );
        assert!(
            plan.assignments.len() >= 3,
            "expected multiple disjoint overlap assignments, got {} from {} opportunities",
            plan.assignments.len(),
            plan.opportunities.len()
        );
        for a in &plan.assignments {
            assert!(
                !a.course.starts_with("NRSC "),
                "overlap should prefer gen-ed courses, not NRSC electives: {}",
                a.course
            );
        }
    }
}
