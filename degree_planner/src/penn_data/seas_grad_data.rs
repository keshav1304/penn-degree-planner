use std::collections::HashMap;

use crate::Major;
use crate::Requirement;
use crate::schedule_template::{schedule_hints_from_array, Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S};

fn placeholder_ms_major(short_name: &str, display_name: &str) -> Major {
    Major {
        short_name: short_name.to_string(),
        name: display_name.to_string(),
        requirements: vec![Requirement::Restriction {
            category: Some("Program Requirements (placeholder)".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: None,
            excluding: None,
            number: 10,
            no_school: None,
        }],
        schedule_hints: HashMap::new(),
        concentrations: None,
    }
}

pub fn create_ms_ee_major() -> Major {
    Major {
        short_name: "MS_EE".to_string(),
        name: "Electrical Engineering, MSE".to_string(),
        requirements: vec![
            ms_ee_core_courses(),
            Requirement::Restriction {
                category: Some("Electrical Engineering Electives".to_string()),
                department: Some(vec!["ESE".to_string()]),
                cu: None,
                level: Some(5000),
                attr: None,
                excluding: None,
                number: 2,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("SEAS Elective".to_string()),
                department: Some(vec![
                    "ESE".to_string(),
                    "CIS".to_string(),
                    "CIT".to_string(),
                    "IPD".to_string(),
                    "MEAM".to_string(),
                    "MSE".to_string(),
                    "EAS".to_string(),
                    "ENM".to_string(),
                ]),
                cu: None,
                level: Some(5000),
                attr: None,
                excluding: None,
                number: 1,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("Open Electives".to_string()),
                department: None,
                cu: None,
                level: Some(5000),
                attr: None,
                excluding: None,
                number: 2,
                no_school: None,
            },
        ],
        schedule_hints: HashMap::new(),
        concentrations: None,
    }
}

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

fn ms_ee_core_courses() -> Requirement {
    Requirement::CourseGroup {
        category: Some("Electrical Engineering Core".to_string()),
        number: 5,
        possibilities: MS_EE_CORE_COURSES
            .iter()
            .map(|code| Requirement::SingleCourse {
                category: None,
                possibilities: vec![(*code).to_string()],
            })
            .collect(),
    }
}

fn ms_robo_ai_area() -> Requirement {
    Requirement::SingleCourse {
        category: Some("Artificial Intelligence".to_string()),
        possibilities: vec![
            "CIS 5190".to_string(),
            "CIS 5200".to_string(),
            "CIS 5210".to_string(),
            "ESE 6500".to_string(),
        ],
    }
}

fn ms_robo_robot_design_area() -> Requirement {
    Requirement::SingleCourse {
        category: Some("Robot Design and Analysis".to_string()),
        possibilities: vec![
            "MEAM 5100".to_string(),
            "MEAM 5200".to_string(),
            "MEAM 6200".to_string(),
        ],
    }
}

fn ms_robo_control_area() -> Requirement {
    Requirement::SingleCourse {
        category: Some("Control".to_string()),
        possibilities: vec![
            "ESE 5000".to_string(),
            "ESE 5050".to_string(),
            "MEAM 5130".to_string(),
            "MEAM 5170".to_string(),
        ],
    }
}

fn ms_robo_perception_area() -> Requirement {
    Requirement::SingleCourse {
        category: Some("Perception".to_string()),
        possibilities: vec![
            "CIS 5800".to_string(),
            "CIS 5810".to_string(),
            "CIS 6800".to_string(),
        ],
    }
}

/// Complete 1 course from 3 out of 4 foundational areas (3 courses total).
fn ms_robo_foundational_courses() -> Requirement {
    Requirement::CourseGroup {
        category: Some("Foundational Courses".to_string()),
        number: 3,
        possibilities: vec![
            ms_robo_ai_area(),
            ms_robo_robot_design_area(),
            ms_robo_control_area(),
            ms_robo_perception_area(),
        ],
    }
}

fn ms_grad_general_elective() -> Requirement {
    Requirement::Restriction {
        category: Some("General Elective".to_string()),
        department: Some(vec![
            "CIS".to_string(),
            "ESE".to_string(),
            "MEAM".to_string(),
            "EAS".to_string(),
            "CIT".to_string(),
            "ENM".to_string(),
            "IPD".to_string(),
            "MATH".to_string(),
        ]),
        cu: None,
        level: Some(5000),
        attr: None,
        excluding: Some(vec![
            "EAS 8950".to_string(),
            "EAS 8960".to_string(),
            "EAS 8970".to_string(),
        ]),
        number: 1,
        no_school: None,
    }
}

fn ms_cis_or_non_cis_elective() -> Requirement {
    Requirement::AnyOf {
        category: Some("CIS or Non-CIS Electives".to_string()),
        possibilities: vec![
            Requirement::Restriction {
                category: Some("CIS Elective".to_string()),
                department: Some(vec!["CIS".to_string()]),
                cu: None,
                level: Some(5000),
                attr: None,
                excluding: None,
                number: 1,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("Non-CIS Elective".to_string()),
                department: None,
                cu: None,
                level: Some(5000),
                attr: Some(vec!["EMCI".to_string()]),
                excluding: None,
                number: 1,
                no_school: None,
            },
        ],
    }
}

fn ms_cis_schedule_hints() -> HashMap<String, (i32, String)> {
    let mut hints = schedule_hints_from_array(&[
        Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S,
    ]);
    for (course, sem) in [
        ("CIS 5050", Y1F),
        ("CIS 5020", Y1S),
        ("CIS 5200", Y2F),
        ("CIS 5000", Y2S),
    ] {
        hints.insert(course.to_string(), sem.to_pair());
    }
    hints
}

pub fn create_ms_robo_major() -> Major {
    Major {
        short_name: "MS_ROBO".to_string(),
        name: "Robotics, MSE".to_string(),
        requirements: vec![
            ms_robo_foundational_courses(),
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },

            ms_grad_general_elective(),
            ms_grad_general_elective(),
            ms_grad_general_elective(),
        ],
        schedule_hints: HashMap::new(),
        concentrations: None,
    }
}

pub fn create_ms_meam_major() -> Major {
    // TODO: populate MS Mechanical Engineering and Applied Mechanics requirements
    placeholder_ms_major("MS_MEAM", "Mechanical Engineering and Applied Mechanics, MSE")
}

pub fn create_ms_cis_major() -> Major {
    Major {
        short_name: "MS_CIS".to_string(),
        name: "Computer Science, MSE".to_string(),
        requirements: vec![
            Requirement::SingleCourse { category: Some("Core Courses".to_string()), possibilities: vec!["CIS 5050".to_string(), "CIS 5480".to_string(), "CIS 5530".to_string(), "CIS 5550".to_string(), "CIS 5010".to_string()] },
            Requirement::SingleCourse { category: Some("Core Courses".to_string()), possibilities: vec!["CIS 5020".to_string(), "CIS 5110".to_string()] },
            Requirement::SingleCourse { category: Some("Core Courses".to_string()), possibilities: vec!["CIS 5200".to_string(), "CIS 5190".to_string(), "CIS 5210".to_string()] },
            Requirement::SingleCourse { category: Some("Core Courses".to_string()), possibilities: vec!["CIS 5050".to_string(), "CIS 5480".to_string(), "CIS 5530".to_string(), "CIS 5550".to_string(), "CIS 5020".to_string(), "CIS 5110".to_string(), "CIS 5000".to_string(), "CIS 5710".to_string()] },
            
            Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: Some(5000), attr: None, excluding: None, number: 3, no_school: None },

            ms_cis_or_non_cis_elective(),
            ms_cis_or_non_cis_elective(),
            ms_cis_or_non_cis_elective(),
        ],
        schedule_hints: ms_cis_schedule_hints(),
        concentrations: None,
    }
}

pub fn create_ms_mse_major() -> Major {
    // TODO: populate MS Materials Science and Engineering requirements
    placeholder_ms_major("MS_MSE", "Materials Science and Engineering, MSE")
}

const MCIT_REQUIRED_COURSES: &[&str] = &[
    "CIT 5910",
    "CIT 5920",
    "CIT 5930",
    "CIT 5940",
    "CIT 5950",
    "CIT 5960",
];

fn mcit_schedule_hints() -> HashMap<String, (i32, String)> {
    let mut hints = schedule_hints_from_array(&[Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S]);
    for (course, sem) in [
        ("CIT 5910", Y1F),
        ("CIT 5920", Y1F),
        ("CIT 5930", Y1F),
        ("CIT 5940", Y1S),
        ("CIT 5950", Y1S),
        ("CIT 5960", Y1S),
    ] {
        hints.insert(course.to_string(), sem.to_pair());
    }
    hints
}

pub fn create_mcit_major() -> Major {
    Major {
        short_name: "MCIT".to_string(),
        name: "Computer & Information Technology, MCIT".to_string(),
        requirements: vec![
            Requirement::SingleCourse {
                category: Some("Required Courses".to_string()),
                possibilities: vec![MCIT_REQUIRED_COURSES[0].to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Required Courses".to_string()),
                possibilities: vec![MCIT_REQUIRED_COURSES[1].to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Required Courses".to_string()),
                possibilities: vec![MCIT_REQUIRED_COURSES[2].to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Required Courses".to_string()),
                possibilities: vec![MCIT_REQUIRED_COURSES[3].to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Required Courses".to_string()),
                possibilities: vec![MCIT_REQUIRED_COURSES[4].to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Required Courses".to_string()),
                possibilities: vec![MCIT_REQUIRED_COURSES[5].to_string()],
            },
            Requirement::Restriction {
                category: Some("Electives".to_string()),
                department: Some(vec!["CIS".to_string()]),
                cu: None,
                level: Some(5000),
                attr: None,
                excluding: None,
                number: 3,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("Free Elective".to_string()),
                department: None,
                cu: None,
                level: None,
                attr: None,
                excluding: None,
                number: 1,
                no_school: None,
            },
        ],
        schedule_hints: mcit_schedule_hints(),
        concentrations: None,
    }
}

