use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::course;

pub const UNDERGRAD_GRAD_CU_LIMIT: f64 = 3.0;
const CU_EPS: f64 = 0.001;

pub fn is_graduate_degree(school: &str) -> bool {
    school == "SEAS_MS"
}

fn lookup_course_cu(cu_map: &HashMap<String, f64>, course: &str) -> f64 {
    *cu_map.get(course).unwrap_or(&1.0)
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrossDegreeViolationKind {
    TooManyDegrees,
    GradGradOverlap,
    UndergradGradCuCap,
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

        let school = &self.degree_schools[degree_idx];
        let Some(existing) = self.claims.get(course) else {
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
            if let Some(&owner) = self.grad_course_owner.get(course) {
                if owner != degree_idx {
                    return Err(CrossDegreeViolationKind::GradGradOverlap);
                }
            }
        }

        if !course::is_graduate_level(course)
            && self.claim_crosses_undergrad_grad(degree_idx, existing)
        {
            let cu = lookup_course_cu(cu_map, course);
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

        let existing_before = self
            .claims
            .get(course)
            .cloned()
            .unwrap_or_default();

        if existing_before.contains(&degree_idx) {
            return;
        }

        self.claims
            .entry(course.to_string())
            .or_default()
            .insert(degree_idx);

        if course::is_graduate_level(course)
            && is_graduate_degree(&self.degree_schools[degree_idx])
        {
            self.grad_course_owner
                .insert(course.to_string(), degree_idx);
        }

        if !course::is_graduate_level(course)
            && self.claim_crosses_undergrad_grad(degree_idx, &existing_before)
        {
            self.undergrad_grad_cu_used += lookup_course_cu(cu_map, course);
        }
    }

    pub fn rebuild_from_allocations(
        &mut self,
        allocations: &HashMap<String, HashSet<usize>>,
        cu_map: &HashMap<String, f64>,
    ) {
        self.claims = allocations.clone();
        self.grad_course_owner.clear();
        self.undergrad_grad_cu_used = 0.0;

        for (course, degree_indices) in allocations {
            if !course::is_valid_course_code(course) {
                continue;
            }
            for &degree_idx in degree_indices {
                if course::is_graduate_level(course)
                    && is_graduate_degree(&self.degree_schools[degree_idx])
                {
                    self.grad_course_owner.insert(course.clone(), degree_idx);
                }
            }
            if !course::is_graduate_level(course) && crosses_undergrad_grad(course, degree_indices, &self.degree_schools)
            {
                self.undergrad_grad_cu_used += lookup_course_cu(cu_map, course);
            }
        }
    }

    pub fn to_summary(&self) -> CrossDegreeSummary {
        let mut course_allocations: HashMap<String, Vec<CourseAllocation>> = HashMap::new();

        for (course, degree_indices) in &self.claims {
            if !course::is_valid_course_code(course) {
                continue;
            }
            let uses_budget = !course::is_graduate_level(course)
                && crosses_undergrad_grad(course, degree_indices, &self.degree_schools);

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
            course_allocations.insert(course.clone(), allocs);
        }

        CrossDegreeSummary {
            undergrad_grad_cu_used: self.undergrad_grad_cu_used,
            undergrad_grad_cu_limit: UNDERGRAD_GRAD_CU_LIMIT,
            course_allocations,
            violations: self.violations.clone(),
        }
    }
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
            "{course} would exceed the {UNDERGRAD_GRAD_CU_LIMIT} CU undergrad→grad double-count limit"
        ),
    }
}

pub fn detect_violations(
    allocations: &HashMap<String, HashSet<usize>>,
    degree_schools: &[String],
    cu_map: &HashMap<String, f64>,
) -> Vec<CrossDegreeViolation> {
    let mut violations = Vec::new();
    let mut undergrad_grad_cu = 0.0;

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

        if !course::is_graduate_level(course)
            && crosses_undergrad_grad(course, degree_indices, degree_schools)
        {
            undergrad_grad_cu += lookup_course_cu(cu_map, course);
        }
    }

    if undergrad_grad_cu > UNDERGRAD_GRAD_CU_LIMIT + CU_EPS {
        violations.push(CrossDegreeViolation {
            course_id: String::new(),
            kind: CrossDegreeViolationKind::UndergradGradCuCap,
            message: format!(
                "Undergraduate→graduate double-count budget exceeded ({undergrad_grad_cu:.1} / {UNDERGRAD_GRAD_CU_LIMIT} CU)"
            ),
            degree_indices: vec![],
        });
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::requirement::{MappedRequirement, Requirement};

    fn cu_map() -> HashMap<String, f64> {
        HashMap::from([
            ("CIS 1200".to_string(), 1.0),
            ("CIS 5190".to_string(), 1.0),
            ("MATH 1400".to_string(), 1.0),
            ("STAT 4300".to_string(), 1.0),
            ("MEAM 1100".to_string(), 0.5),
        ])
    }

    #[test]
    fn blocks_third_degree_claim() {
        let schools = vec!["SEAS".to_string(), "WH".to_string(), "SEAS_MS".to_string()];
        let majors = vec!["CIS".to_string(), "WH_FL".to_string(), "MS_ROBO".to_string()];
        let mut state = CrossDegreeState::new(schools, majors);
        let cu = cu_map();

        state.register_claim("CIS 1200", 0, &cu);
        state.register_claim("CIS 1200", 1, &cu);
        assert!(matches!(
            state.can_claim("CIS 1200", 2, &cu),
            Err(CrossDegreeViolationKind::TooManyDegrees)
        ));
    }

    #[test]
    fn blocks_grad_grad_overlap() {
        let schools = vec!["SEAS_MS".to_string(), "SEAS_MS".to_string()];
        let majors = vec!["MS_ROBO".to_string(), "MS_EE".to_string()];
        let mut state = CrossDegreeState::new(schools, majors);
        let cu = cu_map();

        state.register_claim("CIS 5190", 0, &cu);
        assert!(matches!(
            state.can_claim("CIS 5190", 1, &cu),
            Err(CrossDegreeViolationKind::GradGradOverlap)
        ));
    }

    #[test]
    fn undergrad_grad_cu_cap() {
        let schools = vec!["SEAS".to_string(), "SEAS_MS".to_string()];
        let majors = vec!["CIS".to_string(), "MS_ROBO".to_string()];
        let mut state = CrossDegreeState::new(schools, majors);
        let cu = cu_map();

        state.register_claim("CIS 1200", 0, &cu);
        state.register_claim("CIS 1200", 1, &cu);
        state.register_claim("MATH 1400", 0, &cu);
        state.register_claim("MATH 1400", 1, &cu);
        state.register_claim("STAT 4300", 0, &cu);
        state.register_claim("STAT 4300", 1, &cu);
        state.register_claim("MEAM 1100", 0, &cu);
        assert!(matches!(
            state.can_claim("MEAM 1100", 1, &cu),
            Err(CrossDegreeViolationKind::UndergradGradCuCap)
        ));
    }
}
