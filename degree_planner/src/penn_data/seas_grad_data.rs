use std::collections::{BTreeMap, HashMap};

use crate::Major;
use crate::Requirement;
use crate::penn_data::requirement_builders::{
    any_of, course_group, repeat_req, required_slots, restriction,
    schedule_hints, single,
};
use crate::schedule_template::{Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S};

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
    let ee_core = single("Electrical Engineering Core", MS_EE_CORE_COURSES);
    let mut requirements = repeat_req(&ee_core, 5);
    requirements.extend([
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
    ]);
    Major {
        short_name: "MS_EE".to_string(),
        name: "Electrical Engineering, MSE".to_string(),
        requirements,
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

/// Fall 2026+ curriculum: 8 shared CU + 2 CU thesis or non-thesis track.
pub fn ms_be_concentration_names() -> Vec<String> {
    vec!["Thesis".to_string(), "Non-thesis".to_string()]
}

pub fn create_ms_be_major(concentration_name: String) -> Major {
    // Helper for Biological Science slot
    let ms_be_bio_science_slot = || {
        restriction(1)
            .category("Biological Science")
            .level(5000)
            .attr(&["EMBS", "EPBS"])
            .into()
    };

    // Helper for SEAS or Biological Science slot
    let ms_be_seas_or_bio_slot = |category: &str| {
        any_of(
            category,
            vec![
                restriction(1)
                    .departments(MS_BE_SEAS_DEPTS)
                    .level(5000)
                    .into(),
                ms_be_bio_science_slot(),
            ],
        )
    };

    // Helpers for shared & elective requirements
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

    // Build base requirements (8 CU)
    let mut requirements = [
        repeat_req(&math, 2),
        repeat_req(&ms_be_bio_science_slot(), 2),
        repeat_req(&be_elective, 2),
        vec![
            ms_be_seas_or_bio_slot("SEAS and/or Biological Science Elective"),
            restriction(1)
                .category("General Elective")
                .level(5000)
                .into(),
        ],
    ]
    .concat();

    // Determine track requirements (2 CU)
    let (track, concentrations): (Vec<Requirement>, Option<BTreeMap<String, Vec<Requirement>>>) =
        match concentration_name.as_str() {
            "Non-thesis" => {
                let non_thesis = repeat_req(
                    &ms_be_seas_or_bio_slot("SEAS and/or Biological Science Elective"),
                    2,
                );
                (
                    non_thesis.clone(),
                    Some(BTreeMap::from([
                        (
                            "Thesis".to_string(),
                            vec![
                                single("Master's Thesis", &["BE 9990"]),
                                single("Master's Thesis", &["BE 9990"]),
                            ],
                        ),
                        ("Non-thesis".to_string(), non_thesis),
                    ])),
                )
            }
            _ => {
                let thesis = vec![
                    single("Master's Thesis", &["BE 9990"]),
                    single("Master's Thesis", &["BE 9990"]),
                ];
                (
                    thesis.clone(),
                    Some(BTreeMap::from([
                        (
                            "Thesis".to_string(),
                            thesis,
                        ),
                        (
                            "Non-thesis".to_string(),
                            repeat_req(
                                &ms_be_seas_or_bio_slot("SEAS and/or Biological Science Elective"),
                                2,
                            ),
                        ),
                    ])),
                )
            }
        };

    requirements.extend(track);

    Major {
        short_name: "MS_BE".to_string(),
        name: "Bioengineering, MSE".to_string(),
        requirements,
        schedule_hints: schedule_hints(
            &[Y1F, Y1F, Y1F, Y1S, Y1S, Y1S, Y1S, Y2F, Y2F, Y2S],
            &[("BE 9990", Y2F)],
        ),
        concentrations,
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
