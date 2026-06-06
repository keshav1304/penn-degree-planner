use std::collections::{BTreeMap, HashMap};
use crate::Major;
use crate::Requirement;

// ── Path@Penn attribute codes ────────────────────────────────────────────────

pub const FA_CROSS_CULTURAL: &str = "AUCC";
pub const FA_QUANT_DATA: &str = "AUQD";
pub const FA_CULTURAL_DIVERSITY: &str = "AUCD";
pub const FA_FORMAL_REASONING: &str = "AUFR";
pub const FA_LANGUAGE: &str = "AULL";

pub const SECTOR_SOCIETY: &str = "AUSO";
pub const SECTOR_HISTORY: &str = "AUHT";
pub const SECTOR_ARTS_LETTERS: &str = "AUAL";
pub const SECTOR_HUM_SOC_SCI: &str = "AUHS";
pub const SECTOR_LIVING_WORLD: &str = "AULW";
pub const SECTOR_PHYSICAL_WORLD: &str = "AUPW";
pub const SECTOR_NAT_SCI: &str = "AUNM";

/// Total degree credit units (writing included).
pub const CAS_DEGREE_CU: i32 = 32;

const FOUNDATIONAL_APPROACHES: &[(&str, &str)] = &[
    ("Cross-Cultural Analysis", FA_CROSS_CULTURAL),
    ("Quantitative Data Analysis", FA_QUANT_DATA),
    ("Cultural Diversity in the U.S.", FA_CULTURAL_DIVERSITY),
    ("Formal Reasoning & Analysis", FA_FORMAL_REASONING),
    ("Language (last course)", FA_LANGUAGE),
];

const SECTORS: &[(&str, &str)] = &[
    ("I — Society", SECTOR_SOCIETY),
    ("II — History and Tradition", SECTOR_HISTORY),
    ("III — Arts and Letters", SECTOR_ARTS_LETTERS),
    ("IV — Humanities & Social Sciences", SECTOR_HUM_SOC_SCI),
    ("V — Living World", SECTOR_LIVING_WORLD),
    ("VI — Physical World", SECTOR_PHYSICAL_WORLD),
    ("VII — Natural Sciences Across Disciplines", SECTOR_NAT_SCI),
];

/// College-wide gen-ed configuration shared by every CAS major.
pub struct CasMajorConfig {
    pub short_name: String,
    pub name: String,
    /// Major-specific courses and electives (the portion that varies by major).
    pub major_requirements: Vec<Requirement>,
    /// Sector attribute codes auto-completed on major declaration (e.g. `AUHS`, or
    /// `AULW` + `AUPW` for biology-style majors). Remaining sectors are required.
    pub auto_completed_sectors: Vec<String>,
    pub concentrations: Option<BTreeMap<String, Vec<Requirement>>>,
}

/// Writing is siloed: it may not double-count with any other College requirement.
fn cas_writing_requirement() -> Requirement {
    Requirement::Restriction {
        category: Some("Foundational Approaches — Writing".to_string()),
        department: Some(vec!["WRIT".to_string()]),
        cu: None,
        level: None,
        attr: None,
        excluding: None,
        number: 1,
        no_school: None,
    }
}

fn cas_foundational_approach(label: &str, attr: &str) -> Requirement {
    Requirement::Restriction {
        category: Some(format!("Foundational Approaches — {label}")),
        department: None,
        cu: None,
        level: None,
        attr: Some(vec![attr.to_string()]),
        excluding: None,
        number: 1,
        no_school: None,
    }
}

fn cas_sector_requirement(label: &str, attr: &str) -> Requirement {
    Requirement::Restriction {
        category: Some(format!("Sectors of Knowledge — {label}")),
        department: None,
        cu: None,
        level: None,
        attr: Some(vec![attr.to_string()]),
        excluding: None,
        number: 1,
        no_school: None,
    }
}

/// Double-count overlay slots: FAs + non-auto-completed Sectors.
pub fn cas_double_counting_requirements(auto_completed_sectors: &[String]) -> Vec<Requirement> {
    let mut requirements = Vec::new();

    for (label, attr) in FOUNDATIONAL_APPROACHES {
        requirements.push(cas_foundational_approach(label, attr));
    }

    for (label, attr) in SECTORS {
        if !auto_completed_sectors.iter().any(|s| s == attr) {
            requirements.push(cas_sector_requirement(label, attr));
        }
    }

    requirements
}

/// Approximate CU represented by a requirement subtree (whole-CU slots).
fn requirement_slot_cu(req: &Requirement) -> i32 {
    match req {
        Requirement::SingleCourse { .. } | Requirement::AnyOf { .. } => 1,
        Requirement::AllOf { requirements, .. } | Requirement::Concentration { requirements, .. } => {
            requirements.iter().map(requirement_slot_cu).sum()
        }
        Requirement::CourseGroup { number, .. } => *number,
        Requirement::Restriction { number, cu, .. } => {
            if cu.is_some() {
                1
            } else {
                *number
            }
        }
        Requirement::DoubleCount {
            base_requirements, ..
        } => base_requirements.iter().map(requirement_slot_cu).sum(),
    }
}

/// Fill remaining non-writing CU with a single unrestricted-elective bucket.
fn cas_unrestricted_electives(major_reqs: &[Requirement]) -> Vec<Requirement> {
    let major_cu: i32 = major_reqs.iter().map(requirement_slot_cu).sum();
    let base_cu = CAS_DEGREE_CU - 1;
    let remaining = (base_cu - major_cu).max(0);
    if remaining == 0 {
        return vec![];
    }
    vec![Requirement::Restriction {
        category: Some("Unrestricted Electives".to_string()),
        department: None,
        cu: None,
        level: None,
        attr: None,
        excluding: None,
        number: remaining,
        no_school: None,
    }]
}

/// Assemble a full CAS degree using one DoubleCount block for gen-ed overlays.
pub fn create_cas_major(config: CasMajorConfig) -> Major {
    let mut base_requirements = config.major_requirements.clone();
    base_requirements.extend(cas_unrestricted_electives(&config.major_requirements));

    let requirements = vec![
        cas_writing_requirement(),
        Requirement::DoubleCount {
            category: Some("College of Arts and Sciences".to_string()),
            base_requirements,
            double_counting_requirements: cas_double_counting_requirements(
                &config.auto_completed_sectors,
            ),
        },
    ];

    Major {
        short_name: config.short_name,
        name: config.name,
        requirements,
        schedule_hints: HashMap::new(),
        concentrations: config.concentrations,
    }
}

// ── Major-specific requirement blocks ────────────────────────────────────────
// Add `create_XX_major()` functions below. Each should define only major courses,
// then call `create_cas_major` with the appropriate auto-completed sector(s).

fn econ_major_requirements() -> Vec<Requirement> {
    vec![
        Requirement::SingleCourse {
            category: Some("Introductory Econ".to_string()),
            possibilities: vec!["ECON 0100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Introductory Econ".to_string()),
            possibilities: vec!["ECON 0200".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Econ".to_string()),
            possibilities: vec!["ECON 2100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Econ".to_string()),
            possibilities: vec!["ECON 2200".to_string()],
        },
        Requirement::AnyOf {
            category: Some("Statistics".to_string()),
            possibilities: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["ECON 2300".to_string()],
                },
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["STAT 4300".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["STAT 4310".to_string()],
                        },
                    ],
                },
            ],
        },
        Requirement::SingleCourse {
            category: Some("Econometrics".to_string()),
            possibilities: vec!["ECON 2310".to_string()],
        },
        Requirement::Restriction {
            category: Some("ECON Electives".to_string()),
            department: Some(vec!["ECON".to_string()]),
            cu: None,
            level: Some(4000),
            attr: None,
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::Restriction {
            category: Some("ECON Electives".to_string()),
            department: Some(vec!["ECON".to_string()]),
            cu: None,
            level: Some(4000),
            attr: None,
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::Restriction {
            category: Some("ECON Electives".to_string()),
            department: Some(vec!["ECON".to_string()]),
            cu: None,
            level: Some(4000),
            attr: None,
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::Restriction {
            category: Some("ECON Electives".to_string()),
            department: Some(vec!["ECON".to_string()]),
            cu: None,
            level: Some(4000),
            attr: None,
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::AnyOf {
            category: Some("Econ — Mathematics".to_string()),
            possibilities: vec![
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["MATH 1070".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["MATH 1080".to_string()],
                        },
                    ],
                },
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["MATH 1400".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec![
                                "MATH 1410".to_string(),
                                "MATH 1610".to_string(),
                            ],
                        },
                    ],
                },
            ],
        },
    ]
}

pub fn create_econ_major() -> Major {
    create_cas_major(CasMajorConfig {
        short_name: "ECON".to_string(),
        name: "Economics".to_string(),
        major_requirements: econ_major_requirements(),
        auto_completed_sectors: vec![SECTOR_SOCIETY.to_string()],
        concentrations: None,
    })
}

fn mathecon_major_requirements() -> Vec<Requirement> {
    vec![
        Requirement::SingleCourse {
            category: Some("Introductory Econ".to_string()),
            possibilities: vec!["ECON 0100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Introductory Econ".to_string()),
            possibilities: vec!["ECON 0200".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Econ".to_string()),
            possibilities: vec!["ECON 2100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Econ".to_string()),
            possibilities: vec!["ECON 2200".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Econ".to_string()),
            possibilities: vec!["ECON 6100".to_string()],
        },

        
        Requirement::AnyOf {
            category: Some("Stat Core".to_string()),
            possibilities: vec![
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["ECON 2300".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["MATH 5460".to_string(), "ECON 2310".to_string(), "ECON 4310".to_string(), "ECON 4320".to_string()],
                        },
                    ],
                },
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["STAT 4300".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["STAT 4310".to_string()],
                        },
                    ],
                },
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["ESE 3010".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["ESE 4020".to_string()],
                        },
                    ],
                },
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["STAT 4300".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["ESE 2310".to_string()],
                        },
                    ],
                },
            ],
        },


        Requirement::Restriction {
            category: Some("ECON Electives".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["AMAE".to_string()]),
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::Restriction {
            category: Some("ECON Electives".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["AMAE".to_string()]),
            excluding: None,
            number: 1,
            no_school: None,
        },

        Requirement::SingleCourse {
            category: Some("Math Core".to_string()),
            possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string(), "MATH 1080".to_string()],
        },

        Requirement::SingleCourse {
            category: Some("Math Core".to_string()),
            possibilities: vec!["MATH 3000".to_string()],
        },

        Requirement::AnyOf {
            category: Some("Math Core".to_string()),
            possibilities: vec![
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["MATH 3600".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["MATH 3610".to_string()],
                        },
                    ],
                },
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["MATH 5080".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["MATH 5090".to_string()],
                        },
                    ],
                },
            ],
        },

        Requirement::Restriction { category: Some("Math Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["AMAM".to_string()]), excluding: None, number: 1, no_school: None },
        Requirement::Restriction { category: Some("Math Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["AMAM".to_string()]), excluding: None, number: 1, no_school: None },
        Requirement::Restriction { category: Some("Math Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["AMAM".to_string()]), excluding: None, number: 1, no_school: None },
    ]
}

pub fn create_mathecon_major() -> Major {
    create_cas_major(CasMajorConfig {
        short_name: "MECON".to_string(),
        name: "Mathematical Economics".to_string(),
        major_requirements: mathecon_major_requirements(),
        auto_completed_sectors: vec![SECTOR_SOCIETY.to_string()],
        concentrations: None,
    })
}
