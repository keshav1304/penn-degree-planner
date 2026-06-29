use std::collections::HashMap;

use crate::Major;
use crate::Requirement;
use crate::schedule_template::{
    schedule_hints_from_array, ScheduleHint, Semester, Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S,
};

// --- Generic builders ---

struct RestrictionBuilder {
    number: i32,
    category: Option<String>,
    department: Option<Vec<String>>,
    level: Option<i32>,
    attr: Option<Vec<String>>,
    excluding: Option<Vec<String>>,
}

fn restriction(number: i32) -> RestrictionBuilder {
    RestrictionBuilder {
        number,
        category: None,
        department: None,
        level: None,
        attr: None,
        excluding: None,
    }
}

impl RestrictionBuilder {
    fn category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    fn departments(mut self, depts: &[&str]) -> Self {
        self.department = Some(depts.iter().map(|s| s.to_string()).collect());
        self
    }

    fn level(mut self, level: i32) -> Self {
        self.level = Some(level);
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
            department: b.department,
            cu: None,
            level: b.level,
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

fn course_group(category: &str, number: i32, children: Vec<Requirement>) -> Requirement {
    Requirement::CourseGroup {
        category: Some(category.to_string()),
        number,
        possibilities: children,
    }
}

fn course_group_from_codes(category: &str, number: i32, codes: &[&str]) -> Requirement {
    course_group(
        category,
        number,
        codes
            .iter()
            .map(|code| Requirement::SingleCourse {
                category: None,
                possibilities: vec![(*code).to_string()],
            })
            .collect(),
    )
}

fn any_of(category: &str, possibilities: Vec<Requirement>) -> Requirement {
    Requirement::AnyOf {
        category: Some(category.to_string()),
        possibilities,
    }
}

fn repeat_req(req: &Requirement, n: usize) -> Vec<Requirement> {
    std::iter::repeat_n(req.clone(), n).collect()
}

fn required_slots(category: &str, codes: &[&str]) -> Vec<Requirement> {
    codes
        .iter()
        .map(|code| single(category, &[*code]))
        .collect()
}

fn schedule_hints(
    semesters: &[Semester],
    overrides: &[(&str, Semester)],
) -> HashMap<String, ScheduleHint> {
    let mut hints = schedule_hints_from_array(semesters);
    for (course, sem) in overrides {
        hints.insert(course.to_string(), (*sem).into());
    }
    hints
}

fn placeholder_ms_major(short_name: &str, display_name: &str) -> Major {
    Major {
        short_name: short_name.to_string(),
        name: display_name.to_string(),
        requirements: vec![restriction(10)
            .category("Program Requirements (placeholder)")
            .into()],
        schedule_hints: HashMap::new(),
        concentrations: None,
    }
}

// --- Shared dept slices ---

const SEAS_ELECTIVE_DEPTS: &[&str] = &[
    "ESE", "CIS", "CIT", "IPD", "MEAM", "MSE", "EAS", "ENM",
];

const GRAD_GENERAL_ELECTIVE_DEPTS: &[&str] = &[
    "CIS", "ESE", "MEAM", "EAS", "CIT", "ENM", "IPD", "MATH",
];

const EAS_RESEARCH_EXCLUSIONS: &[&str] = &["EAS 8950", "EAS 8960", "EAS 8970"];

const MS_BE_SEAS_DEPTS: &[&str] = &[
    "BE", "CBE", "CIS", "CIT", "EAS", "ENM", "ESE", "IPD", "MEAM", "MSE",
];

const MS_EE_CORE_COURSES: &[&str] = &[
    "ESE 5090",
    "ESE 5100",
    "ESE 5130",
    "ESE 5210",
    "ESE 5230",
    "ESE 5250",
    "ESE 5290",
    "ESE 5360",
    "ESE 5150",
    "ESE 5160",
    "ESE 5180",
    "ESE 5190",
    "ESE 5320",
    "ESE 5390",
    "ESE 5700",
    "ESE 5720",
    "ESE 5730",
    "ESE 5750",
    "ESE 5780",
    "ESE 5800",
    "ESE 6680",
    "ESE 5000",
    "ESE 5030",
    "ESE 5050",
    "ESE 5060",
    "ESE 5070",
    "ESE 5140",
    "ESE 5280",
    "ESE 5300",
    "ESE 5310",
    "ESE 5380",
    "ESE 5420",
    "ESE 5460",
    "ESE 6500",
];

const MCIT_REQUIRED_COURSES: &[&str] = &[
    "CIT 5910",
    "CIT 5920",
    "CIT 5930",
    "CIT 5940",
    "CIT 5950",
    "CIT 5960",
];

// --- Majors ---

pub fn create_ms_ee_major() -> Major {
    Major {
        short_name: "MS_EE".to_string(),
        name: "Electrical Engineering, MSE".to_string(),
        requirements: vec![
            course_group_from_codes("Electrical Engineering Core", 5, MS_EE_CORE_COURSES),
            restriction(2)
                .category("Electrical Engineering Electives")
                .departments(&["ESE"])
                .level(5000)
                .into(),
            restriction(1)
                .category("SEAS Elective")
                .departments(SEAS_ELECTIVE_DEPTS)
                .level(5000)
                .into(),
            restriction(2)
                .category("Open Electives")
                .level(5000)
                .into(),
        ],
        schedule_hints: HashMap::new(),
        concentrations: None,
    }
}

pub fn create_ms_robo_major() -> Major {
    let tech_elective: Requirement = restriction(1)
        .category("Technical Elective")
        .attr(&["EMRT"])
        .into();
    let general_elective: Requirement = restriction(1)
        .category("General Elective")
        .departments(GRAD_GENERAL_ELECTIVE_DEPTS)
        .level(5000)
        .excluding(EAS_RESEARCH_EXCLUSIONS)
        .into();

    Major {
        short_name: "MS_ROBO".to_string(),
        name: "Robotics, MSE".to_string(),
        requirements: [
            vec![course_group(
                "Foundational Courses",
                3,
                vec![
                    single(
                        "Artificial Intelligence",
                        &["CIS 5190", "CIS 5200", "CIS 5210", "ESE 6500"],
                    ),
                    single(
                        "Robot Design and Analysis",
                        &["MEAM 5100", "MEAM 5200", "MEAM 6200"],
                    ),
                    single(
                        "Control",
                        &["ESE 5000", "ESE 5050", "MEAM 5130", "MEAM 5170"],
                    ),
                    single("Perception", &["CIS 5800", "CIS 5810", "CIS 6800"]),
                ],
            )],
            repeat_req(&tech_elective, 5),
            repeat_req(&general_elective, 3),
        ]
        .concat(),
        schedule_hints: HashMap::new(),
        concentrations: None,
    }
}

pub fn create_ms_meam_major() -> Major {
    // TODO: populate MS Mechanical Engineering and Applied Mechanics requirements
    placeholder_ms_major("MS_MEAM", "Mechanical Engineering and Applied Mechanics, MSE")
}

pub fn create_ms_cis_major() -> Major {
    let cis_or_non_cis = any_of(
        "CIS or Non-CIS Electives",
        vec![
            restriction(1)
                .category("CIS Elective")
                .departments(&["CIS"])
                .level(5000)
                .into(),
            restriction(1)
                .category("Non-CIS Elective")
                .level(5000)
                .attr(&["EMCI"])
                .into(),
        ],
    );

    Major {
        short_name: "MS_CIS".to_string(),
        name: "Computer Science, MSE".to_string(),
        requirements: [
            vec![
                single(
                    "Core Courses",
                    &[
                        "CIS 5050", "CIS 5480", "CIS 5530", "CIS 5550", "CIS 5010",
                    ],
                ),
                single("Core Courses", &["CIS 5020", "CIS 5110"]),
                single(
                    "Core Courses",
                    &["CIS 5200", "CIS 5190", "CIS 5210"],
                ),
                single(
                    "Core Courses",
                    &[
                        "CIS 5050", "CIS 5480", "CIS 5530", "CIS 5550", "CIS 5020",
                        "CIS 5110", "CIS 5000", "CIS 5710",
                    ],
                ),
                restriction(3)
                    .category("CIS Elective")
                    .departments(&["CIS"])
                    .level(5000)
                    .into(),
            ],
            repeat_req(&cis_or_non_cis, 3),
        ]
        .concat(),
        schedule_hints: schedule_hints(
            &[Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S],
            &[
                ("CIS 5050", Y1F),
                ("CIS 5020", Y1S),
                ("CIS 5200", Y2F),
                ("CIS 5000", Y2S),
            ],
        ),
        concentrations: None,
    }
}

pub fn create_ms_mse_major() -> Major {
    // TODO: populate MS Materials Science and Engineering requirements
    placeholder_ms_major("MS_MSE", "Materials Science and Engineering, MSE")
}

pub fn create_ms_be_major() -> Major {
    let bio_science: Requirement = restriction(1)
        .category("Biological Science")
        .level(5000)
        .attr(&["EMBS", "EPBS"])
        .into();
    let seas_grad: Requirement = restriction(1)
        .departments(MS_BE_SEAS_DEPTS)
        .level(5000)
        .into();
    let be_elective: Requirement = restriction(1)
        .category("Bioengineering Elective")
        .departments(&["BE"])
        .level(5000)
        .into();
    let math: Requirement = restriction(1)
        .category("Math")
        .level(5000)
        .attr(&["EMBM", "EPBM"])
        .into();

    Major {
        short_name: "MS_BE".to_string(),
        name: "Bioengineering, MSE".to_string(),
        requirements: [
            repeat_req(&math, 2),
            repeat_req(&bio_science, 2),
            repeat_req(&be_elective, 2),
            vec![
                any_of(
                    "Bioengineering Elective",
                    vec![seas_grad.clone(), bio_science.clone()],
                ),
                restriction(1)
                    .category("General Elective")
                    .level(5000)
                    .into(),
                any_of(
                    "Bioengineering Elective",
                    vec![seas_grad.clone(), bio_science.clone()],
                ),
            ],
        ]
        .concat(),
        schedule_hints: schedule_hints(
            &[Y1F, Y1F, Y1F, Y1S, Y1S, Y1S, Y2F, Y2F, Y2S],
            &[("BE 9990", Y2F)],
        ),
        concentrations: None,
    }
}

pub fn create_mcit_major() -> Major {
    let mut requirements = required_slots("Required Courses", MCIT_REQUIRED_COURSES);
    requirements.push(
        restriction(3)
            .category("Electives")
            .departments(&["CIS"])
            .level(5000)
            .into(),
    );
    requirements.push(restriction(1).category("Free Elective").into());

    Major {
        short_name: "MCIT".to_string(),
        name: "Computer & Information Technology, MCIT".to_string(),
        requirements,
        schedule_hints: schedule_hints(
            &[Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S],
            &[
                ("CIT 5910", Y1F),
                ("CIT 5920", Y1F),
                ("CIT 5930", Y1F),
                ("CIT 5940", Y1S),
                ("CIT 5950", Y1S),
                ("CIT 5960", Y1S),
            ],
        ),
        concentrations: None,
    }
}
