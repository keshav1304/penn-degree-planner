use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::course;
use crate::course_relations;

pub const UNDERGRAD_GRAD_CU_LIMIT: f64 = 3.0;
pub const CU_EPS: f64 = 0.001;

pub fn is_graduate_degree(school: &str) -> bool {
    school == "SEAS_MS"
}

/// Cross-degree overlap optimizer (shared courses, paired requirement blocks, allocation filtering).
/// Applies only when every selected degree is undergraduate.
pub fn cross_degree_optimizer_applicable(degree_schools: &[String]) -> bool {
    degree_schools.len() >= 2 && degree_schools.iter().all(|s| !is_graduate_degree(s))
}

/// Overlap discovery whenever two or more degrees are selected (including grad↔undergrad).
pub fn overlap_plan_applicable(degree_schools: &[String]) -> bool {
    degree_schools.len() >= 2
}

pub fn undergrad_degree_count(degree_schools: &[String]) -> usize {
    degree_schools
        .iter()
        .filter(|s| !is_graduate_degree(s))
        .count()
}

/// Two or more undergraduate degrees selected (grad programs may also be present).
pub fn has_dual_undergrad(degree_schools: &[String]) -> bool {
    undergrad_degree_count(degree_schools) >= 2
}

/// Every undergraduate degree is in CAS (dual-college rule).
pub fn all_undergrad_degrees_are_cas(degree_schools: &[String]) -> bool {
    let ug: Vec<_> = degree_schools
        .iter()
        .filter(|s| !is_graduate_degree(s))
        .collect();
    ug.len() >= 2 && ug.iter().all(|s| *s == "CAS")
}

fn lookup_course_cu(cu_map: &HashMap<String, f64>, course: &str) -> f64 {
    if let Some(&cu) = cu_map.get(course) {
        return cu;
    }
    let canon = course_relations::canonical(course);
    *cu_map.get(&canon).unwrap_or(&1.0)
}

fn shared_undergrad_grad_cu(
    claims: &HashMap<String, HashSet<usize>>,
    degree_schools: &[String],
    cu_map: &HashMap<String, f64>,
) -> f64 {
    claims
        .iter()
        .filter(|(course, indices)| {
            course::is_valid_course_code(course)
                && crosses_undergrad_grad(course, indices, degree_schools)
        })
        .map(|(course, _)| lookup_course_cu(cu_map, course))
        .sum()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrossDegreeViolationKind {
    TooManyDegrees,
    GradGradOverlap,
    UndergradGradCuCap,
    MutuallyExclusive,
    /// Multiple also_offered_as spellings of the same course appear in the plan.
    AlsoOfferedSameCourse,
    /// A planned course is missing one or more catalog prerequisites.
    MissingPrerequisite,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossDegreeViolation {
    pub course_id: String,
    pub kind: CrossDegreeViolationKind,
    pub message: String,
    pub degree_indices: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourseAllocation {
    pub degree_index: usize,
    pub school: String,
    pub major: String,
    #[serde(skip_serializing)]
    pub uses_undergrad_grad_budget: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossDegreeSummary {
    #[serde(skip_serializing)]
    pub undergrad_grad_cu_used: f64,
    #[serde(skip_serializing)]
    pub undergrad_grad_cu_limit: f64,
    pub course_allocations: HashMap<String, Vec<CourseAllocation>>,
    pub violations: Vec<CrossDegreeViolation>,
}

#[derive(Debug, Clone)]
pub struct CrossDegreeState {
    pub claims: HashMap<String, HashSet<usize>>,
    pub grad_course_owner: HashMap<String, usize>,
    pub undergrad_grad_cu_used: f64,
    pub violations: Vec<CrossDegreeViolation>,
    /// Courses required for overlay-style undergrad concentrations (course → UG degree indices).
    pub ug_concentration_courses: HashMap<String, HashSet<usize>>,
    degree_schools: Vec<String>,
    degree_majors: Vec<String>,
}

impl CrossDegreeState {
    pub fn new(degree_schools: Vec<String>, degree_majors: Vec<String>) -> Self {
        Self {
            claims: HashMap::new(),
            grad_course_owner: HashMap::new(),
            undergrad_grad_cu_used: 0.0,
            violations: Vec::new(),
            ug_concentration_courses: HashMap::new(),
            degree_schools,
            degree_majors,
        }
    }

    fn claim_crosses_undergrad_grad(
        &self,
        degree_idx: usize,
        existing: &HashSet<usize>,
    ) -> bool {
        let new_is_grad = is_graduate_degree(&self.degree_schools[degree_idx]);
        let has_grad = existing
            .iter()
            .any(|&i| is_graduate_degree(&self.degree_schools[i]));
        let has_undergrad = existing
            .iter()
            .any(|&i| !is_graduate_degree(&self.degree_schools[i]));

        if new_is_grad && has_undergrad {
            return true;
        }
        if !new_is_grad && has_grad {
            return true;
        }
        false
    }

    pub fn can_claim(
        &self,
        course: &str,
        degree_idx: usize,
        cu_map: &HashMap<String, f64>,
    ) -> Result<(), CrossDegreeViolationKind> {
        if !course::is_valid_course_code(course) {
            return Ok(());
        }

        let course_key = course_relations::canonical(course);
        let school = &self.degree_schools[degree_idx];
        let Some(existing) = self.claims.get(&course_key) else {
            return Ok(());
        };

        if existing.contains(&degree_idx) {
            return Ok(());
        }

        if existing.len() >= 2 {
            return Err(CrossDegreeViolationKind::TooManyDegrees);
        }

        if course::is_graduate_level(course) && is_graduate_degree(school) {
            for &other_idx in existing {
                if is_graduate_degree(&self.degree_schools[other_idx]) {
                    return Err(CrossDegreeViolationKind::GradGradOverlap);
                }
            }
            if let Some(&owner) = self.grad_course_owner.get(&course_key) {
                if owner != degree_idx {
                    return Err(CrossDegreeViolationKind::GradGradOverlap);
                }
            }
        }

        if self.claim_crosses_undergrad_grad(degree_idx, existing) {
            let cu = lookup_course_cu(cu_map, &course_key);
            if self.undergrad_grad_cu_used + cu > UNDERGRAD_GRAD_CU_LIMIT + CU_EPS {
                return Err(CrossDegreeViolationKind::UndergradGradCuCap);
            }
        }

        Ok(())
    }

    pub fn register_claim(&mut self, course: &str, degree_idx: usize, cu_map: &HashMap<String, f64>) {
        if !course::is_valid_course_code(course) {
            return;
        }

        let course_key = course_relations::canonical(course);
        let existing_before = self
            .claims
            .get(&course_key)
            .cloned()
            .unwrap_or_default();

        if existing_before.contains(&degree_idx) {
            return;
        }

        self.claims
            .entry(course_key.clone())
            .or_default()
            .insert(degree_idx);

        if course::is_graduate_level(course)
            && is_graduate_degree(&self.degree_schools[degree_idx])
        {
            self.grad_course_owner
                .insert(course_key.clone(), degree_idx);
        }

        if self.claim_crosses_undergrad_grad(degree_idx, &existing_before) {
            self.undergrad_grad_cu_used += lookup_course_cu(cu_map, &course_key);
        }
    }

    pub fn rebuild_from_allocations(
        &mut self,
        allocations: &HashMap<String, HashSet<usize>>,
        cu_map: &HashMap<String, f64>,
    ) {
        let mut normalized: HashMap<String, HashSet<usize>> = HashMap::new();
        for (course, degree_indices) in allocations {
            if !course::is_valid_course_code(course) {
                continue;
            }
            let key = course_relations::canonical(course);
            normalized
                .entry(key)
                .or_default()
                .extend(degree_indices.iter().copied());
        }

        self.claims = normalized;
        self.grad_course_owner.clear();
        self.undergrad_grad_cu_used = 0.0;

        for (course, degree_indices) in &self.claims {
            for &degree_idx in degree_indices {
                if course::is_graduate_level(course)
                    && is_graduate_degree(&self.degree_schools[degree_idx])
                {
                    self.grad_course_owner.insert(course.clone(), degree_idx);
                }
            }
            if crosses_undergrad_grad(course, degree_indices, &self.degree_schools) {
                self.undergrad_grad_cu_used += lookup_course_cu(cu_map, course);
            }
        }
    }

    pub fn to_summary(&self) -> CrossDegreeSummary {
        self.to_summary_with_plan_codes(None)
    }

    /// Mirror allocations under every alias spelling that appears in `plan_codes`.
    pub fn to_summary_with_plan_codes(
        &self,
        plan_codes: Option<&HashSet<String>>,
    ) -> CrossDegreeSummary {
        let mut course_allocations: HashMap<String, Vec<CourseAllocation>> = HashMap::new();

        for (course, degree_indices) in &self.claims {
            if !course::is_valid_course_code(course) {
                continue;
            }
            let uses_budget =
                crosses_undergrad_grad(course, degree_indices, &self.degree_schools);

            let mut allocs: Vec<CourseAllocation> = degree_indices
                .iter()
                .map(|&degree_index| CourseAllocation {
                    degree_index,
                    school: self.degree_schools[degree_index].clone(),
                    major: self.degree_majors[degree_index].clone(),
                    uses_undergrad_grad_budget: uses_budget,
                })
                .collect();
            allocs.sort_by_key(|a| a.degree_index);
            course_allocations.insert(course.clone(), allocs.clone());

            if let Some(plan) = plan_codes {
                for code in plan {
                    if course_relations::equivalent(code, course) && code != course {
                        course_allocations.insert(code.clone(), allocs.clone());
                    }
                }
                // Also mirror known aliases that appear under any spelling in the plan.
                for alias in course_relations::aliases(course) {
                    if plan.iter().any(|p| course_relations::equivalent(p, alias))
                        && !course_allocations.contains_key(alias)
                    {
                        // Prefer the plan's displayed spelling when available.
                        if let Some(display) = plan.iter().find(|p| course_relations::equivalent(p, alias)) {
                            course_allocations.insert(display.clone(), allocs.clone());
                        }
                    }
                }
            }
        }

        CrossDegreeSummary {
            undergrad_grad_cu_used: self.undergrad_grad_cu_used,
            undergrad_grad_cu_limit: UNDERGRAD_GRAD_CU_LIMIT,
            course_allocations,
            violations: self.violations.clone(),
        }
    }
}

/// Whether a course may count toward an undergraduate minor.
///
/// Allowed: shared with an undergrad major, minor-exclusive (no major claim), or
/// undergrad + graduate major(s) + minor together.
/// Disallowed: graduate major(s) + minor with no undergrad major.
pub fn course_may_count_toward_minor(
    course: &str,
    major_claims: &HashMap<String, HashSet<usize>>,
    major_degree_schools: &[String],
) -> bool {
    if !course::is_valid_course_code(course) {
        return true;
    }
    let Some(indices) = major_claims.get(course) else {
        return true;
    };
    if indices.is_empty() {
        return true;
    }
    indices
        .iter()
        .any(|&i| !is_graduate_degree(&major_degree_schools[i]))
}

pub fn crosses_undergrad_grad(
    _course: &str,
    degree_indices: &HashSet<usize>,
    degree_schools: &[String],
) -> bool {
    let has_grad = degree_indices
        .iter()
        .any(|&i| is_graduate_degree(&degree_schools[i]));
    let has_undergrad = degree_indices
        .iter()
        .any(|&i| !is_graduate_degree(&degree_schools[i]));
    has_grad && has_undergrad
}

fn violation_message(kind: &CrossDegreeViolationKind, course: &str) -> String {
    match kind {
        CrossDegreeViolationKind::TooManyDegrees => {
            format!("{course} cannot count toward more than two degrees")
        }
        CrossDegreeViolationKind::GradGradOverlap => {
            format!("{course} cannot overlap across multiple graduate degrees")
        }
        CrossDegreeViolationKind::UndergradGradCuCap => format!(
            "{course} would exceed the {UNDERGRAD_GRAD_CU_LIMIT} CU total undergrad↔masters double-count limit"
        ),
        CrossDegreeViolationKind::MutuallyExclusive => {
            format!("{course} conflicts with a mutually exclusive course on the schedule")
        }
        CrossDegreeViolationKind::AlsoOfferedSameCourse => {
            format!("{course} appears under more than one also-offered course code")
        }
        CrossDegreeViolationKind::MissingPrerequisite => {
            format!("{course} is missing a catalog prerequisite")
        }
    }
}

/// Detect mutex pairs where **both** partners appear in `schedule_codes` (grid courses only).
pub fn detect_mutex_violations(schedule_codes: &HashSet<String>) -> Vec<CrossDegreeViolation> {
    let mut violations = Vec::new();
    let mut reported_pairs: HashSet<(String, String)> = HashSet::new();

    let codes: Vec<String> = schedule_codes
        .iter()
        .filter(|c| course::is_valid_course_code(c))
        .cloned()
        .collect();

    for a in &codes {
        for partner in course_relations::mutex_partners(a) {
            let Some(b) = codes
                .iter()
                .find(|c| course_relations::equivalent(c, partner) || c.as_str() == partner.as_str())
            else {
                continue;
            };
            if course_relations::equivalent(a, b) {
                continue;
            }
            let mut ordered = (
                course_relations::canonical(a),
                course_relations::canonical(b),
            );
            if ordered.0 > ordered.1 {
                std::mem::swap(&mut ordered.0, &mut ordered.1);
            }
            if !reported_pairs.insert(ordered) {
                continue;
            }
            let message = format!("{a} is mutually exclusive with {b}.");
            for course_id in [a.clone(), b.clone()] {
                violations.push(CrossDegreeViolation {
                    course_id,
                    kind: CrossDegreeViolationKind::MutuallyExclusive,
                    message: message.clone(),
                    degree_indices: vec![],
                });
            }
        }
    }

    violations
}

/// When the plan lists two or more also-offered spellings of one course, emit a warn-only note.
pub fn detect_also_offered_duplicates(plan_codes: &[String]) -> Vec<CrossDegreeViolation> {
    let mut by_canonical: HashMap<String, Vec<String>> = HashMap::new();
    let mut seen_spelling: HashSet<String> = HashSet::new();

    for raw in plan_codes {
        if !course::is_valid_course_code(raw) {
            continue;
        }
        let n = course_relations::normalize_code(raw);
        if !seen_spelling.insert(n.clone()) {
            continue;
        }
        let canon = course_relations::canonical(&n);
        // Only warn when the catalog actually has an also-offered cluster.
        if course_relations::aliases(&n).len() < 2 {
            continue;
        }
        by_canonical.entry(canon).or_default().push(n);
    }

    let mut violations = Vec::new();
    for spellings in by_canonical.values() {
        if spellings.len() < 2 {
            continue;
        }
        let mut ordered = spellings.clone();
        ordered.sort();
        let message = if ordered.len() == 2 {
            format!("{} and {} are the same course.", ordered[0], ordered[1])
        } else {
            let last = ordered.last().unwrap();
            let head = ordered[..ordered.len() - 1].join(", ");
            format!("{head}, and {last} are the same course.")
        };
        for course_id in &ordered {
            violations.push(CrossDegreeViolation {
                course_id: course_id.clone(),
                kind: CrossDegreeViolationKind::AlsoOfferedSameCourse,
                message: message.clone(),
                degree_indices: vec![],
            });
        }
    }
    violations
}

/// Warn when planned courses are missing catalog prerequisites (does not add courses).
/// `subject_codes` are courses to check (taken/frozen); `plan_codes` is the full set
/// that may satisfy those prerequisites (includes schedule placements).
pub fn detect_missing_prerequisites(
    subject_codes: &[String],
    plan_codes: &[String],
) -> Vec<CrossDegreeViolation> {
    crate::prereq::missing_prereq_messages(subject_codes, plan_codes)
        .into_iter()
        .map(|(course_id, message)| CrossDegreeViolation {
            course_id,
            kind: CrossDegreeViolationKind::MissingPrerequisite,
            message,
            degree_indices: vec![],
        })
        .collect()
}

pub fn detect_violations(
    allocations: &HashMap<String, HashSet<usize>>,
    degree_schools: &[String],
    cu_map: &HashMap<String, f64>,
) -> Vec<CrossDegreeViolation> {
    let mut violations = Vec::new();

    for (course, degree_indices) in allocations {
        if !course::is_valid_course_code(course) {
            continue;
        }

        if degree_indices.len() > 2 {
            violations.push(CrossDegreeViolation {
                course_id: course.clone(),
                kind: CrossDegreeViolationKind::TooManyDegrees,
                message: violation_message(&CrossDegreeViolationKind::TooManyDegrees, course),
                degree_indices: degree_indices.iter().copied().collect(),
            });
        }

        if course::is_graduate_level(course) {
            let grad_degrees: Vec<usize> = degree_indices
                .iter()
                .copied()
                .filter(|&i| is_graduate_degree(&degree_schools[i]))
                .collect();
            if grad_degrees.len() > 1 {
                violations.push(CrossDegreeViolation {
                    course_id: course.clone(),
                    kind: CrossDegreeViolationKind::GradGradOverlap,
                    message: violation_message(&CrossDegreeViolationKind::GradGradOverlap, course),
                    degree_indices: grad_degrees,
                });
            }
        }
    }

    let undergrad_grad_cu = shared_undergrad_grad_cu(allocations, degree_schools, cu_map);
    if undergrad_grad_cu > UNDERGRAD_GRAD_CU_LIMIT + CU_EPS {
        violations.push(CrossDegreeViolation {
            course_id: String::new(),
            kind: CrossDegreeViolationKind::UndergradGradCuCap,
            message: format!(
                "Undergraduate↔masters double-count budget exceeded ({undergrad_grad_cu:.1} / {UNDERGRAD_GRAD_CU_LIMIT} CU total across all degrees)"
            ),
            degree_indices: vec![],
        });
    }

    violations
}

fn choose_two_degree_indices(
    course: &str,
    indices: &[usize],
    degree_schools: &[String],
    ug_concentration_courses: &HashMap<String, HashSet<usize>>,
) -> HashSet<usize> {
    if indices.len() <= 2 {
        return indices.iter().copied().collect();
    }
    let in_ug_conc = |idx: usize| {
        ug_concentration_courses
            .get(course)
            .map(|set| set.contains(&idx))
            .unwrap_or(false)
    };
    let mut sorted = indices.to_vec();
    sorted.sort_by_key(|&i| (is_graduate_degree(&degree_schools[i]), !in_ug_conc(i), i));
    HashSet::from([sorted[0], sorted[1]])
}

/// Trim in-memory claims until all cross-degree rules are satisfied.
pub fn enforce_claim_rules(state: &mut CrossDegreeState, cu_map: &HashMap<String, f64>) {
    loop {
        let violations = detect_violations(&state.claims, &state.degree_schools, cu_map);
        if violations.is_empty() {
            break;
        }

        let mut changed = false;

        for violation in &violations {
            match violation.kind {
                CrossDegreeViolationKind::MutuallyExclusive => {
                    // Warn-only: never strip mutex mates from claims.
                }
                CrossDegreeViolationKind::AlsoOfferedSameCourse => {
                    // Warn-only: aliases are already collapsed for claims/CU.
                }
                CrossDegreeViolationKind::MissingPrerequisite => {
                    // Warn-only: never auto-add missing prerequisites.
                }
                CrossDegreeViolationKind::TooManyDegrees => {
                    let course = &violation.course_id;
                    if let Some(indices) = state.claims.get(course).cloned() {
                        let keep = choose_two_degree_indices(
                            course,
                            &indices.iter().copied().collect::<Vec<_>>(),
                            &state.degree_schools,
                            &state.ug_concentration_courses,
                        );
                        if let Some(set) = state.claims.get_mut(course) {
                            set.retain(|idx| keep.contains(idx));
                            if set.is_empty() {
                                state.claims.remove(course);
                            }
                            changed = true;
                        }
                    }
                }
                CrossDegreeViolationKind::GradGradOverlap => {
                    let course = &violation.course_id;
                    if violation.degree_indices.len() <= 1 {
                        continue;
                    }
                    let best_idx = *violation
                        .degree_indices
                        .iter()
                        .min_by_key(|&&i| i)
                        .unwrap_or(&violation.degree_indices[0]);
                    if let Some(set) = state.claims.get_mut(course) {
                        set.retain(|&idx| {
                            !is_graduate_degree(&state.degree_schools[idx]) || idx == best_idx
                        });
                        if set.is_empty() {
                            state.claims.remove(course);
                        }
                        changed = true;
                    }
                }
                CrossDegreeViolationKind::UndergradGradCuCap => {
                    let mut shared: Vec<(String, f64)> = state
                        .claims
                        .iter()
                        .filter(|(course, indices)| {
                            course::is_valid_course_code(course)
                                && crosses_undergrad_grad(course, indices, &state.degree_schools)
                        })
                        .map(|(course, _)| (course.clone(), lookup_course_cu(cu_map, course)))
                        .collect();

                    let ug_conc_priority = |course: &str| -> usize {
                        state
                            .ug_concentration_courses
                            .get(course)
                            .map(|set| set.len())
                            .unwrap_or(0)
                    };

                    shared.sort_by(|a, b| {
                        ug_conc_priority(&a.0)
                            .cmp(&ug_conc_priority(&b.0))
                            .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                            .then_with(|| a.0.cmp(&b.0))
                    });

                    let mut used =
                        shared_undergrad_grad_cu(&state.claims, &state.degree_schools, cu_map);
                    while used > UNDERGRAD_GRAD_CU_LIMIT + CU_EPS {
                        let Some((course, cu)) = shared.pop() else {
                            break;
                        };
                        if let Some(indices) = state.claims.get_mut(&course) {
                            let grad_indices: Vec<usize> = indices
                                .iter()
                                .copied()
                                .filter(|&i| is_graduate_degree(&state.degree_schools[i]))
                                .collect();
                            for idx in grad_indices {
                                indices.remove(&idx);
                            }
                            if indices.is_empty() {
                                state.claims.remove(&course);
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
    }

    state.rebuild_from_allocations(&state.claims.clone(), cu_map);
}
