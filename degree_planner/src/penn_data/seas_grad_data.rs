use std::collections::{BTreeMap, HashMap};

use crate::Major;
use crate::Requirement;
use crate::schedule_template::{schedule_hints_from_array, ScheduleHint, Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S};

fn placeholder_ms_major(short_name: &str, display_name: &str) -> Major {
    Major {
        short_name: short_name.to_string(),
        name: display_name.to_string(),
        requirements: vec![Requirement::Restriction {
            category: Some("Program Requirements (placeholder)".to_string()),
            department: None,
            cu: None,
            level: None,
            max_level: None,
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
                max_level: None,
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
                max_level: None,
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
                max_level: None,
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
        max_level: None,
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
                max_level: None,
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
                max_level: None,
                attr: Some(vec!["EMCI".to_string()]),
                excluding: None,
                number: 1,
                no_school: None,
            },
        ],
    }
}

fn ms_cis_schedule_hints() -> HashMap<String, ScheduleHint> {
    let mut hints = schedule_hints_from_array(&[
        Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S,
    ]);
    for (course, sem) in [
        ("CIS 5050", Y1F),
        ("CIS 5020", Y1S),
        ("CIS 5200", Y2F),
        ("CIS 5000", Y2S),
    ] {
        hints.insert(course.to_string(), sem.into());
    }
    hints
}

pub fn create_ms_robo_major() -> Major {
    Major {
        short_name: "MS_ROBO".to_string(),
        name: "Robotics, MSE".to_string(),
        requirements: vec![
            ms_robo_foundational_courses(),
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },

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
            
            Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: Some(5000), max_level: None, attr: None, excluding: None, number: 3, no_school: None },

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

const MS_BE_SEAS_DEPTS: &[&str] = &[
    "BE", "CBE", "CIS", "CIT", "EAS", "ENM", "ESE", "IPD", "MEAM", "MSE",
];

fn ms_be_math_course() -> Requirement {
    Requirement::AnyOf {
        category: Some("Math".to_string()),
        possibilities: vec![
            Requirement::Restriction {
                category: None,
                department: None,
                cu: None,
                level: Some(5000),
                max_level: None,
                attr: Some(vec!["EMBM".to_string()]),
                excluding: None,
                number: 1,
                no_school: None,
            },
            Requirement::Restriction {
                category: None,
                department: None,
                cu: None,
                level: Some(5000),
                max_level: None,
                attr: Some(vec!["EPBM".to_string()]),
                excluding: None,
                number: 1,
                no_school: None,
            },
        ],
    }
}

fn ms_be_bio_science_course() -> Requirement {
    Requirement::AnyOf {
        category: Some("Biological Science".to_string()),
        possibilities: vec![
            Requirement::Restriction {
                category: None,
                department: None,
                cu: None,
                level: Some(5000),
                max_level: None,
                attr: Some(vec!["EMBS".to_string()]),
                excluding: None,
                number: 1,
                no_school: None,
            },
            Requirement::Restriction {
                category: None,
                department: None,
                cu: None,
                level: Some(5000),
                max_level: None,
                attr: Some(vec!["EPBS".to_string()]),
                excluding: None,
                number: 1,
                no_school: None,
            },
        ],
    }
}

fn ms_be_grad_course() -> Requirement {
    Requirement::Restriction {
        category: Some("Bioengineering Graduate Course".to_string()),
        department: Some(vec!["BE".to_string()]),
        cu: None,
        level: Some(5000),
        max_level: None,
        attr: None,
        excluding: None,
        number: 1,
        no_school: None,
    }
}

fn ms_be_seas_grad_restriction(category: Option<&str>) -> Requirement {
    Requirement::Restriction {
        category: category.map(str::to_string),
        department: Some(MS_BE_SEAS_DEPTS.iter().map(|d| (*d).to_string()).collect()),
        cu: None,
        level: Some(5000),
        max_level: None,
        attr: None,
        excluding: None,
        number: 1,
        no_school: None,
    }
}

fn ms_be_seas_or_bio_elective(category: &str) -> Requirement {
    Requirement::AnyOf {
        category: Some(category.to_string()),
        possibilities: vec![
            ms_be_seas_grad_restriction(None),
            ms_be_bio_science_course(),
        ],
    }
}

fn ms_be_non_thesis_elective() -> Requirement {
    Requirement::AnyOf {
        category: Some("SEAS and/or Biological Science Elective".to_string()),
        possibilities: vec![
            ms_be_seas_grad_restriction(None),
            ms_be_bio_science_course(),
            Requirement::SingleCourse {
                category: None,
                possibilities: vec!["BE 5990".to_string()],
            },
        ],
    }
}

fn ms_be_general_elective() -> Requirement {
    Requirement::Restriction {
        category: Some("General Elective".to_string()),
        department: None,
        cu: None,
        level: Some(5000),
        max_level: None,
        attr: None,
        excluding: None,
        number: 1,
        no_school: None,
    }
}

fn ms_be_thesis_or_non_thesis() -> Requirement {
    Requirement::AnyOf {
        category: Some("Thesis or Non-Thesis".to_string()),
        possibilities: vec![
            Requirement::AllOf {
                category: None,
                requirements: vec![
                    Requirement::SingleCourse {
                        category: Some("Master's Thesis".to_string()),
                        possibilities: vec!["BE 9990".to_string()],
                    },
                    Requirement::SingleCourse {
                        category: Some("Master's Thesis".to_string()),
                        possibilities: vec!["BE 9990".to_string()],
                    },
                ],
            },
            Requirement::AllOf {
                category: None,
                requirements: vec![
                    ms_be_non_thesis_elective(),
                    ms_be_non_thesis_elective(),
                ],
            },
        ],
    }
}

fn ms_be_conc_slot(category: &str, courses: &[&str]) -> Requirement {
    Requirement::SingleCourse {
        category: Some(category.to_string()),
        possibilities: courses.iter().map(|code| (*code).to_string()).collect(),
    }
}

fn ms_be_concentration(name: &str, courses: &[&str]) -> (String, Vec<Requirement>) {
    let category = name.to_string();
    (
        category.clone(),
        (0..4)
            .map(|_| ms_be_conc_slot(&category, courses))
            .collect(),
    )
}

fn ms_be_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    BTreeMap::from([
        ms_be_concentration(
            "Biomedical Data Science and Computational Medicine",
            &[
                "BE 9990", "BE 5990", "BE 5040", "BE 5060", "BE 5210", "BE 5300", "BE 5320",
                "BE 5370", "BE 5400", "BE 5570", "BE 5590", "BE 5660", "BE 5740", "BBCB 6340",
                "BIOL 5262", "BIOL 5510", "BIOL 5860", "BMIN 5010", "BMIN 5030", "BMIN 5200",
                "BMIN 5220", "CBE 5250", "CIS 5190", "CIS 5200", "CIS 5210", "CIS 5350",
                "BIOM 5350", "MTR 5350", "CIS 5450", "CIT 5900", "CIS 7000", "ENM 5320",
                "ESE 5420", "GCB 5330", "BMIN 5330", "IMUN 5770", "GCB 5360", "BIOL 5536",
                "CIS 5360", "GCB 5370", "STAT 5000", "STAT 5010", "STAT 5030",
            ],
        ),
        ms_be_concentration(
            "Biomedical Devices",
            &[
                "BE 9990", "BE 5990", "BE 5020", "BE 5060", "BE 5130", "BE 5140", "BE 5180",
                "BE 5210", "BE 5280", "BE 5290", "BE 5510", "BE 5560", "BE 5700", "BE 5850",
                "BE 6080", "ESE 5050", "MEAM 5130", "ESE 5290", "ESE 5360", "HCMG 8530",
                "IPD 5150", "IPD 5190", "MEAM 5100", "MEAM 5140", "MEAM 5200", "MEAM 5750",
                "MSE 5050", "MEAM 5050",
            ],
        ),
        ms_be_concentration(
            "Cellular/Tissue Engineering and Biomaterials",
            &[
                "BE 9990", "BE 5990", "BE 5120", "BE 5400", "BE 5530", "BE 5580", "BE 5590",
                "BE 5650", "BE 5690", "BE 5780", "BE 5850", "MSE 5850", "CBE 5570", "MEAM 5140",
                "MEAM 5180",
            ],
        ),
        ms_be_concentration(
            "Biomedical Imaging and Radiation Physics",
            &[
                "BE 9990", "BE 5990", "BE 5180", "BE 5370", "BE 5470", "BE 5810", "BE 5830",
                "BE 5840", "BE 6500", "BBCB 6010", "MPHY 6030", "MPHY 6070", "PHYS 5529",
            ],
        ),
        ms_be_concentration(
            "Systems and Synthetic Biology",
            &[
                "BE 9990", "BE 5990", "BE 5270", "BE 5400", "BE 5440", "BE 5580", "BE 5590",
                "BE 5650", "BE 5690", "BIOL 5262", "CBE 5170", "CBE 5270", "CBE 5520", "CBE 5540",
                "CBE 5570", "MEAM 6630",
            ],
        ),
        ms_be_concentration(
            "Neuroengineering",
            &[
                "BE 9990", "BE 5990", "BE 5060", "BE 5210", "BE 5300", "BE 5660", "ESE 5660",
                "BE 5850", "BE 5950", "BE 6100", "NGG 5720", "NGG 5730", "PSYC 5470", "PSYC 5490",
            ],
        ),
        ms_be_concentration(
            "Multiscale Biomechanics",
            &[
                "BE 9990", "BE 5990", "BE 5100", "BE 5140", "BE 5500", "BE 5610", "BE 5700",
                "MSE 6500",
            ],
        ),
        ms_be_concentration(
            "Therapeutics, Drug Delivery and Nanomedicine",
            &[
                "BE 9990", "BE 5990", "BE 5020", "BE 5260", "BE 5270", "BE 5550", "CBE 5550",
                "BE 5570", "BE 5620", "CBE 5620", "BE 5780", "BE 6080", "CAMB 6090", "IMUN 6090",
                "CBE 5540", "CBE 5570", "CBE 5640",
            ],
        ),
        ms_be_concentration(
            "Immune Engineering",
            &[
                "BE 9990", "BE 5990", "BE 5120", "BE 5260", "BE 5270", "BE 5550", "CBE 5550",
                "BE 5570", "BE 5620", "CBE 5620", "BBCB 5850", "CAMB 6330", "CAMB 7070",
                "MTR 6210", "CBE 5640", "IMUN 5060", "IMUN 5070", "IMUN 6090", "CAMB 6090",
                "REG 6180",
            ],
        ),
    ])
}

fn ms_be_schedule_hints() -> HashMap<String, ScheduleHint> {
    let mut hints = schedule_hints_from_array(&[
        Y1F, Y1F, Y1F, Y1S, Y1S, Y1S, Y2F, Y2F, Y2S,
    ]);
    hints.insert("BE 9990".to_string(), Y2F.into());
    hints
}

pub fn ms_be_concentration_names() -> Vec<String> {
    ms_be_concentrations().keys().cloned().collect()
}

pub fn create_ms_be_major() -> Major {
    Major {
        short_name: "MS_BE".to_string(),
        name: "Bioengineering, MSE".to_string(),
        requirements: vec![
            ms_be_math_course(),
            ms_be_math_course(),
            ms_be_bio_science_course(),
            ms_be_bio_science_course(),
            ms_be_grad_course(),
            ms_be_grad_course(),
            ms_be_seas_or_bio_elective("SEAS and/or Biological Science Elective"),
            ms_be_general_elective(),
            ms_be_thesis_or_non_thesis(),
        ],
        schedule_hints: ms_be_schedule_hints(),
        concentrations: Some(ms_be_concentrations()),
    }
}

const MCIT_REQUIRED_COURSES: &[&str] = &[
    "CIT 5910",
    "CIT 5920",
    "CIT 5930",
    "CIT 5940",
    "CIT 5950",
    "CIT 5960",
];

fn mcit_schedule_hints() -> HashMap<String, ScheduleHint> {
    let mut hints = schedule_hints_from_array(&[Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S]);
    for (course, sem) in [
        ("CIT 5910", Y1F),
        ("CIT 5920", Y1F),
        ("CIT 5930", Y1F),
        ("CIT 5940", Y1S),
        ("CIT 5950", Y1S),
        ("CIT 5960", Y1S),
    ] {
        hints.insert(course.to_string(), sem.into());
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
                max_level: None,
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
                max_level: None,
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

