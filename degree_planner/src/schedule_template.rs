use std::collections::{HashMap, HashSet};

use crate::Requirement;
use crate::course;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleHintMode {
    /// Exact semester — only user frozen placement may override.
    Fixed,
    /// Prefer the template semester; may backfill earlier open semesters, then at-or-after target.
    Flexible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleHint {
    pub year: i32,
    pub semester: String,
    pub mode: ScheduleHintMode,
}

impl ScheduleHint {
    pub fn flexible(sem: Semester) -> Self {
        Self {
            year: sem.year,
            semester: sem.name.to_string(),
            mode: ScheduleHintMode::Flexible,
        }
    }

    pub fn fixed(sem: Semester) -> Self {
        Self {
            year: sem.year,
            semester: sem.name.to_string(),
            mode: ScheduleHintMode::Fixed,
        }
    }

    pub fn ord(&self) -> i32 {
        semester_order(self.year, &self.semester)
    }
}

impl From<Semester> for ScheduleHint {
    fn from(sem: Semester) -> Self {
        Self::flexible(sem)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semester {
    pub year: i32,
    pub name: &'static str,
}

pub const Y1F: Semester = Semester { year: 1, name: "Fall" };
pub const Y1S: Semester = Semester { year: 1, name: "Spring" };
pub const Y2F: Semester = Semester { year: 2, name: "Fall" };
pub const Y2S: Semester = Semester { year: 2, name: "Spring" };
pub const Y3F: Semester = Semester { year: 3, name: "Fall" };
pub const Y3S: Semester = Semester { year: 3, name: "Spring" };
pub const Y4F: Semester = Semester { year: 4, name: "Fall" };
pub const Y4S: Semester = Semester { year: 4, name: "Spring" };

impl Semester {
    pub fn to_pair(self) -> (i32, String) {
        (self.year, self.name.to_string())
    }
}

fn insert_hint(hints: &mut HashMap<String, ScheduleHint>, index: usize, sem: Semester) {
    hints.insert(index.to_string(), ScheduleHint::flexible(sem));
}

/// Mark specific course codes as mandatory semester placements.
pub fn insert_fixed_course_hints(
    hints: &mut HashMap<String, ScheduleHint>,
    courses: &[(&str, Semester)],
) {
    for (course, sem) in courses {
        hints.insert(course.to_string(), ScheduleHint::fixed(*sem));
    }
}

/// Build a flat requirement list and schedule hints from `(semester, requirement)` pairs.
pub fn scheduled(entries: Vec<(Semester, Requirement)>) -> (Vec<Requirement>, HashMap<String, ScheduleHint>) {
    let mut requirements = Vec::with_capacity(entries.len());
    let mut schedule_hints = HashMap::new();
    for (sem, req) in entries {
        insert_hint(&mut schedule_hints, requirements.len(), sem);
        requirements.push(req);
    }
    (requirements, schedule_hints)
}

/// Append a batch of requirements tagged with the same semester (for dynamic `.chain()` sections).
pub fn append_semester(
    requirements: &mut Vec<Requirement>,
    schedule_hints: &mut HashMap<String, ScheduleHint>,
    sem: Semester,
    batch: Vec<Requirement>,
) {
    for req in batch {
        insert_hint(schedule_hints, requirements.len(), sem);
        requirements.push(req);
    }
}

/// Optional bulk template: one semester entry per top-level requirement, in list order.
pub fn schedule_hints_from_array(schedule: &[Semester]) -> HashMap<String, ScheduleHint> {
    let mut hints = HashMap::new();
    for (index, sem) in schedule.iter().enumerate() {
        insert_hint(&mut hints, index, *sem);
    }
    hints
}

/// Resolve a schedule hint for an instance id, inheriting from the parent index for
/// CoursePool children (`"20:f0"` / `"20:p0"` → `"20"`).
pub fn resolve_semester_hint(
    id: &str,
    hints: &HashMap<String, ScheduleHint>,
) -> Option<ScheduleHint> {
    if let Some(h) = hints.get(id) {
        return Some(h.clone());
    }
    id.split(':')
        .next()
        .and_then(|base| hints.get(base).cloned())
}

/// Semester candidates for placing an item, in try order.
pub fn placement_semesters(hint: &ScheduleHint, max_year: i32) -> Vec<(i32, String)> {
    if hint.mode == ScheduleHintMode::Fixed {
        return vec![(hint.year, hint.semester.clone())];
    }

    let target_ord = hint.ord();
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    let mut push = |year: i32, semester: &str| {
        let ord = semester_order(year, semester);
        if seen.insert(ord) {
            out.push((year, semester.to_string()));
        }
    };

    for year in 1..=max_year {
        for sem in ["Fall", "Spring"] {
            if semester_order(year, sem) < target_ord {
                push(year, sem);
            }
        }
    }
    for (year, semester) in later_semesters((hint.year, hint.semester.as_str()), max_year) {
        push(year, &semester);
    }
    out
}

/// When two degrees supply hints for the same key, keep the stricter / later binding.
pub fn merge_schedule_hint(existing: Option<&ScheduleHint>, new: ScheduleHint) -> ScheduleHint {
    match existing {
        None => new,
        Some(old) if new.mode == ScheduleHintMode::Fixed && old.mode == ScheduleHintMode::Flexible => {
            new
        }
        Some(old) if old.mode == ScheduleHintMode::Fixed => old.clone(),
        Some(old) if new.ord() > old.ord() => new,
        Some(old) => old.clone(),
    }
}

pub fn semester_key(year: i32, semester: &str) -> String {
    format!("{}-{}", year, semester)
}

pub fn semester_order(year: i32, semester: &str) -> i32 {
    let sem = match semester {
        "Fall" => 0,
        "Spring" => 1,
        "Summer" => 2,
        _ => 3,
    };
    year * 10 + sem
}

/// Semesters from `(year, name)` onward in chronological order (Fall/Spring only).
pub fn later_semesters(
    start: (i32, &str),
    max_year: i32,
) -> Vec<(i32, String)> {
    let start_ord = semester_order(start.0, start.1);
    let mut out = Vec::new();
    for year in 1..=max_year.max(1) {
        for sem in ["Fall", "Spring"] {
            if semester_order(year, sem) >= start_ord {
                out.push((year, sem.to_string()));
            }
        }
    }
    out
}

/// Default semester target for MS degree courses.
///
/// Undergrad-level MS courses start early; graduate-level courses prefer upper years but may
/// still backfill empty semesters in years 1–4 when scheduling alongside a UG degree.
pub fn ms_default_semester_target(course_id: &str) -> (i32, String) {
    if course::is_valid_course_code(course_id) && !course::is_graduate_level(course_id) {
        (1, "Fall".to_string())
    } else {
        (3, "Fall".to_string())
    }
}

/// Placement order for MS graduate items when co-scheduled with an undergrad degree.
///
/// 1. Preferred window: `target` through `undergrad_window_end` (typically years 3–4).
/// 2. Backfill: open spots in year 2 only.
/// 3. Extension: years beyond the undergrad window (e.g. a 5th year) when still needed.
pub fn ms_grad_placement_candidates(
    target: (i32, &str),
    undergrad_window_end: i32,
    max_year: i32,
) -> Vec<(i32, String)> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    let mut push = |year: i32, semester: &str| {
        let ord = semester_order(year, semester);
        if seen.insert(ord) {
            out.push((year, semester.to_string()));
        }
    };

    for (year, semester) in later_semesters(target, undergrad_window_end.max(target.0)) {
        push(year, &semester);
    }
    for semester in ["Fall", "Spring"] {
        push(2, semester);
    }
    if max_year > undergrad_window_end {
        for (year, semester) in later_semesters((undergrad_window_end + 1, "Fall"), max_year) {
            push(year, &semester);
        }
    }

    out
}

/// Default semester target for MS requirement slots (restriction placeholders).
pub fn ms_default_semester_target_for_requirement(req: &Requirement) -> (i32, String) {
    match req {
        Requirement::Restriction { level, .. } if level.is_some_and(|l| l < 5000) => {
            (1, "Fall".to_string())
        }
        _ => (3, "Fall".to_string()),
    }
}
