use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::course;
use crate::cross_degree::{self, is_graduate_degree, CrossDegreeSummary};
use crate::major::{self, Major, resolve_major};
use crate::overlap_planner::{
    self, OverlapOpportunity, OverlapPair, OverlapPlan, OverlapScheduleGroup, OverlapSlotRef,
};
use crate::penn_data::{self, college_data, courses_data};
use crate::penn_data::college_data::CasGenEdInfo;
use crate::requirement::{
    self, ConcentrationInfo, MappedRequirement, PoolCoverageInfo,
};
use crate::schedule_template::{
    merge_schedule_hint, ms_default_semester_target, ms_default_semester_target_for_requirement,
    ms_grad_placement_candidates, placement_semesters, resolve_semester_hint,
    ScheduleHint, ScheduleHintMode, semester_order,
};

pub const DEFAULT_SEMESTER_CU_LIMIT: f64 = 5.5;
pub const DUAL_UG_SEMESTER_CU_LIMIT: f64 = 6.5;
pub const DEFAULT_SUMMER_CU_LIMIT: f64 = 2.0;
pub const CU_EPS: f64 = 0.001;

fn overlap_slots_equal(a: &[OverlapSlotRef], b: &[OverlapSlotRef]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut keys: Vec<_> = a
        .iter()
        .map(|s| (s.degree_index, s.slot_key.as_str()))
        .collect();
    let mut other: Vec<_> = b
        .iter()
        .map(|s| (s.degree_index, s.slot_key.as_str()))
        .collect();
    keys.sort_unstable();
    other.sort_unstable();
    keys == other
}

/// Shared named course on both sides of a pair → schedule as a course card, not a dashed overlap block.
/// At least one degree must name the course explicitly (typically SingleCourse); the other may
/// accept it via an explicit option list or a matching restriction pool.
fn overlap_pair_fixed_course(
    pair: &OverlapPair,
    opportunities: &[OverlapOpportunity],
    per_degree: &[requirement::DegreeValidationResult],
) -> Option<String> {
    let opp = opportunities
        .iter()
        .find(|o| overlap_slots_equal(&o.slots, &pair.slots))?;
    for course in &opp.suggested_courses {
        if !course::is_valid_course_code(course) {
            continue;
        }
        let mut names_course_explicitly = false;
        let mut all_accept = true;
        for slot_ref in &pair.slots {
            let validation = per_degree.get(slot_ref.degree_index)?;
            let mapped = validation.mapped_for_instance(&slot_ref.slot_key)?;
            if requirement::requirement_explicitly_lists_course(&mapped.requirement, course) {
                names_course_explicitly = true;
            }
            if !requirement::requirement_accepts_shared_course(&mapped.requirement, course) {
                all_accept = false;
                break;
            }
        }
        if names_course_explicitly && all_accept {
            return Some(course.clone());
        }
    }
    None
}

pub fn dual_undergrad_only(schools: &[String]) -> bool {
    cross_degree::cross_degree_optimizer_applicable(schools)
}

pub fn all_cas_college(schools: &[String]) -> bool {
    !schools.is_empty() && schools.iter().all(|s| s == "CAS")
}

/// Default max CU for a semester before user overrides.
pub fn default_semester_cu_limit(schools: &[String], year: i32, semester: &str) -> f64 {
    if semester == "Summer" {
        return DEFAULT_SUMMER_CU_LIMIT;
    }
    if year == 1 && semester == "Fall" {
        return DEFAULT_SEMESTER_CU_LIMIT;
    }
    if dual_undergrad_only(schools) && !all_cas_college(schools) {
        return DUAL_UG_SEMESTER_CU_LIMIT;
    }
    DEFAULT_SEMESTER_CU_LIMIT
}

pub fn undergrad_schedule_years(schools: &[String]) -> i32 {
    if schools.len() < 2 {
        return 4;
    }
    if all_cas_college(schools) {
        return 4;
    }
    if dual_undergrad_only(schools) {
        return 5;
    }
    4
}


#[derive(Debug, Clone, Deserialize)]
pub struct DegreeInput {
    pub major: String,
    pub school: String,
    #[serde(default)]
    pub concentrations: Vec<String>,
    pub concentration: Option<String>,
}

impl DegreeInput {
    fn effective_concentrations(&self) -> Vec<String> {
        if !self.concentrations.is_empty() {
            return major::normalize_degree_concentrations(&self.school, &self.concentrations);
        }
        self.concentration.clone().into_iter().collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrozenCourse {
    pub course_id: String,
    pub year: i32,
    pub semester: String,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleInput {
    pub taken: Vec<String>,
    pub degrees: Vec<DegreeInput>,
    pub frozen: Vec<FrozenCourse>,
    pub allow_summer: Option<bool>,
    pub semester_cu_limits: Option<HashMap<String, f64>>,
}

#[derive(Serialize)]
pub struct SemesterPlan {
    pub year: i32,
    pub semester: String,
    pub courses: Vec<String>,
    /// Open requirement placeholders (stable `req:` ids — not course codes).
    pub requirement_slots: Vec<String>,
    pub total_cu: f64,
}

#[derive(Serialize)]
pub struct DegreeResult {
    pub school: String,
    pub major: String,
    pub fulfilled_requirements: Vec<MappedRequirement>,
    pub unfulfilled_requirements: Vec<MappedRequirement>,
    pub suggested_for_unfulfilled: Vec<MappedRequirement>,
    pub unapplicable_courses: Vec<String>,
    pub pool_coverage_info: Vec<PoolCoverageInfo>,
    pub concentration_info: Vec<ConcentrationInfo>,
    pub available_concentrations: Vec<String>,
    pub has_core_concentration: bool,
    pub category_order: Vec<String>,
    pub cas_gen_ed: Option<CasGenEdInfo>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ScheduleOutput {
    pub schedule: Vec<SemesterPlan>,
    pub degree_results: Vec<DegreeResult>,
    /// Maps requirement slot id → human-readable description for the schedule UI.
    pub slot_labels: HashMap<String, String>,
    pub cross_degree_summary: Option<CrossDegreeSummary>,
    pub overlap_plan: Option<OverlapPlan>,
    /// Paired requirement blocks (one CU) shown together on the schedule grid.
    pub overlap_schedule_groups: Vec<OverlapScheduleGroup>,
    pub error: Option<String>,
}
pub(crate) fn schedule_target_for_dual_degrees(
    year: i32,
    semester: &str,
    schools: &[String],
) -> (i32, String) {
    if !dual_undergrad_only(schools) || all_cas_college(schools) {
        return (year, semester.to_string());
    }
    let adjusted_year = match year {
        1 | 2 => year,
        3 => 2,
        4 => 3,
        _ => 4,
    };
    (adjusted_year, semester.to_string())
}

fn adjust_schedule_hint_for_dual_degrees(
    hint: ScheduleHint,
    schools: &[String],
) -> ScheduleHint {
    if hint.mode == ScheduleHintMode::Fixed {
        return hint;
    }
    let (year, semester) = schedule_target_for_dual_degrees(hint.year, &hint.semester, schools);
    ScheduleHint {
        year,
        semester,
        mode: hint.mode,
    }
}

fn store_item_hint(
    hints: &mut HashMap<String, ScheduleHint>,
    item_id: String,
    hint: ScheduleHint,
) {
    let merged = merge_schedule_hint(hints.get(&item_id), hint);
    hints.insert(item_id, merged);
}

fn flexible_ms_hint(year: i32, semester: String) -> ScheduleHint {
    ScheduleHint {
        year,
        semester,
        mode: ScheduleHintMode::Flexible,
    }
}

fn cas_college_shared_mapped(mapped: &requirement::MappedRequirement) -> bool {
    mapped
        .instance_id
        .as_deref()
        .is_some_and(college_data::is_cas_college_shared_instance_scope)
}

fn skip_cas_shared_schedule_item(
    is_secondary_cas_major: bool,
    is_primary_cas_major: bool,
    mapped: &requirement::MappedRequirement,
    course_id: &str,
    shared_flex_cap: Option<i32>,
) -> bool {
    if is_secondary_cas_major {
        return cas_college_shared_mapped(mapped)
            || college_data::is_cas_college_shared_schedule_slot(course_id);
    }
    if is_primary_cas_major {
        if let Some(cap) = shared_flex_cap {
            if college_data::is_cas_excess_shared_flexible_schedule_slot(course_id, cap) {
                return true;
            }
        }
    }
    false
}

fn overlap_member_display_label(
    slot_ref: &OverlapSlotRef,
    school: &str,
    major_data: &Major,
) -> String {
    if school != "CAS" {
        return slot_ref.label.clone();
    }
    let Some((pool_idx, _)) = college_data::cas_gened_pool(major_data) else {
        return slot_ref.label.clone();
    };
    if college_data::is_cas_gened_pool_constraint_key(&slot_ref.slot_key, pool_idx) {
        return college_data::cas_gened_overlap_display_label(major_data)
            .unwrap_or_else(|| slot_ref.label.clone());
    }
    slot_ref.label.clone()
}

fn is_cas_gened_constraint_overlap_member(
    member: &overlap_planner::OverlapScheduleGroupMember,
    school: &str,
    major_data: &Major,
) -> bool {
    if school != "CAS" {
        return false;
    }
    let Some((pool_idx, _)) = college_data::cas_gened_pool(major_data) else {
        return false;
    };
    if !requirement::is_pool_constraint_slot_id(&member.schedule_slot_id) {
        return false;
    }
    let Some(rest) = member.schedule_slot_id.strip_prefix("req:") else {
        return false;
    };
    let scope = rest.split(":R:").next().unwrap_or(rest);
    college_data::is_cas_gened_pool_constraint_key(scope, pool_idx)
}

fn suppress_redundant_cas_gened_flex_for_wh_overlaps(
    all_requirement_slots: &mut Vec<String>,
    ug_schedule_items: &mut HashSet<String>,
    degree_results: &mut [DegreeResult],
    overlap_groups: &[OverlapScheduleGroup],
    degree_schools: &[String],
    major_data: &[&Major],
) {
    if !degree_schools.iter().any(|s| s == "WH") {
        return;
    }
    let suppress_count = overlap_groups
        .iter()
        .filter(|g| {
            g.members.iter().any(|m| {
                degree_schools
                    .get(m.degree_index)
                    .is_some_and(|school| school == "CAS")
                    && major_data
                        .get(m.degree_index)
                        .is_some_and(|major| is_cas_gened_constraint_overlap_member(m, "CAS", major))
            })
        })
        .count();
    if suppress_count == 0 {
        return;
    }
    for (degree_idx, school) in degree_schools.iter().enumerate() {
        if school != "CAS" {
            continue;
        }
        let Some(major) = major_data.get(degree_idx) else {
            continue;
        };
        let Some((pool_idx, _)) = college_data::cas_gened_pool(major) else {
            continue;
        };
        let mut removed = 0usize;
        all_requirement_slots.retain(|slot_id| {
            if removed >= suppress_count {
                return true;
            }
            if college_data::is_cas_gened_flex_schedule_slot(slot_id, pool_idx) {
                removed += 1;
                ug_schedule_items.remove(slot_id);
                false
            } else {
                true
            }
        });
        if let Some(result) = degree_results.get_mut(degree_idx) {
            let mut api_removed = 0usize;
            result.suggested_for_unfulfilled.retain(|m| {
                if api_removed >= suppress_count {
                    return true;
                }
                if m.instance_id.as_deref().is_some_and(|k| {
                    college_data::is_cas_gened_pool_flex_key(k, pool_idx)
                }) {
                    api_removed += 1;
                    false
                } else {
                    true
                }
            });
        }

        let max_flex_after_overlap =
            college_data::cas_gened_requirement_row_count().saturating_sub(suppress_count);
        let flex_on_schedule: Vec<String> = all_requirement_slots
            .iter()
            .filter(|s| college_data::is_cas_gened_flex_schedule_slot(s, pool_idx))
            .cloned()
            .collect();
        for slot_id in flex_on_schedule.into_iter().skip(max_flex_after_overlap) {
            all_requirement_slots.retain(|s| s != &slot_id);
            ug_schedule_items.remove(&slot_id);
        }
        if let Some(result) = degree_results.get_mut(degree_idx) {
            let flex_mapped: Vec<String> = result
                .suggested_for_unfulfilled
                .iter()
                .filter_map(|m| m.instance_id.clone())
                .filter(|k| college_data::is_cas_gened_pool_flex_key(k, pool_idx))
                .collect();
            let drop: HashSet<String> = flex_mapped
                .into_iter()
                .skip(max_flex_after_overlap)
                .collect();
            if !drop.is_empty() {
                result
                    .suggested_for_unfulfilled
                    .retain(|m| !m.instance_id.as_deref().is_some_and(|k| drop.contains(k)));
            }
        }
        break;
    }
}

fn is_cas_double_major_excluded_mapped(
    mapped: &requirement::MappedRequirement,
    is_secondary_cas_major: bool,
    is_primary_cas_major: bool,
    shared_flex_cap: Option<i32>,
) -> bool {
    if is_secondary_cas_major && cas_college_shared_mapped(mapped) {
        return true;
    }
    if is_primary_cas_major {
        if let Some(cap) = shared_flex_cap {
            if mapped
                .instance_id
                .as_deref()
                .is_some_and(|scope| college_data::is_cas_excess_shared_flexible_slot(scope, cap))
            {
                return true;
            }
            if mapped.course_ids.iter().all(|id| {
                college_data::is_cas_excess_shared_flexible_schedule_slot(id, cap)
            }) && !mapped.course_ids.is_empty()
            {
                return true;
            }
        }
    }
    false
}

pub fn generate_schedule(payload: ScheduleInput) -> ScheduleOutput {

    let mut taken: Vec<String> = payload
        .taken
        .iter()
        .filter(|c| course::is_valid_course_code(c))
        .cloned()
        .collect();
    let frozen: Vec<FrozenCourse> = payload
        .frozen
        .iter()
        .filter(|f| {
            course::is_valid_course_code(&f.course_id)
                || requirement::is_requirement_slot_id(&f.course_id)
        })
        .cloned()
        .collect();
    // Taken + frozen course codes count toward requirement fulfillment (frozen ≠ completed).
    let mut courses_for_validation: Vec<String> = taken.clone();
    for f in &frozen {
        if course::is_valid_course_code(&f.course_id) && !courses_for_validation.contains(&f.course_id) {
            courses_for_validation.push(f.course_id.clone());
        }
    }

    let mut degree_results: Vec<DegreeResult> = Vec::new();
    let mut all_suggested_courses: Vec<String> = Vec::new();
    let mut all_requirement_slots: Vec<String> = Vec::new();
    let mut slot_labels: HashMap<String, String> = HashMap::new();
    let mut item_hints: HashMap<String, ScheduleHint> = HashMap::new();
    let mut ug_schedule_items: HashSet<String> = HashSet::new();
    let mut ms_schedule_items: HashSet<String> = HashSet::new();
    let mut ms_grad_schedule_items: HashSet<String> = HashSet::new();

    // Build a CU lookup map from all courses
    let all_courses = courses_data::all_courses();
    let cu_map: HashMap<String, f64> = all_courses.iter()
        .map(|c| (c.course_code.clone(), c.cu))
        .collect();

    struct ResolvedDegree {
        input: DegreeInput,
        major_data: Major,
        concs: Vec<String>,
    }

    let mut resolved_degrees: Vec<ResolvedDegree> = Vec::new();
    let mut per_degree_validation: Vec<requirement::DegreeValidationResult> = Vec::new();
    let mut degree_schools: Vec<String> = Vec::new();
    let mut degree_majors: Vec<String> = Vec::new();

    for degree in &payload.degrees {
        let concs = degree.effective_concentrations();
        if let Some(major_data) = resolve_major(&degree.school, &degree.major, &concs) {
            let mut validation = requirement::validate_courses_for_degree(
                major_data.requirements.clone(),
                &courses_for_validation,
                &cu_map,
            );
            for mapped in &mut validation.fulfilled {
                mapped.course_ids = requirement::filter_valid_course_ids(mapped.course_ids.clone());
            }
            validation.fulfilled.retain(|m| !m.course_ids.is_empty());
            validation
                .fulfilled
                .sort_by_key(|r| r.requirement.get_category());

            per_degree_validation.push(validation);
            degree_schools.push(degree.school.clone());
            degree_majors.push(degree.major.clone());
            resolved_degrees.push(ResolvedDegree {
                input: degree.clone(),
                major_data,
                concs,
            });
        } else {
            degree_results.push(DegreeResult {
                school: degree.school.clone(),
                major: degree.major.clone(),
                fulfilled_requirements: vec![],
                unfulfilled_requirements: vec![],
                suggested_for_unfulfilled: vec![],
                unapplicable_courses: vec![],
                pool_coverage_info: vec![],
                concentration_info: vec![],
                available_concentrations: vec![],
                has_core_concentration: false,
                category_order: vec![],
                cas_gen_ed: None,
                error: Some(format!(
                    "Major '{}' in school '{}' is not implemented yet.",
                    degree.major, degree.school
                )),
            });
        }
    }

    let conc_contexts: Vec<requirement::DegreeConcentrationContext> = resolved_degrees
        .iter()
        .map(|resolved| {
            requirement::degree_concentration_context_from_major(
                &resolved.major_data.requirements,
                &resolved.major_data.concentrations,
                &resolved.concs,
            )
        })
        .collect();

    if !per_degree_validation.is_empty() {
        requirement::resolve_cross_degree_conflicts(
            &mut per_degree_validation,
            &degree_schools,
            &degree_majors,
            &cu_map,
            Some(&conc_contexts),
            Some(&courses_for_validation),
        );
    }

    let ug_conc_claims = requirement::build_ug_concentration_claims(
        &conc_contexts,
        &degree_schools,
        &per_degree_validation,
        &courses_for_validation,
        &cu_map,
    );

    let mut cross_state = cross_degree::CrossDegreeState::new(
        degree_schools.clone(),
        degree_majors.clone(),
    );
    if !per_degree_validation.is_empty() {
        let mut fulfilled_allocations =
            requirement::build_allocations_from_fulfilled(&per_degree_validation);
        requirement::merge_concentration_claims_into(&mut fulfilled_allocations, &ug_conc_claims);
        cross_state.rebuild_from_allocations(&fulfilled_allocations, &cu_map);
        cross_state.ug_concentration_courses = ug_conc_claims;
    }

    let major_refs: Vec<&Major> = resolved_degrees
        .iter()
        .map(|r| &r.major_data)
        .collect();
    for (degree_idx, resolved) in resolved_degrees.iter().enumerate() {
        per_degree_validation[degree_idx].refresh_pool_coverage_info(
            &resolved.major_data.requirements,
            &cu_map,
        );
    }
    let mut overlap_plan = if cross_degree::cross_degree_optimizer_applicable(&degree_schools) {
        Some(overlap_planner::compute_overlap_plan(
            &per_degree_validation,
            &major_refs,
            &degree_schools,
            &degree_majors,
            &courses_for_validation.iter().cloned().collect(),
            &cross_state,
            &cu_map,
        ))
    } else {
        None
    };

    let overlap_pair_slots: HashSet<(usize, String)> = overlap_plan
        .as_ref()
        .map(|plan| {
            plan.pairs
                .iter()
                .flat_map(|pair| {
                    pair.slots
                        .iter()
                        .map(|s| (s.degree_index, s.slot_key.clone()))
                })
                .collect()
        })
        .unwrap_or_default();

    let fixed_course_overlap_slots: HashSet<(usize, String)> = overlap_plan
        .as_ref()
        .map(|plan| {
            plan.pairs
                .iter()
                .filter(|pair| {
                    overlap_pair_fixed_course(pair, &plan.opportunities, &per_degree_validation)
                        .is_some()
                })
                .flat_map(|pair| {
                    pair.slots
                        .iter()
                        .map(|s| (s.degree_index, s.slot_key.clone()))
                })
                .collect()
        })
        .unwrap_or_default();

    let cross_degree_optimizer =
        cross_degree::cross_degree_optimizer_applicable(&degree_schools);
    let cas_college_double_major = college_data::is_cas_college_double_major(&degree_schools);
    let primary_cas_degree_index = cas_college_double_major
        .then(|| degree_schools.iter().position(|s| s == "CAS"))
        .flatten();
    let cas_shared_flex_cap = cas_college_double_major.then(|| {
        let cas_majors: Vec<&Major> = resolved_degrees
            .iter()
            .filter(|r| r.input.school == "CAS")
            .map(|r| &r.major_data)
            .collect();
        college_data::cas_double_major_shared_flexible_slots(&cas_majors)
    });

    for (degree_idx, resolved) in resolved_degrees.iter().enumerate() {
        let degree = &resolved.input;
        let major_data = &resolved.major_data;
        let concs = &resolved.concs;
        per_degree_validation[degree_idx]
            .refresh_pool_coverage_info(&major_data.requirements, &cu_map);
        let validation = &mut per_degree_validation[degree_idx];
        validation
            .fulfilled
            .sort_by_key(|r| r.requirement.get_category());

        let fulfilled = validation.fulfilled.clone();
        let unfulfilled = validation.unfulfilled.clone();
        let pool_coverage = validation.pool_coverage_info.clone();
        let is_secondary_cas_major = primary_cas_degree_index.is_some_and(|primary| {
            degree_idx != primary && degree.school == "CAS"
        });
        let is_primary_cas_major =
            primary_cas_degree_index.is_some_and(|primary| degree_idx == primary);

        let mut suggested = requirement::suggest_courses_for_requirements(
            &unfulfilled,
            &courses_for_validation,
            &cu_map,
            if cross_degree_optimizer {
                Some(&cross_state)
            } else {
                None
            },
            if cross_degree_optimizer {
                Some(degree_idx)
            } else {
                None
            },
        );

        if cross_degree_optimizer {
            for mapped in &mut suggested {
                let Some(instance_id) = mapped.instance_id.clone() else {
                    continue;
                };
                if !overlap_pair_slots.contains(&(degree_idx, instance_id.clone())) {
                    continue;
                }
                if fixed_course_overlap_slots.contains(&(degree_idx, instance_id.clone())) {
                    mapped.course_ids.clear();
                    continue;
                }
                if let Some(slot_id) = mapped
                    .requirement
                    .schedulable_placeholder_id(Some(&instance_id))
                {
                    mapped.course_ids = vec![slot_id];
                }
            }
        }

        if cross_degree_optimizer {
            for mapped in &suggested {
                for course_id in &mapped.course_ids {
                    if course::is_valid_course_code(course_id)
                        && cross_state
                            .can_claim(course_id, degree_idx, &cu_map)
                            .is_ok()
                    {
                        cross_state.register_claim(course_id, degree_idx, &cu_map);
                    }
                }
            }
        }

        // Collect unique suggested courses and requirement slots for the schedule
        for mapped in &suggested {
                if let Some(instance_id) = mapped.instance_id.as_deref() {
                    if let Some(hint) =
                        resolve_semester_hint(instance_id, &major_data.schedule_hints)
                    {
                        let adjusted =
                            adjust_schedule_hint_for_dual_degrees(hint, &degree_schools);
                        for course_id in &mapped.course_ids {
                            store_item_hint(&mut item_hints, course_id.clone(), adjusted.clone());
                        }
                    }
                }
                if is_graduate_degree(&degree.school) {
                    for course_id in &mapped.course_ids {
                        let target = if course::is_valid_course_code(course_id) {
                            ms_default_semester_target(course_id)
                        } else if requirement::is_requirement_slot_id(course_id) {
                            ms_default_semester_target_for_requirement(&mapped.requirement)
                        } else {
                            continue;
                        };
                        store_item_hint(
                            &mut item_hints,
                            course_id.clone(),
                            flexible_ms_hint(target.0, target.1),
                        );
                        ms_schedule_items.insert(course_id.clone());
                        let is_grad = (course::is_valid_course_code(course_id)
                            && course::is_graduate_level(course_id))
                            || (requirement::is_requirement_slot_id(course_id) && target.0 >= 3);
                        if is_grad {
                            ms_grad_schedule_items.insert(course_id.clone());
                        }
                    }
                } else {
                    for course_id in &mapped.course_ids {
                        ug_schedule_items.insert(course_id.clone());
                    }
                }
                for course_id in &mapped.course_ids {
                    if skip_cas_shared_schedule_item(
                        is_secondary_cas_major,
                        is_primary_cas_major,
                        mapped,
                        course_id,
                        cas_shared_flex_cap,
                    ) {
                        continue;
                    }
                    let allocated_to_degree = if cross_degree_optimizer {
                        cross_state
                            .claims
                            .get(course_id)
                            .map(|indices| indices.contains(&degree_idx))
                            .unwrap_or(false)
                    } else {
                        true
                    };
                    if course::is_valid_course_code(course_id)
                        && !all_suggested_courses.contains(course_id)
                        && !courses_for_validation.contains(course_id)
                        && allocated_to_degree
                    {
                        all_suggested_courses.push(course_id.clone());
                    } else if requirement::is_schedulable_requirement_slot_id(course_id)
                        && !all_requirement_slots.contains(course_id)
                    {
                        all_requirement_slots.push(course_id.clone());
                        let label = if mapped
                            .instance_id
                            .as_deref()
                            .is_some_and(|id| id.contains(":p"))
                        {
                            if let Some(pool_idx) = mapped
                                .instance_id
                                .as_deref()
                                .and_then(|id| id.split(':').next())
                                .and_then(|s| s.parse::<usize>().ok())
                            {
                                pool_coverage
                                    .iter()
                                    .find(|p| p.pool_index == pool_idx)
                                    .map(|p| format!("1 CU from {}", p.category))
                                    .unwrap_or_else(|| {
                                        mapped.requirement.slot_label_for_id(course_id)
                                    })
                            } else {
                                mapped.requirement.slot_label_for_id(course_id)
                            }
                        } else {
                            mapped.requirement.slot_label_for_id(course_id)
                        };
                        slot_labels.insert(course_id.clone(), label);
                    }
                }
            }

            for (key, hint) in &major_data.schedule_hints {
                if course::is_valid_course_code(key) {
                    store_item_hint(
                        &mut item_hints,
                        key.clone(),
                        adjust_schedule_hint_for_dual_degrees(hint.clone(), &degree_schools),
                    );
                }
            }

            let mut unapplicable = courses_for_validation.clone();
            for req in &fulfilled {
                for course in &req.course_ids {
                    unapplicable.retain(|x| x != course);
                }
            }

            let conc_info = requirement::extract_concentration_info(
                &major_data.requirements,
                &major_data.concentrations,
                &concs,
                &courses_for_validation,
                &cu_map,
                Some(&per_degree_validation[degree_idx]),
            );

            // Available concentration names
            let available_concs: Vec<String> = major_data.concentrations.as_ref()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();

            // Check if this major uses core concentrations
            let has_core = degree.major == "MEAM"
                || requirement::requirements_contain_concentration(&major_data.requirements);

            // Extract category order from requirement definition (includes nested CAS/DC children)
            let mut category_order: Vec<String> = Vec::new();
            for req in &major_data.requirements {
                req.collect_category_order(&mut category_order);
            }

            let cas_gen_ed = if degree.school == "CAS" {
                pool_coverage
                    .iter()
                    .find(|p| p.category == "General Education")
                    .map(|pool| {
                        college_data::build_cas_gen_ed_info(
                            pool,
                            &college_data::cas_auto_completed_sectors_for(&major_data.short_name),
                        )
                    })
            } else {
                None
            };

            let mut api_suggested = suggested;
            let mut api_unfulfilled = unfulfilled;
            if is_secondary_cas_major || is_primary_cas_major {
                let exclude = |mapped: &MappedRequirement| {
                    is_cas_double_major_excluded_mapped(
                        mapped,
                        is_secondary_cas_major,
                        is_primary_cas_major,
                        cas_shared_flex_cap,
                    )
                };
                api_suggested.retain(|m| !exclude(m));
                api_unfulfilled.retain(|m| !exclude(m));
            }

            degree_results.push(DegreeResult {
                school: degree.school.clone(),
                major: degree.major.clone(),
                fulfilled_requirements: fulfilled,
                unfulfilled_requirements: api_unfulfilled,
                suggested_for_unfulfilled: api_suggested,
                unapplicable_courses: unapplicable,
                pool_coverage_info: pool_coverage,
                concentration_info: conc_info,
                available_concentrations: available_concs,
                has_core_concentration: has_core,
                category_order,
                cas_gen_ed,
                error: None,
            });
    }

    if let Some(ref plan) = overlap_plan {
        for pair in &plan.pairs {
            let Some(course) =
                overlap_pair_fixed_course(pair, &plan.opportunities, &per_degree_validation)
            else {
                continue;
            };
            if !all_suggested_courses.contains(&course) {
                all_suggested_courses.push(course.clone());
            }
            ug_schedule_items.insert(course.clone());
            for slot_ref in &pair.slots {
                let degree_idx = slot_ref.degree_index;
                if cross_state
                    .can_claim(&course, degree_idx, &cu_map)
                    .is_ok()
                {
                    cross_state.register_claim(&course, degree_idx, &cu_map);
                }
                let major_data = &resolved_degrees[degree_idx].major_data;
                if let Some(hint) =
                    resolve_semester_hint(&slot_ref.slot_key, &major_data.schedule_hints)
                {
                    store_item_hint(
                        &mut item_hints,
                        course.clone(),
                        adjust_schedule_hint_for_dual_degrees(hint, &degree_schools),
                    );
                }
            }
        }
    }

    let mut schedulable_slot_lookup: HashMap<(usize, String), String> = HashMap::new();
    for (degree_idx, result) in degree_results.iter().enumerate() {
        for mapped in &result.suggested_for_unfulfilled {
            let instance = mapped.instance_id.clone().unwrap_or_default();
            for cid in &mapped.course_ids {
                if requirement::is_schedulable_requirement_slot_id(cid) {
                    schedulable_slot_lookup.insert((degree_idx, instance.clone()), cid.clone());
                    if let Some(rest) = cid.strip_prefix("req:") {
                        let scope = rest.split(":R:").next().unwrap_or(rest);
                        schedulable_slot_lookup.insert((degree_idx, scope.to_string()), cid.clone());
                        if let Some(base) = scope.split(':').next() {
                            schedulable_slot_lookup
                                .insert((degree_idx, base.to_string()), cid.clone());
                        }
                    }
                }
            }
        }
    }

    fn resolve_overlap_schedule_slot_id(
        degree_idx: usize,
        slot_key: &str,
        per_degree_validation: &[requirement::DegreeValidationResult],
        schedulable_slot_lookup: &HashMap<(usize, String), String>,
        all_requirement_slots: &[String],
    ) -> Option<String> {
        if let Some(id) = schedulable_slot_lookup.get(&(degree_idx, slot_key.to_string())) {
            return Some(id.clone());
        }
        if let Some(id) = all_requirement_slots.iter().find_map(|id| {
            if !requirement::is_schedulable_requirement_slot_id(id) {
                return None;
            }
            let rest = id.strip_prefix("req:")?;
            if rest == slot_key || rest.starts_with(&format!("{slot_key}:")) {
                Some(id.clone())
            } else {
                None
            }
        }) {
            return Some(id);
        }
        let validation = &per_degree_validation[degree_idx];
        let mapped = validation
            .unfulfilled
            .iter()
            .find(|m| m.instance_id.as_deref() == Some(slot_key))
            .or_else(|| {
                validation
                    .fulfilled
                    .iter()
                    .find(|m| m.partial && m.instance_id.as_deref() == Some(slot_key))
            })?;
        mapped
            .requirement
            .schedulable_placeholder_id(Some(slot_key))
    }

    let mut overlap_schedule_groups: Vec<OverlapScheduleGroup> = Vec::new();
    let mut suppressed_overlap_slots: HashSet<String> = HashSet::new();

    if let Some(ref plan) = overlap_plan {
        for pair in &plan.pairs {
            if overlap_pair_fixed_course(pair, &plan.opportunities, &per_degree_validation).is_some()
            {
                continue;
            }
            let mut members: Vec<overlap_planner::OverlapScheduleGroupMember> = Vec::new();
            let mut resolved_schedulable: Vec<String> = Vec::new();

            for slot_ref in &pair.slots {
                let schedule_id = resolve_overlap_schedule_slot_id(
                    slot_ref.degree_index,
                    &slot_ref.slot_key,
                    &per_degree_validation,
                    &schedulable_slot_lookup,
                    &all_requirement_slots,
                );

                if let Some(schedule_slot_id) = schedule_id {
                    resolved_schedulable.push(schedule_slot_id.clone());
                    let school = degree_schools
                        .get(slot_ref.degree_index)
                        .map(String::as_str)
                        .unwrap_or("");
                    let display_label = resolved_degrees
                        .get(slot_ref.degree_index)
                        .map(|r| {
                            overlap_member_display_label(
                                slot_ref,
                                school,
                                &r.major_data,
                            )
                        })
                        .unwrap_or_else(|| slot_ref.label.clone());
                    members.push(overlap_planner::OverlapScheduleGroupMember {
                        schedule_slot_id,
                        label: display_label,
                        degree_index: slot_ref.degree_index,
                        school: slot_ref.school.clone(),
                        major: slot_ref.major.clone(),
                    });
                } else {
                    let school = degree_schools
                        .get(slot_ref.degree_index)
                        .map(String::as_str)
                        .unwrap_or("");
                    let display_label = resolved_degrees
                        .get(slot_ref.degree_index)
                        .map(|r| {
                            overlap_member_display_label(
                                slot_ref,
                                school,
                                &r.major_data,
                            )
                        })
                        .unwrap_or_else(|| slot_ref.label.clone());
                    members.push(overlap_planner::OverlapScheduleGroupMember {
                        schedule_slot_id: overlap_planner::hint_key(
                            slot_ref.degree_index,
                            &slot_ref.slot_key,
                        ),
                        label: display_label,
                        degree_index: slot_ref.degree_index,
                        school: slot_ref.school.clone(),
                        major: slot_ref.major.clone(),
                    });
                }
            }

            if members.len() != 2 {
                continue;
            }
            if resolved_schedulable.len() != 2 {
                continue;
            }

            let group_id = overlap_planner::overlap_group_schedule_id(&pair.slots);
            for id in &resolved_schedulable {
                suppressed_overlap_slots.insert(id.clone());
            }

            let combined_label = members
                .iter()
                .map(|m| format!("{} ({})", m.label, m.major))
                .collect::<Vec<_>>()
                .join(" + ");
            slot_labels.insert(group_id.clone(), combined_label);

            overlap_schedule_groups.push(OverlapScheduleGroup {
                group_id,
                members,
                explanation: pair.explanation.clone(),
            });
        }
    }

    all_requirement_slots.retain(|s| !suppressed_overlap_slots.contains(s));
    for group in &overlap_schedule_groups {
        if !all_requirement_slots.contains(&group.group_id) {
            all_requirement_slots.push(group.group_id.clone());
            ug_schedule_items.insert(group.group_id.clone());
        }
    }

    for group in &overlap_schedule_groups {
        let mut best_target: Option<ScheduleHint> = None;
        for member in &group.members {
            let candidate = item_hints.get(&member.schedule_slot_id);
            if let Some(target) = candidate {
                let ord = target.ord();
                let best_ord = best_target.as_ref().map(|t| t.ord()).unwrap_or(i32::MAX);
                if ord < best_ord {
                    best_target = Some(target.clone());
                }
            }
        }
        if let Some(target) = best_target {
            item_hints.insert(group.group_id.clone(), target);
        }
    }

    let cas_major_refs: Vec<&Major> = resolved_degrees
        .iter()
        .map(|r| &r.major_data)
        .collect();
    suppress_redundant_cas_gened_flex_for_wh_overlaps(
        &mut all_requirement_slots,
        &mut ug_schedule_items,
        &mut degree_results,
        &overlap_schedule_groups,
        &degree_schools,
        &cas_major_refs,
    );

    let get_cu = |course_id: &str| -> f64 {
        if requirement::is_requirement_slot_id(course_id) {
            return 1.0;
        }
        *cu_map.get(course_id).unwrap_or(&1.0)
    };

    let place_in_semester = |plan: &mut SemesterPlan, item_id: &str| {
        if requirement::is_requirement_slot_id(item_id) {
            if !requirement::is_schedulable_requirement_slot_id(item_id)
                && !overlap_planner::is_overlap_schedule_group_id(item_id)
            {
                return;
            }
            if !plan.requirement_slots.contains(&item_id.to_string()) {
                plan.requirement_slots.push(item_id.to_string());
                plan.total_cu += get_cu(item_id);
            }
        } else if !plan.courses.contains(&item_id.to_string()) {
            plan.courses.push(item_id.to_string());
            plan.total_cu += get_cu(item_id);
        }
    };

    // Build schedule dynamically — expand semesters until ALL courses fit
    let allow_summer = payload.allow_summer.unwrap_or(true);
    let cu_limits = payload.semester_cu_limits.unwrap_or_default();

    let get_max_cu = |year: i32, semester: &str| -> f64 {
        let key = format!("{}-{}", year, semester);
        if let Some(&limit) = cu_limits.get(&key) {
            return limit;
        }
        default_semester_cu_limit(&degree_schools, year, semester)
    };

    // Helper: ensure schedule has semesters for a given year
    let mut schedule: Vec<SemesterPlan> = Vec::new();

    let ensure_year = |schedule: &mut Vec<SemesterPlan>, year: i32, allow_summer: bool| {
        let has_fall = schedule.iter().any(|p| p.year == year && p.semester == "Fall");
        if !has_fall {
            schedule.push(SemesterPlan {
                year,
                semester: "Fall".to_string(),
                courses: Vec::new(),
                requirement_slots: Vec::new(),
                total_cu: 0.0,
            });
            schedule.push(SemesterPlan {
                year,
                semester: "Spring".to_string(),
                courses: Vec::new(),
                requirement_slots: Vec::new(),
                total_cu: 0.0,
            });
            if allow_summer {
                schedule.push(SemesterPlan {
                    year,
                    semester: "Summer".to_string(),
                    courses: Vec::new(),
                    requirement_slots: Vec::new(),
                    total_cu: 0.0,
                });
            }
            schedule.sort_by(|a, b| {
                let sem_order = |s: &str| match s { "Fall" => 0, "Spring" => 1, "Summer" => 2, _ => 3 };
                a.year.cmp(&b.year).then(sem_order(&a.semester).cmp(&sem_order(&b.semester)))
            });
        }
    };

    let initial_years = undergrad_schedule_years(&degree_schools);
    for yr in 1..=initial_years {
        ensure_year(&mut schedule, yr, allow_summer);
    }

    // Place frozen items first (courses and requirement slots)
    for frozen in &frozen {
        ensure_year(&mut schedule, frozen.year, allow_summer);
        for plan in schedule.iter_mut() {
            if plan.year == frozen.year && plan.semester == frozen.semester {
                place_in_semester(plan, &frozen.course_id);
            }
        }
        all_suggested_courses.retain(|c| c != &frozen.course_id);
        all_requirement_slots.retain(|s| s != &frozen.course_id);
    }

    // Courses and requirement slots share one queue so UG items always compete
    // fairly with MS items regardless of item type.
    let mut remaining_items: Vec<String> = all_suggested_courses;
    for slot in all_requirement_slots {
        if !remaining_items.contains(&slot) {
            remaining_items.push(slot);
        }
    }

    let fixed_schedule_items: HashSet<String> = item_hints
        .iter()
        .filter(|(_, h)| h.mode == ScheduleHintMode::Fixed)
        .map(|(k, _)| k.clone())
        .collect();

    let has_undergrad = payload
        .degrees
        .iter()
        .any(|d| !is_graduate_degree(&d.school));

    let schedule_item_priority = |item: &str| -> u8 {
        if ug_schedule_items.contains(item) {
            0
        } else if !ms_schedule_items.contains(item) {
            1
        } else if ms_grad_schedule_items.contains(item) {
            3
        } else {
            2
        }
    };

    let item_fits_semester =
        |item_id: &str, plan_total_cu: f64, max_cu: f64| -> bool {
            let cu = get_cu(item_id);
            plan_total_cu + cu <= max_cu + CU_EPS
        };

    let find_best_fitting = |remaining: &[String],
                             plan_total_cu: f64,
                             max_cu: f64,
                             skip_ids: &HashSet<String>,
                             only_items: Option<&HashSet<String>>| -> Option<usize> {
        let mut best_idx: Option<usize> = None;
        let mut best_priority = u8::MAX;
        for (idx, item) in remaining.iter().enumerate() {
            if skip_ids.contains(item) {
                continue;
            }
            if fixed_schedule_items.contains(item) {
                continue;
            }
            if only_items.is_some_and(|set| !set.contains(item)) {
                continue;
            }
            if !item_fits_semester(item, plan_total_cu, max_cu) {
                continue;
            }
            let priority = schedule_item_priority(item);
            if priority < best_priority {
                best_priority = priority;
                best_idx = Some(idx);
            }
        }
        best_idx
    };

    let pop_best_fitting = |remaining: &mut Vec<String>,
                            plan_total_cu: f64,
                            max_cu: f64,
                            skip_ids: &HashSet<String>,
                            only_items: Option<&HashSet<String>>| -> Option<String> {
        find_best_fitting(remaining, plan_total_cu, max_cu, skip_ids, only_items)
            .map(|idx| remaining.remove(idx))
    };

    let try_place_item =
        |schedule: &mut Vec<SemesterPlan>, item_id: &str, year: i32, semester: &str| -> bool {
            ensure_year(schedule, year, allow_summer);
            for plan in schedule.iter_mut() {
                if plan.year == year && plan.semester == semester {
                    let already = if requirement::is_requirement_slot_id(item_id) {
                        plan.requirement_slots.contains(&item_id.to_string())
                    } else {
                        plan.courses.contains(&item_id.to_string())
                    };
                    if already {
                        return true;
                    }
                    let cu = get_cu(item_id);
                    let max_cu = get_max_cu(year, semester);
                    if plan.total_cu + cu <= max_cu || plan.total_cu == 0.0 {
                        place_in_semester(plan, item_id);
                        return true;
                    }
                    return false;
                }
            }
            false
        };

    let try_place_fixed_item =
        |schedule: &mut Vec<SemesterPlan>, item_id: &str, year: i32, semester: &str| -> bool {
            ensure_year(schedule, year, allow_summer);
            for plan in schedule.iter_mut() {
                if plan.year == year && plan.semester == semester {
                    let already = if requirement::is_requirement_slot_id(item_id) {
                        plan.requirement_slots.contains(&item_id.to_string())
                    } else {
                        plan.courses.contains(&item_id.to_string())
                    };
                    if already {
                        return true;
                    }
                    place_in_semester(plan, item_id);
                    return true;
                }
            }
            false
        };

    let undergrad_schedule_window = undergrad_schedule_years(&degree_schools);

    let place_with_template =
        |schedule: &mut Vec<SemesterPlan>, item_id: &str, hint: &ScheduleHint| -> bool {
            if hint.mode == ScheduleHintMode::Fixed {
                if try_place_item(schedule, item_id, hint.year, &hint.semester) {
                    return true;
                }
                return try_place_fixed_item(schedule, item_id, hint.year, &hint.semester);
            }
            let max_year = if has_undergrad && ms_schedule_items.contains(item_id) {
                12
            } else {
                undergrad_schedule_window
            };
            let candidates = if has_undergrad && ms_schedule_items.contains(item_id) {
                ms_grad_placement_candidates(
                    (hint.year, hint.semester.as_str()),
                    undergrad_schedule_window,
                    max_year,
                )
            } else {
                placement_semesters(hint, max_year)
            };
            for (year, semester) in candidates {
                if try_place_item(schedule, item_id, year, &semester) {
                    return true;
                }
            }
            false
        };

    let try_place_greedy =
        |schedule: &mut Vec<SemesterPlan>, item_id: &str, max_year: i32| -> bool {
            let mut best: Option<(i32, String)> = None;
            let mut best_load = f64::MAX;
            let mut best_tie_ord = i32::MIN;
            for year in 1..=max_year {
                for semester in ["Fall", "Spring"] {
                    let max_cu = get_max_cu(year, semester);
                    let load = schedule
                        .iter()
                        .find(|p| p.year == year && p.semester == semester)
                        .map(|p| p.total_cu)
                        .unwrap_or(0.0);
                    if !item_fits_semester(item_id, load, max_cu) {
                        continue;
                    }
                    let tie_ord = semester_order(year, semester);
                    if load < best_load
                        || (load == best_load && tie_ord < best_tie_ord)
                    {
                        best_load = load;
                        best_tie_ord = tie_ord;
                        best = Some((year, semester.to_string()));
                    }
                }
            }
            if let Some((year, semester)) = best {
                return try_place_item(schedule, item_id, year, &semester);
            }
            false
        };

    let sort_schedule_items = |items: &mut [String]| {
        items.sort_by_key(|item| {
            let fixed_rank = usize::from(
                !item_hints
                    .get(item)
                    .is_some_and(|h| h.mode == ScheduleHintMode::Fixed),
            );
            let template_ord = item_hints
                .get(item)
                .map(|h| h.ord())
                .unwrap_or(i32::MAX);
            (fixed_rank, schedule_item_priority(item), template_ord)
        });
    };

    let place_schedule_batch =
        |items: &mut Vec<String>, schedule: &mut Vec<SemesterPlan>, greedy_max_year: i32| {
            sort_schedule_items(items);
            let mut overflow = Vec::new();
            for item in items.drain(..) {
                let placed = if let Some(hint) = item_hints.get(&item) {
                    place_with_template(schedule, &item, hint)
                } else {
                    try_place_greedy(schedule, &item, greedy_max_year)
                };
                if !placed {
                    overflow.push(item);
                }
            }
            overflow
        };

    let partition_and_place = |remaining: &mut Vec<String>, schedule: &mut Vec<SemesterPlan>| {
        let mut items: Vec<String> = remaining.drain(..).collect();
        let mut overflow = Vec::new();
        if has_undergrad {
            let (mut ug_items, mut other_items): (Vec<String>, Vec<String>) =
                items.into_iter().partition(|item| ug_schedule_items.contains(item));
            overflow.extend(place_schedule_batch(
                &mut ug_items,
                schedule,
                undergrad_schedule_window,
            ));
            overflow.extend(place_schedule_batch(&mut other_items, schedule, 12));
        } else {
            overflow.extend(place_schedule_batch(&mut items, schedule, 12));
        }
        *remaining = overflow;
    };

    partition_and_place(&mut remaining_items, &mut schedule);

    if has_undergrad || !ms_schedule_items.is_empty() {
        remaining_items.sort_by_key(|item| schedule_item_priority(item));
    }

    let distribute = |remaining: &mut Vec<String>,
                        schedule: &mut Vec<SemesterPlan>,
                        allow_summer: bool,
                        skip_summer_for: &HashSet<String>,
                        only_items: Option<&HashSet<String>>,
                        year_range: Option<(i32, i32)>|
     -> bool {
        if remaining.is_empty() {
            return false;
        }
        let empty_skip: HashSet<String> = HashSet::new();
        let mut placed_any = false;
        loop {
            if remaining.is_empty() {
                break;
            }
            if only_items.is_some_and(|set| !remaining.iter().any(|item| set.contains(item))) {
                break;
            }

            let mut best_plan_idx: Option<usize> = None;
            let mut best_item_idx: Option<usize> = None;
            let mut best_load = f64::MAX;
            let mut best_tie_ord = i32::MAX;

            for (plan_idx, plan) in schedule.iter().enumerate() {
                if plan.semester == "Summer" && !allow_summer {
                    continue;
                }
                if let Some((min_y, max_y)) = year_range {
                    if plan.year < min_y || plan.year > max_y {
                        continue;
                    }
                }

                let max_cu = get_max_cu(plan.year, &plan.semester);
                let skip_ids = if plan.semester == "Summer" {
                    skip_summer_for
                } else {
                    &empty_skip
                };
                let Some(item_idx) = find_best_fitting(
                    remaining,
                    plan.total_cu,
                    max_cu,
                    skip_ids,
                    only_items,
                ) else {
                    continue;
                };

                let tie_ord = semester_order(plan.year, &plan.semester);
                if plan.total_cu < best_load
                    || (plan.total_cu == best_load && tie_ord < best_tie_ord)
                {
                    best_load = plan.total_cu;
                    best_tie_ord = tie_ord;
                    best_plan_idx = Some(plan_idx);
                    best_item_idx = Some(item_idx);
                }
            }

            let (plan_idx, item_idx) = match (best_plan_idx, best_item_idx) {
                (Some(p), Some(i)) => (p, i),
                _ => break,
            };
            let item = remaining.remove(item_idx);
            place_in_semester(&mut schedule[plan_idx], &item);
            placed_any = true;
        }
        placed_any
    };

    // UG overflow: fill earlier open semesters before any MS placement.
    if has_undergrad && remaining_items.iter().any(|item| ug_schedule_items.contains(item)) {
        loop {
            if !remaining_items
                .iter()
                .any(|item| ug_schedule_items.contains(item))
            {
                break;
            }
            let placed = distribute(
                &mut remaining_items,
                &mut schedule,
                allow_summer,
                &ms_grad_schedule_items,
                Some(&ug_schedule_items),
                Some((1, undergrad_schedule_window)),
            );
            if !placed {
                break;
            }
        }
    }

    // MS courses only after UG: years 3–4, then year 2, then year 5+.
    if has_undergrad {
        for (min_y, max_y) in [(3, 4), (2, 2)] {
            loop {
                if !remaining_items
                    .iter()
                    .any(|item| ms_schedule_items.contains(item))
                {
                    break;
                }
                let placed = distribute(
                    &mut remaining_items,
                    &mut schedule,
                    allow_summer,
                    &ms_grad_schedule_items,
                    Some(&ms_schedule_items),
                    Some((min_y, max_y)),
                );
                if !placed {
                    break;
                }
            }
        }
    }

    if dual_undergrad_only(&degree_schools) && allow_summer {
        let empty_skip: HashSet<String> = HashSet::new();
        for year in 1..=undergrad_schedule_window {
            loop {
                let before = remaining_items.len();
                if before == 0 {
                    break;
                }
                if !distribute(
                    &mut remaining_items,
                    &mut schedule,
                    true,
                    &empty_skip,
                    Some(&ug_schedule_items),
                    Some((year, year)),
                ) {
                    break;
                }
                if remaining_items.len() == before {
                    break;
                }
            }
        }
    }

    let squeeze_undergrad_remaining =
        |remaining: &mut Vec<String>,
         schedule: &mut Vec<SemesterPlan>,
         hints: &HashMap<String, ScheduleHint>| -> bool {
            if remaining.is_empty() || !has_undergrad {
                return false;
            }
            let mut placed_any = false;
            let semesters: Vec<&str> = if allow_summer {
                vec!["Fall", "Spring", "Summer"]
            } else {
                vec!["Fall", "Spring"]
            };
            let mut i = 0;
            while i < remaining.len() {
                let item = remaining[i].clone();
                let mut placed = false;
                if let Some(hint) = hints.get(&item) {
                    if hint.mode == ScheduleHintMode::Fixed {
                        if try_place_fixed_item(schedule, &item, hint.year, &hint.semester) {
                            placed = true;
                            placed_any = true;
                        }
                    } else {
                        'place: for year in 1..=undergrad_schedule_window {
                            for semester in &semesters {
                                if try_place_item(schedule, &item, year, *semester) {
                                    placed = true;
                                    placed_any = true;
                                    break 'place;
                                }
                            }
                        }
                    }
                } else {
                    'place: for year in 1..=undergrad_schedule_window {
                        for semester in &semesters {
                            if try_place_item(schedule, &item, year, *semester) {
                                placed = true;
                                placed_any = true;
                                break 'place;
                            }
                        }
                    }
                }
                if !placed
                    && dual_undergrad_only(&degree_schools)
                    && !hints
                        .get(&item)
                        .is_some_and(|h| h.mode == ScheduleHintMode::Fixed)
                {
                    let max_existing = schedule.iter().map(|p| p.year).max().unwrap_or(undergrad_schedule_window);
                    for year in (undergrad_schedule_window + 1)..=(max_existing + 1) {
                        ensure_year(schedule, year, allow_summer);
                        for semester in &semesters {
                            if try_place_item(schedule, &item, year, *semester) {
                                placed = true;
                                placed_any = true;
                                break;
                            }
                        }
                        if placed {
                            break;
                        }
                    }
                }
                if placed {
                    remaining.remove(i);
                } else {
                    i += 1;
                }
            }
            placed_any
        };

    loop {
        if remaining_items.is_empty() {
            break;
        }
        let only_items = if has_undergrad
            && remaining_items
                .iter()
                .any(|item| ms_schedule_items.contains(item))
        {
            Some(&ms_schedule_items)
        } else {
            None
        };
        let placed = distribute(
            &mut remaining_items,
            &mut schedule,
            allow_summer,
            &ms_grad_schedule_items,
            only_items,
            None,
        );
        if remaining_items.is_empty() {
            break;
        }
        if !placed {
            if dual_undergrad_only(&degree_schools) {
                let flexible_overflow: Vec<String> = remaining_items
                    .iter()
                    .filter(|item| {
                        item_hints
                            .get(*item)
                            .is_some_and(|h| h.mode == ScheduleHintMode::Flexible)
                    })
                    .cloned()
                    .collect();
                for item in flexible_overflow {
                    item_hints.remove(&item);
                }
                if squeeze_undergrad_remaining(&mut remaining_items, &mut schedule, &item_hints) {
                    continue;
                }
            }
            let max_year = schedule.iter().map(|p| p.year).max().unwrap_or(4);
            ensure_year(&mut schedule, max_year + 1, allow_summer);
        }
    }

    let mut fixed_remaining: Vec<String> = remaining_items
        .iter()
        .filter(|item| {
            item_hints
                .get(*item)
                .is_some_and(|h| h.mode == ScheduleHintMode::Fixed)
        })
        .cloned()
        .collect();
    for item in fixed_remaining.drain(..) {
        if let Some(hint) = item_hints.get(&item) {
            let placed = try_place_item(&mut schedule, &item, hint.year, &hint.semester)
                || try_place_fixed_item(&mut schedule, &item, hint.year, &hint.semester);
            if placed {
                remaining_items.retain(|i| i != &item);
            }
        }
    }

    let cross_degree_summary = if degree_schools.len() > 1 {
        cross_degree::enforce_claim_rules(&mut cross_state, &cu_map);

        if cross_degree_optimizer {
            for (degree_idx, result) in degree_results.iter_mut().enumerate() {
                requirement::filter_mapped_requirements_by_allocation(
                    &mut result.fulfilled_requirements,
                    degree_idx,
                    &cross_state.claims,
                );
                requirement::filter_mapped_requirements_by_allocation(
                    &mut result.suggested_for_unfulfilled,
                    degree_idx,
                    &cross_state.claims,
                );
                requirement::filter_mapped_requirements_by_allocation(
                    &mut result.unfulfilled_requirements,
                    degree_idx,
                    &cross_state.claims,
                );
                requirement::filter_concentration_info_by_claims(
                    &mut result.concentration_info,
                    degree_idx,
                    &cross_state.claims,
                );
            }
        }

        let mut summary = cross_state.to_summary();
        summary.violations = cross_degree::detect_violations(
            &cross_state.claims,
            &degree_schools,
            &cu_map,
        );
        Some(summary)
    } else {
        None
    };

    for plan in schedule.iter_mut() {
        plan.total_cu = plan
            .courses
            .iter()
            .map(|c| get_cu(c))
            .chain(plan.requirement_slots.iter().map(|s| get_cu(s)))
            .sum();
    }

    ScheduleOutput {
        schedule,
        degree_results,
        slot_labels,
        cross_degree_summary,
        overlap_plan,
        overlap_schedule_groups,
        error: None,
    }
}
