use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::catalog_index::CatalogIndex;
use crate::course;
use crate::course_matcher::{candidates_for_matcher, compile_matcher, course_satisfies_matcher, CourseMatcher};
use crate::cross_degree::CrossDegreeState;
use crate::major::Major;
use crate::requirement::{
    DegreeValidationResult, MappedRequirement, Requirement,
};

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
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlapPlan {
    pub opportunities: Vec<OverlapOpportunity>,
    /// `"{degree_index}:{slot_key}"` → overlap course suggestions for UI hover.
    pub hints_by_slot: HashMap<String, Vec<String>>,
    /// Courses selected by the greedy overlap assigner (injected into validation).
    pub planned_courses: Vec<String>,
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
                    if slots.iter().any(|s| s.degree_index == degree_index && s.slot_key == slot_key)
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

fn distinct_degrees(slots: &[&OpenSlot]) -> usize {
    slots
        .iter()
        .map(|s| s.degree_index)
        .collect::<HashSet<_>>()
        .len()
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
    slot: &OpenSlot,
    suggested: &[String],
) {
    hints_by_slot
        .entry(hint_key(slot.degree_index, &slot.slot_key))
        .or_default()
        .extend(suggested.iter().cloned());
    if let Some(attr) = &slot.gened_attr {
        hints_by_slot
            .entry(hint_key(slot.degree_index, &format!("gened:{attr}")))
            .or_default()
            .extend(suggested.iter().cloned());
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

    let attributes = crate::attributes_data::create_attributes();

    let slot_candidates: Vec<Option<HashSet<String>>> = open_slots
        .iter()
        .map(|slot| {
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

    for (slot_idx, slot) in open_slots.iter().enumerate() {
        if slot_candidates[slot_idx].is_some() {
            continue;
        }
        for (peer_idx, peer_set) in slot_candidates.iter().enumerate() {
            if slot_idx == peer_idx {
                continue;
            }
            let Some(peer) = peer_set else {
                continue;
            };
            for course in peer {
                if course_satisfies_matcher(&slot.matcher, course, &attributes) {
                    let entry = course_to_slots.entry(course.clone()).or_default();
                    if !entry.contains(&slot_idx) {
                        entry.push(slot_idx);
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
        let slot_refs: Vec<&OpenSlot> = unique.iter().map(|&i| &open_slots[i]).collect();
        if distinct_degrees(&slot_refs) < 2 {
            continue;
        }
        if slots_share_consumption_group(&slot_refs) {
            continue;
        }
        if !can_claim_all(&course, &slot_refs, cross_state, cu_map) {
            continue;
        }
        let specificity = unique
            .iter()
            .map(|&i| open_slots[i].matcher.specificity_score())
            .min()
            .unwrap_or(usize::MAX);
        group_courses
            .entry(unique)
            .or_default()
            .push((course, specificity));
    }

    let mut opportunities = Vec::new();
    let mut hints_by_slot: HashMap<String, Vec<String>> = HashMap::new();

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

        for &i in &slot_indices {
            register_hints(&mut hints_by_slot, &open_slots[i], &suggested);
        }

        opportunities.push(OverlapOpportunity {
            slots,
            suggested_courses: suggested,
        });
    }

    dedupe_hints(&mut hints_by_slot);

    opportunities.sort_by(|a, b| {
        b.slots
            .len()
            .cmp(&a.slots.len())
            .then_with(|| a.suggested_courses.len().cmp(&b.suggested_courses.len()))
    });

    let planned_courses = greedy_select_overlaps(
        &opportunities,
        &open_slots,
        cross_state,
        cu_map,
        &attributes,
    );

    OverlapPlan {
        opportunities,
        hints_by_slot,
        planned_courses,
    }
}

impl OverlapPlan {
    fn empty() -> Self {
        Self {
            opportunities: vec![],
            hints_by_slot: HashMap::new(),
            planned_courses: vec![],
        }
    }
}

fn greedy_select_overlaps(
    opportunities: &[OverlapOpportunity],
    open_slots: &[OpenSlot],
    cross_state: &CrossDegreeState,
    cu_map: &HashMap<String, f64>,
    attributes: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let mut planned = Vec::new();
    let mut used_slots: HashSet<(usize, String)> = HashSet::new();
    let mut used_groups: HashSet<(usize, String)> = HashSet::new();

    for opp in opportunities {
        let Some(course) = opp.suggested_courses.first() else {
            continue;
        };

        let mut slot_refs: Vec<&OpenSlot> = Vec::new();
        for s in &opp.slots {
            let key = (s.degree_index, s.slot_key.clone());
            if used_slots.contains(&key) {
                continue;
            }
            if let Some(slot) = open_slots
                .iter()
                .find(|o| o.degree_index == s.degree_index && o.slot_key == s.slot_key)
            {
                slot_refs.push(slot);
            }
        }
        if slot_refs.len() < 2 || distinct_degrees(&slot_refs) < 2 {
            continue;
        }
        if slots_share_consumption_group(&slot_refs) {
            continue;
        }
        let blocked_by_group = slot_refs.iter().any(|slot| {
            slot.consumption_group
                .as_ref()
                .is_some_and(|g| used_groups.contains(&(slot.degree_index, g.clone())))
        });
        if blocked_by_group {
            continue;
        }
        if !can_claim_all(course, &slot_refs, cross_state, cu_map) {
            continue;
        }
        if !slot_refs
            .iter()
            .all(|s| course_satisfies_matcher(&s.matcher, course, attributes))
        {
            continue;
        }

        planned.push(course.clone());
        for slot in slot_refs {
            used_slots.insert((slot.degree_index, slot.slot_key.clone()));
            if let Some(g) = &slot.consumption_group {
                used_groups.insert((slot.degree_index, g.clone()));
            }
        }
    }

    planned
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
    }
}
