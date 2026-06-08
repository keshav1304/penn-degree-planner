use std::collections::{BTreeMap, HashMap};
use crate::Major;
use crate::Requirement;
use crate::requirement::{PoolConstraint, PoolCoverageInfo};
use crate::schedule_template::{Y1F, Y1S, Y2F, Y2S, Y3F, Y3S};
use serde::Serialize;

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
    ("I - Society", SECTOR_SOCIETY),
    ("II - History and Tradition", SECTOR_HISTORY),
    ("III - Arts and Letters", SECTOR_ARTS_LETTERS),
    ("IV - Humanities & Social Sciences", SECTOR_HUM_SOC_SCI),
    ("V - Living World", SECTOR_LIVING_WORLD),
    ("VI - Physical World", SECTOR_PHYSICAL_WORLD),
    ("VII - Natural Sciences Across Disciplines", SECTOR_NAT_SCI),
];

/// Sector attribute codes auto-completed when a CAS major is declared.
pub fn cas_auto_completed_sectors_for(short_name: &str) -> Vec<String> {
    match short_name {
        "ECON" | "MECON" => vec![SECTOR_SOCIETY.to_string()],
        _ => vec![],
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct CasGenEdRequirementStatus {
    pub name: String,
    pub attr: String,
    pub fulfilled: bool,
    pub fulfilled_by_major: bool,
    pub matched_courses: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CasGenEdInfo {
    pub foundational_approaches: Vec<CasGenEdRequirementStatus>,
    pub sectors: Vec<CasGenEdRequirementStatus>,
}

pub fn build_cas_gen_ed_info(
    pool: &PoolCoverageInfo,
    auto_completed_sectors: &[String],
) -> CasGenEdInfo {
    let constraint_for_attr = |attr: &str| {
        pool.constraints
            .iter()
            .find(|c| c.label == attr)
    };

    let foundational_approaches = FOUNDATIONAL_APPROACHES
        .iter()
        .map(|(name, attr)| {
            if let Some(c) = constraint_for_attr(attr) {
                CasGenEdRequirementStatus {
                    name: name.to_string(),
                    attr: attr.to_string(),
                    fulfilled: c.fulfilled,
                    fulfilled_by_major: false,
                    matched_courses: c.matched_courses.clone(),
                }
            } else {
                CasGenEdRequirementStatus {
                    name: name.to_string(),
                    attr: attr.to_string(),
                    fulfilled: false,
                    fulfilled_by_major: false,
                    matched_courses: vec![],
                }
            }
        })
        .collect();

    let sectors = SECTORS
        .iter()
        .map(|(name, attr)| {
            if auto_completed_sectors.iter().any(|s| s == attr) {
                CasGenEdRequirementStatus {
                    name: name.to_string(),
                    attr: attr.to_string(),
                    fulfilled: true,
                    fulfilled_by_major: true,
                    matched_courses: vec![],
                }
            } else if let Some(c) = constraint_for_attr(attr) {
                CasGenEdRequirementStatus {
                    name: name.to_string(),
                    attr: attr.to_string(),
                    fulfilled: c.fulfilled,
                    fulfilled_by_major: false,
                    matched_courses: c.matched_courses.clone(),
                }
            } else {
                CasGenEdRequirementStatus {
                    name: name.to_string(),
                    attr: attr.to_string(),
                    fulfilled: false,
                    fulfilled_by_major: false,
                    matched_courses: vec![],
                }
            }
        })
        .collect();

    CasGenEdInfo {
        foundational_approaches,
        sectors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cas_gen_ed_info_lists_major_completed_sector() {
        let major = create_econ_major();
        let cu_map = HashMap::from([("WRIT 0100".to_string(), 1.0)]);
        let taken = vec!["WRIT 0100".to_string()];
        let validation =
            crate::requirement::validate_courses_for_degree(major.requirements, &taken, &cu_map);
        let pool = validation
            .pool_coverage_info
            .into_iter()
            .find(|p| p.category == "General Education")
            .expect("gen ed pool");
        let info = build_cas_gen_ed_info(&pool, &cas_auto_completed_sectors_for("ECON"));

        assert_eq!(info.foundational_approaches.len(), 5);
        assert_eq!(info.sectors.len(), 7);
        let society = info
            .sectors
            .iter()
            .find(|s| s.attr == SECTOR_SOCIETY)
            .expect("society sector");
        assert!(society.fulfilled);
        assert!(society.fulfilled_by_major);
    }
}

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
    /// Requirement-index and/or course-code keys → preferred `(year, semester)`.
    pub schedule_hints: HashMap<String, (i32, String)>,
}

/// Writing is siloed: it may not double-count with any other College requirement.
fn cas_writing_requirement() -> Requirement {
    Requirement::Restriction {
        category: Some("Writing Seminar".to_string()),
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
        category: Some(format!("Foundational Approaches - {label}")),
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
        category: Some(format!("Sectors of Knowledge - {label}")),
        department: None,
        cu: None,
        level: None,
        attr: Some(vec![attr.to_string()]),
        excluding: None,
        number: 1,
        no_school: None,
    }
}

/// Coverage constraints: FAs + non-auto-completed Sectors.
pub fn cas_pool_constraints(auto_completed_sectors: &[String]) -> Vec<PoolConstraint> {
    let mut constraints = Vec::new();

    for (label, attr) in FOUNDATIONAL_APPROACHES {
        constraints.push(PoolConstraint {
            requirement: cas_foundational_approach(label, attr),
            count: 1,
            consumption_group: Some("cas:fa".to_string()),
        });
    }

    for (label, attr) in SECTORS {
        if !auto_completed_sectors.iter().any(|s| s == attr) {
            constraints.push(PoolConstraint {
                requirement: cas_sector_requirement(label, attr),
                count: 1,
                consumption_group: Some("cas:sector".to_string()),
            });
        }
    }

    constraints
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
        Requirement::CoursePool {
            fixed_slots,
            flexible_slots,
            ..
        } => fixed_slots.iter().map(requirement_slot_cu).sum::<i32>() + flexible_slots,
    }
}

/// Assemble a full CAS degree using a shared course pool for major + electives + gen-ed coverage.
pub fn create_cas_major(config: CasMajorConfig) -> Major {
    let major_cu: i32 = config.major_requirements.iter().map(requirement_slot_cu).sum();
    let flexible_slots = (CAS_DEGREE_CU - 1 - major_cu).max(0);

    let requirements = vec![
        cas_writing_requirement(),
        Requirement::CoursePool {
            category: Some("General Education".to_string()),
            fixed_slots: config.major_requirements,
            flexible_slots,
            constraints: cas_pool_constraints(&config.auto_completed_sectors),
        },
    ];

    let mut schedule_hints = config.schedule_hints;
    schedule_hints.insert("0".to_string(), Y1F.to_pair());

    Major {
        short_name: config.short_name,
        name: config.name,
        requirements,
        schedule_hints,
        concentrations: config.concentrations,
    }
}
// ── Major-specific requirement blocks ────────────────────────────────────────
// Add `create_XX_major()` functions below. Each should define only major courses,
// then call `create_cas_major` with the appropriate auto-completed sector(s).

fn econ_major_requirements() -> Vec<Requirement> {
    vec![
        Requirement::SingleCourse {
            category: Some("Introductory Economics".to_string()),
            possibilities: vec!["ECON 0100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Introductory Economics".to_string()),
            possibilities: vec!["ECON 0200".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Economics".to_string()),
            possibilities: vec!["ECON 2100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Economics".to_string()),
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
            category: Some("Economics Electives".to_string()),
            department: Some(vec!["ECON".to_string()]),
            cu: None,
            level: Some(4000),
            attr: None,
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::AnyOf {
            category: Some("Mathematics".to_string()),
            possibilities: vec![
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
            ],
        },
    ]
}

pub fn create_econ_major() -> Major {
    let schedule_hints = HashMap::from([
        ("MATH 1070".to_string(), Y1F.to_pair()),
        ("MATH 1080".to_string(), Y1S.to_pair()),
        ("ECON 0100".to_string(), Y1F.to_pair()),
        ("ECON 0200".to_string(), Y1S.to_pair()),
        ("ECON 2100".to_string(), Y2F.to_pair()),
        ("ECON 2200".to_string(), Y2S.to_pair()),
        ("ECON 2300".to_string(), Y2F.to_pair()),
        ("ECON 2310".to_string(), Y2S.to_pair()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "ECON".to_string(),
        name: "Economics".to_string(),
        major_requirements: econ_major_requirements(),
        auto_completed_sectors: vec![SECTOR_SOCIETY.to_string()],
        concentrations: None,
        schedule_hints,
    })
}

fn mathecon_major_requirements() -> Vec<Requirement> {
    vec![
        Requirement::SingleCourse {
            category: Some("Introductory Economics".to_string()),
            possibilities: vec!["ECON 0100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Introductory Economics".to_string()),
            possibilities: vec!["ECON 0200".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Economics".to_string()),
            possibilities: vec!["ECON 2100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Economics".to_string()),
            possibilities: vec!["ECON 2200".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Intermediate Economics".to_string()),
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
    let schedule_hints = HashMap::from([
        ("ECON 0100".to_string(), Y1F.to_pair()),
        ("ECON 0200".to_string(), Y1S.to_pair()),
        ("MATH 1080".to_string(), Y1S.to_pair()),
        ("MATH 1410".to_string(), Y1F.to_pair()),
        ("MATH 1610".to_string(), Y1F.to_pair()),
        ("ECON 2100".to_string(), Y2F.to_pair()),
        ("ECON 2200".to_string(), Y2S.to_pair()),
        ("ECON 2300".to_string(), Y2F.to_pair()),
        ("ECON 2310".to_string(), Y2S.to_pair()),
        ("STAT 4300".to_string(), Y2F.to_pair()),
        ("STAT 4310".to_string(), Y2S.to_pair()),
        ("ESE 3010".to_string(), Y2F.to_pair()),
        ("ESE 4020".to_string(), Y2S.to_pair()),
        ("MATH 3000".to_string(), Y2F.to_pair()),
        ("ECON 6100".to_string(), Y3F.to_pair()),
        ("MATH 3600".to_string(), Y3F.to_pair()),
        ("MATH 3610".to_string(), Y3S.to_pair()),
        ("MATH 5080".to_string(), Y3F.to_pair()),
        ("MATH 5090".to_string(), Y3S.to_pair()),
        ("MATH 5460".to_string(), Y3S.to_pair()),
        ("ESE 2310".to_string(), Y3S.to_pair()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "MECON".to_string(),
        name: "Mathematical Economics".to_string(),
        major_requirements: mathecon_major_requirements(),
        auto_completed_sectors: vec![SECTOR_SOCIETY.to_string()],
        concentrations: None,
        schedule_hints,
    })
}

const CIS_PROJECT_ELECTIVES: &[&str] = &[
    "CIS 3500",
    "CIS 4120",
    "CIS 5120",
    "CIS 4410",
    "CIS 5410",
    "CIS 4480",
    "CIS 5480",
    "CIS 4500",
    "CIS 5500",
    "CIS 4521",
    "CIS 5521",
    "CIS 4550",
    "CIS 5550",
    "CIS 4600",
    "CIS 5600",
    "CIS 4710",
    "CIS 5710",
    "CIS 5050",
    "CIS 5530",
    "ESE 3500",
    "NETS 2120",
];

fn cis_major_requirements() -> Vec<Requirement> {
    let project_electives: Vec<String> = CIS_PROJECT_ELECTIVES
        .iter()
        .map(|code| code.to_string())
        .collect();

    vec![
        Requirement::SingleCourse {
            category: Some("Core Courses".to_string()),
            possibilities: vec!["CIS 1100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Core Courses".to_string()),
            possibilities: vec!["CIS 1200".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Core Courses".to_string()),
            possibilities: vec!["CIS 1600".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Core Courses".to_string()),
            possibilities: vec!["CIS 1210".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Core Courses".to_string()),
            possibilities: vec!["CIS 2400".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Core Courses".to_string()),
            possibilities: vec!["CIS 2620".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Core Courses".to_string()),
            possibilities: vec!["CIS 3200".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Project Electives".to_string()),
            possibilities: project_electives.clone(),
        },
        Requirement::SingleCourse {
            category: Some("Project Electives".to_string()),
            possibilities: project_electives,
        },
        Requirement::AnyOf {
            category: Some("CIS Elective".to_string()),
            possibilities: vec![
                Requirement::Restriction {
                    category: None,
                    department: Some(vec!["CIS".to_string(), "NETS".to_string()]),
                    cu: None,
                    level: None,
                    attr: None,
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["ESE 3500".to_string()],
                },
            ],
        },
        Requirement::AnyOf {
            category: Some("CIS Elective >= 2000".to_string()),
            possibilities: vec![
                Requirement::Restriction {
                    category: None,
                    department: Some(vec!["CIS".to_string(), "NETS".to_string()]),
                    cu: None,
                    level: Some(2000),
                    attr: None,
                    excluding: None,
                    number: 1,
                    no_school: None,
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["ESE 3500".to_string()],
                },
            ],
        },
    ]
}

pub fn create_cis_cas_major() -> Major {
    let schedule_hints = HashMap::from([
        ("CIS 1100".to_string(), Y1F.to_pair()),
        ("CIS 1200".to_string(), Y1S.to_pair()),
        ("CIS 1600".to_string(), Y1F.to_pair()),
        ("CIS 1210".to_string(), Y2F.to_pair()),
        ("CIS 2400".to_string(), Y2S.to_pair()),
        ("CIS 2620".to_string(), Y3F.to_pair()),
        ("CIS 3200".to_string(), Y3S.to_pair()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "CIS".to_string(),
        name: "Computer Science (2nd major only)".to_string(),
        major_requirements: cis_major_requirements(),
        auto_completed_sectors: vec![],
        concentrations: None,
        schedule_hints,
    })
}
