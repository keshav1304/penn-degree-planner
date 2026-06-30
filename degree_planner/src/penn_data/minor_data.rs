use std::collections::HashMap;

use crate::Major;
use crate::Requirement;

struct RestrictionBuilder {
    number: i32,
    category: Option<String>,
    attr: Option<Vec<String>>,
    excluding: Option<Vec<String>>,
}

fn restriction(number: i32) -> RestrictionBuilder {
    RestrictionBuilder {
        number,
        category: None,
        attr: None,
        excluding: None,
    }
}

impl RestrictionBuilder {
    fn category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    fn attr(mut self, attrs: &[&str]) -> Self {
        self.attr = Some(attrs.iter().map(|s| s.to_string()).collect());
        self
    }

    fn excluding(mut self, courses: &[&str]) -> Self {
        self.excluding = Some(courses.iter().map(|s| s.to_string()).collect());
        self
    }
}

impl From<RestrictionBuilder> for Requirement {
    fn from(b: RestrictionBuilder) -> Self {
        Requirement::Restriction {
            category: b.category,
            department: None,
            cu: None,
            level: None,
            max_level: None,
            attr: b.attr,
            excluding: b.excluding,
            number: b.number,
            no_school: None,
        }
    }
}

fn single(category: &str, courses: &[&str]) -> Requirement {
    Requirement::SingleCourse {
        category: Some(category.to_string()),
        possibilities: courses.iter().map(|s| s.to_string()).collect(),
    }
}

fn any_of(category: &str, possibilities: Vec<Requirement>) -> Requirement {
    Requirement::AnyOf {
        category: Some(category.to_string()),
        possibilities,
    }
}

const EENT_CORE_EXCLUSIONS: &[&str] = &[
    "EAS 5450", "EAS 5460", "EAS 5490", "EAS 5410", "EAS 5430",
];

pub fn eent_concentration_names() -> Vec<String> {
    vec!["Standard".to_string(), "Fellows".to_string()]
}

/// Engineering Entrepreneurship minor (6 CU) per Penn catalog.
pub fn create_eent_minor(concentration: &str) -> Major {
    let core = if concentration == "Fellows" {
        vec![
            single("EENT Core", &["EAS 5410"]),
            single("EENT Core", &["EAS 5430"]),
        ]
    } else {
        vec![
            single("EENT Core", &["EAS 5450"]),
            any_of(
                "EENT Core",
                vec![
                    single("EENT Core", &["EAS 5460"]),
                    single("EENT Core", &["EAS 5490"]),
                ],
            ),
        ]
    };

    let electives: Requirement = restriction(4)
        .category("EENT Electives")
        .attr(&["EUNP"])
        .excluding(EENT_CORE_EXCLUSIONS)
        .into();

    Major {
        short_name: "EENT".to_string(),
        name: "Engineering Entrepreneurship".to_string(),
        requirements: core.into_iter().chain(std::iter::once(electives)).collect(),
        concentrations: None,
        schedule_hints: HashMap::new(),
    }
}
