use std::collections::HashMap;

use crate::Requirement;
use crate::requirement::PoolConstraint;
use crate::schedule_template::{schedule_hints_from_array, ScheduleHint, Semester};

// --- Restriction builder ---

#[derive(Clone)]
pub struct RestrictionBuilder {
    number: i32,
    category: Option<String>,
    department: Option<Vec<String>>,
    level: Option<i32>,
    max_level: Option<i32>,
    cu: Option<i32>,
    attr: Option<Vec<String>>,
    excluding: Option<Vec<String>>,
    no_school: Option<String>,
}

pub fn restriction(number: i32) -> RestrictionBuilder {
    RestrictionBuilder {
        number,
        category: None,
        department: None,
        level: None,
        max_level: None,
        cu: None,
        attr: None,
        excluding: None,
        no_school: None,
    }
}

impl RestrictionBuilder {
    pub fn category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    pub fn departments(mut self, depts: &[&str]) -> Self {
        self.department = Some(depts.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn level(mut self, level: i32) -> Self {
        self.level = Some(level);
        self
    }

    pub fn max_level(mut self, max_level: i32) -> Self {
        self.max_level = Some(max_level);
        self
    }

    pub fn cu(mut self, cu: i32) -> Self {
        self.cu = Some(cu);
        self
    }

    pub fn attr(mut self, attrs: &[&str]) -> Self {
        self.attr = Some(attrs.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn excluding(mut self, courses: &[&str]) -> Self {
        self.excluding = Some(courses.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn no_school(mut self, school: &str) -> Self {
        self.no_school = Some(school.to_string());
        self
    }
}

impl From<RestrictionBuilder> for Requirement {
    fn from(b: RestrictionBuilder) -> Self {
        Requirement::Restriction {
            category: b.category,
            department: b.department,
            cu: b.cu,
            level: b.level,
            max_level: b.max_level,
            attr: b.attr,
            excluding: b.excluding,
            number: b.number,
            no_school: b.no_school,
        }
    }
}

// --- Requirement composers ---

pub fn single(category: &str, courses: &[&str]) -> Requirement {
    Requirement::SingleCourse {
        category: Some(category.to_string()),
        possibilities: courses.iter().map(|s| s.to_string()).collect(),
    }
}

pub fn code(courses: &[&str]) -> Requirement {
    Requirement::SingleCourse {
        category: None,
        possibilities: courses.iter().map(|s| s.to_string()).collect(),
    }
}

pub fn course_group(category: &str, number: i32, children: Vec<Requirement>) -> Requirement {
    Requirement::CourseGroup {
        category: Some(category.to_string()),
        number,
        possibilities: children,
    }
}

pub fn course_group_from_codes(category: &str, number: i32, codes: &[&str]) -> Requirement {
    course_group(
        category,
        number,
        codes
            .iter()
            .map(|c| code(&[*c]))
            .collect(),
    )
}

pub fn any_of(category: &str, possibilities: Vec<Requirement>) -> Requirement {
    Requirement::AnyOf {
        category: Some(category.to_string()),
        possibilities,
    }
}

pub fn any_of_opt(category: Option<String>, possibilities: Vec<Requirement>) -> Requirement {
    Requirement::AnyOf {
        category,
        possibilities,
    }
}

pub fn all_of(category: Option<String>, requirements: Vec<Requirement>) -> Requirement {
    Requirement::AllOf {
        category,
        requirements,
    }
}

pub fn concentration(category: &str, number: i32, requirements: Vec<Requirement>) -> Requirement {
    Requirement::Concentration {
        category: Some(category.to_string()),
        number,
        requirements,
    }
}

pub fn repeat_req(req: &Requirement, n: usize) -> Vec<Requirement> {
    std::iter::repeat_n(req.clone(), n).collect()
}

pub fn required_slots(category: &str, codes: &[&str]) -> Vec<Requirement> {
    codes
        .iter()
        .map(|code| single(category, &[*code]))
        .collect()
}

pub fn unrestricted_elective(label: &str) -> Requirement {
    restriction(1).category(label).into()
}

// --- Pool builders ---

pub fn pool_constraint(
    requirement: Requirement,
    count: i32,
    consumption_group: &str,
) -> PoolConstraint {
    PoolConstraint {
        requirement,
        count,
        consumption_group: Some(consumption_group.to_string()),
    }
}

pub fn attr_restriction(label: &str, attr: &str) -> Requirement {
    restriction(1).category(label).attr(&[attr]).into()
}

pub fn attrs_restriction(label: &str, attrs: &[&str]) -> Requirement {
    restriction(1).category(label).attr(attrs).into()
}

pub fn no_school_restriction(label: &str, school: &str) -> Requirement {
    restriction(1).category(label).no_school(school).into()
}

pub fn attr_pool_constraint(label: &str, attr: &str, count: i32, group: &str) -> PoolConstraint {
    pool_constraint(attr_restriction(label, attr), count, group)
}

pub fn attrs_pool_constraint(label: &str, attrs: &[&str], count: i32, group: &str) -> PoolConstraint {
    pool_constraint(attrs_restriction(label, attrs), count, group)
}

pub fn no_school_pool_constraint(label: &str, school: &str, count: i32, group: &str) -> PoolConstraint {
    pool_constraint(no_school_restriction(label, school), count, group)
}

pub fn single_pool_constraint(label: &str, codes: &[&str], count: i32, group: &str) -> PoolConstraint {
    pool_constraint(single(label, codes), count, group)
}

pub fn course_pool(
    category: &str,
    fixed_slots: Vec<Requirement>,
    flexible_slots: i32,
    constraints: Vec<PoolConstraint>,
) -> Requirement {
    Requirement::CoursePool {
        category: Some(category.to_string()),
        fixed_slots,
        flexible_slots,
        constraints,
    }
}

// --- Schedule hints ---

pub fn schedule_hints(
    semesters: &[Semester],
    overrides: &[(&str, Semester)],
) -> HashMap<String, ScheduleHint> {
    let mut hints = schedule_hints_from_array(semesters);
    for (course, sem) in overrides {
        hints.insert(course.to_string(), (*sem).into());
    }
    hints
}
