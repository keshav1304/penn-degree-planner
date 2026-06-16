use std::collections::{BTreeMap, HashMap};
use crate::Major;
use crate::Requirement;
use crate::requirement::{PoolConstraint, PoolCoverageInfo};
use crate::schedule_template::{ScheduleHint, Y1F, Y1S, Y2F, Y2S, Y3F, Y3S};
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
pub const CAS_DEGREE_CU: i32 = 36;

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

/// Two or more CAS majors = one College degree with multiple majors (double major).
pub fn is_cas_college_double_major(degree_schools: &[String]) -> bool {
    degree_schools.len() >= 2 && degree_schools.iter().all(|s| s == "CAS")
}

/// Requirement instance scope for college-wide CAS requirements (not major-only).
pub fn is_cas_college_shared_instance_scope(scope: &str) -> bool {
    if scope.is_empty() {
        return false;
    }
    if scope == "0" || scope == "1" {
        return true;
    }
    if let Some(rest) = scope.strip_prefix("1:") {
        return !rest.starts_with('f');
    }
    false
}

/// Schedule slot id for a college-wide CAS requirement (writing / gen-ed pool).
pub fn is_cas_college_shared_schedule_slot(slot_id: &str) -> bool {
    let Some(rest) = slot_id.strip_prefix("req:") else {
        return false;
    };
    let scope = rest.split(":R:").next().unwrap_or(rest);
    is_cas_college_shared_instance_scope(scope)
}

/// Open-slot key eligible for cross-major overlap when double-majoring in CAS.
pub fn is_cas_major_overlap_slot_key(slot_key: &str) -> bool {
    slot_key.starts_with("1:f")
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
    /// Requirement-index and/or course-code keys → scheduling hint.
    pub schedule_hints: HashMap<String, ScheduleHint>,
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
    schedule_hints.insert("0".to_string(), Y1F.into());

    Major {
        short_name: config.short_name,
        name: config.name,
        requirements,
        schedule_hints,
        concentrations: config.concentrations,
    }
}

// ── CAS degree catalog (2025-26) ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct CasMajorCatalogEntry {
    pub api_code: &'static str,
    pub display_name: &'static str,
    pub concentrations: &'static [&'static str],
}

pub const CAS_DEGREE_CATALOG: &[CasMajorCatalogEntry] = &[
    CasMajorCatalogEntry {
        api_code: "AFRC",
        display_name: "Africana Studies",
        concentrations: &[
            "African American Studies",
            "African Diaspora Studies",
            "African Studies",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "ANCH",
        display_name: "Ancient History",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "ANTH",
        display_name: "Anthropology",
        concentrations: &[
            "Archaeology",
            "Biological Anthropology",
            "Cultural and Linguistic Anthropology",
            "Environmental Anthropology",
            "General Anthropology",
            "Medical Anthropology & Global Health",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "ARCH",
        display_name: "Architecture",
        concentrations: &["Design", "Intensive Design"],
    },
    CasMajorCatalogEntry {
        api_code: "BIOC",
        display_name: "Biochemistry",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "BIOL",
        display_name: "Biology",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "BIOP",
        display_name: "Biophysics",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "CHEM",
        display_name: "Chemistry",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "CIMS",
        display_name: "Cinema and Media Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "CLST",
        display_name: "Classical Studies",
        concentrations: &[
            "Classical Civilizations",
            "Classical Languages and Literature",
            "Mediterranean Archaeology",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "COGS",
        display_name: "Cognitive Science",
        concentrations: &[
            "Cognitive Neuroscience",
            "Computation and Cognition",
            "Individualized",
            "Language & Mind",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "COMM",
        display_name: "Communication",
        concentrations: &[
            "Advocacy & Activism",
            "Audiences & Persuasion",
            "Communication & Public Service",
            "Culture & Society",
            "Data & Network Science",
            "General Communication",
            "Politics & Policy",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "COML",
        display_name: "Comparative Literature",
        concentrations: &["(Trans)national Literatures", "Globalization", "Theory"],
    },
    CasMajorCatalogEntry {
        api_code: "CRIM",
        display_name: "Criminology",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "DSGN",
        display_name: "Design",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "EESC",
        display_name: "Earth and Environmental Science",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "EALC",
        display_name: "East Asian Languages and Civilizations",
        concentrations: &[
            "Dual Language",
            "East Asian Area Studies",
            "General East Asian Languages and Civilizations",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "ECON",
        display_name: "Economics",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "ENGL",
        display_name: "English",
        concentrations: &[
            "18th/19th Centuries",
            "20th/21st Centuries",
            "Africana Literatures & Culture",
            "Cinema & Media Studies",
            "Creative Writing",
            "Drama",
            "Gender/Sexuality",
            "General English",
            "Literary Theory & Cultural Studies",
            "Literature, Journalism and Print Culture",
            "Medieval/Renaissance",
            "Poetry and Poetics",
            "The Novel",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "ENVS",
        display_name: "Environmental Studies",
        concentrations: &[
            "Environmental History and Regional Studies",
            "Environmental Policy and Application",
            "General Environmental Studies",
            "Global Environmental Systems",
            "Sustainability and Environmental Management",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "FNAR",
        display_name: "Fine Arts",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "FIGS",
        display_name: "Francophone, Italian and Germanic Studies",
        concentrations: &[
            "Dual Language",
            "French and Francophone Studies",
            "Germanic Studies",
            "Italian Studies",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "GSWS",
        display_name: "Gender, Sexuality, & Women's Studies",
        concentrations: &[
            "Feminist Studies",
            "General",
            "Global Gender and Sexuality Studies",
            "Health and Disability Studies",
            "LGBTQ Studies",
            "Self Designed",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "HSOC",
        display_name: "Health and Societies",
        concentrations: &[
            "Bioethics and Society",
            "Disease and Culture",
            "Global Health",
            "Health Care Markets & Finance",
            "Health Policy & Law",
            "Public Health",
            "Race, Gender and Health",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "HISP",
        display_name: "Hispanic Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "ARTH",
        display_name: "History of Art",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "HIST",
        display_name: "History",
        concentrations: &[
            "American History",
            "Diplomatic History",
            "Economic History",
            "European History",
            "Gender History",
            "General History",
            "Intellectual History",
            "Jewish History",
            "Political History",
            "World History",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "INDM",
        display_name: "Individualized Major",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "INTR",
        display_name: "International Relations",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "INST",
        display_name: "International Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "JWST",
        display_name: "Jewish Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "LALS",
        display_name: "Latin American & Latinx Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "LAWS",
        display_name: "Law and Society",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "LING",
        display_name: "Linguistics",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "LGIC",
        display_name: "Logic, Information, & Computation",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "MECON",
        display_name: "Mathematical Economics",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "MATH",
        display_name: "Mathematics",
        concentrations: &["Biological Mathematics", "General Mathematics"],
    },
    CasMajorCatalogEntry {
        api_code: "MELC",
        display_name: "Middle Eastern Languages & Cultures",
        concentrations: &[
            "Ancient Middle East",
            "Arabic & Hebrew Studies",
            "Arabic & Islamic Studies",
            "Cultures and Societies of the Middle East and North Africa",
            "Hebrew & Judaica Studies",
            "Persian Languages & Literature",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "MMES",
        display_name: "Modern Middle Eastern Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "MUSC",
        display_name: "Music",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "NEUR",
        display_name: "Neuroscience",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "NUTR",
        display_name: "Nutrition Science",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "PHIL",
        display_name: "Philosophy",
        concentrations: &[
            "General Philosophy",
            "Humanistic Philosophy",
            "Philosophy and Science",
            "Political and Moral Philosophy",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "PPE",
        display_name: "Philosophy, Politics and Economics",
        concentrations: &[
            "Choice and Behaviour",
            "Distributive Justice",
            "Globalization",
            "Public Policy and Governance",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "PHYS",
        display_name: "Physics",
        concentrations: &[
            "Astrophysics",
            "Biological Science",
            "Business & Technology",
            "Chemical Principles",
            "Computer Techniques",
            "Physical Theory and Experimental Technique",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "PSCI",
        display_name: "Political Science",
        concentrations: &[
            "American Politics",
            "Comparative Politics",
            "General Political Science",
            "Individualized",
            "International Relations",
            "Political Economy",
            "Political Theory",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "PSYC",
        display_name: "Psychology",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "RELS",
        display_name: "Religious Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "REES",
        display_name: "Russian and East European Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "STSC",
        display_name: "Science, Technology and Society",
        concentrations: &[
            "Biotechnology & Biomedicine",
            "Energy and Environment",
            "Global Science and Technology",
            "Information and Organizations",
            "Science/Nature/Culture",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "SOCI",
        display_name: "Sociology",
        concentrations: &[
            "Applied Research and Data Analysis",
            "Cities, Markets, and the Global Economy",
            "Culture and Diversity",
            "Education and Society",
            "Family, Gender and Society",
            "Medical Sociology",
            "Structures of Opportunity and Inequality",
        ],
    },
    CasMajorCatalogEntry {
        api_code: "SAST",
        display_name: "South Asia Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "THAR",
        display_name: "Theatre Arts",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "URBS",
        display_name: "Urban Studies",
        concentrations: &[],
    },
    CasMajorCatalogEntry {
        api_code: "VIST",
        display_name: "Visual Studies",
        concentrations: &[
            "Architecture Practice and Technology",
            "Art and Culture of Seeing",
            "Art, Practice and Technology",
            "Philosophy and Science of Seeing",
        ],
    },
    // Not listed on the catalog majors page; Penn allows CIS as a 2nd major in CAS.
    CasMajorCatalogEntry {
        api_code: "CIS",
        display_name: "Computer Science (2nd major only)",
        concentrations: &[],
    },
];

pub fn cas_catalog_entry(api_code: &str) -> Option<&'static CasMajorCatalogEntry> {
    CAS_DEGREE_CATALOG
        .iter()
        .find(|entry| entry.api_code == api_code)
}

pub fn cas_concentration_names(api_code: &str) -> Vec<String> {
    cas_catalog_entry(api_code)
        .map(|entry| entry.concentrations.iter().map(|s| (*s).to_string()).collect())
        .unwrap_or_default()
}

fn placeholder_concentration_map(names: &[&str]) -> BTreeMap<String, Vec<Requirement>> {
    names
        .iter()
        .map(|name| (name.to_string(), Vec::new()))
        .collect()
}

/// Gen-ed-only placeholder until major requirements are authored.
pub fn create_cas_placeholder_major(entry: &CasMajorCatalogEntry) -> Major {
    let concentrations = if entry.concentrations.is_empty() {
        None
    } else {
        Some(placeholder_concentration_map(entry.concentrations))
    };
    create_cas_major(CasMajorConfig {
        short_name: entry.api_code.to_string(),
        name: entry.display_name.to_string(),
        major_requirements: vec![],
        auto_completed_sectors: vec![],
        concentrations,
        schedule_hints: HashMap::new(),
    })
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
        ("MATH 1070".to_string(), Y1F.into()),
        ("MATH 1080".to_string(), Y1S.into()),
        ("ECON 0100".to_string(), Y1F.into()),
        ("ECON 0200".to_string(), Y1S.into()),
        ("ECON 2100".to_string(), Y2F.into()),
        ("ECON 2200".to_string(), Y2S.into()),
        ("ECON 2300".to_string(), Y2F.into()),
        ("ECON 2310".to_string(), Y2S.into()),
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
        ("ECON 0100".to_string(), Y1F.into()),
        ("ECON 0200".to_string(), Y1S.into()),
        ("MATH 1080".to_string(), Y1S.into()),
        ("MATH 1410".to_string(), Y1F.into()),
        ("MATH 1610".to_string(), Y1F.into()),
        ("ECON 2100".to_string(), Y2F.into()),
        ("ECON 2200".to_string(), Y2S.into()),
        ("ECON 2300".to_string(), Y2F.into()),
        ("ECON 2310".to_string(), Y2S.into()),
        ("STAT 4300".to_string(), Y2F.into()),
        ("STAT 4310".to_string(), Y2S.into()),
        ("ESE 3010".to_string(), Y2F.into()),
        ("ESE 4020".to_string(), Y2S.into()),
        ("MATH 3000".to_string(), Y2F.into()),
        ("ECON 6100".to_string(), Y3F.into()),
        ("MATH 3600".to_string(), Y3F.into()),
        ("MATH 3610".to_string(), Y3S.into()),
        ("MATH 5080".to_string(), Y3F.into()),
        ("MATH 5090".to_string(), Y3S.into()),
        ("MATH 5460".to_string(), Y3S.into()),
        ("ESE 2310".to_string(), Y3S.into()),
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

const PPE_ADVANCED_SEMINARS: &[&str] = &[
    "PPE 4000",
    "PPE 4500",
    "PPE 4600",
    "PPE 4601",
    "PPE 4650",
    "PPE 4700",
    "PPE 4701",
    "PPE 4800",
    "PPE 4802",
    "PPE 4803",
    "PPE 4804",
    "PPE 4900",
    "PPE 4903",
    "PPE 4950",
];

const PPE_GLOBALIZATION_INTERNATIONAL_ATTRS: &[&str] = &[
    "AIRE", "AIRN", "APSI", "AAFS", "AHAF", "AHCE", "AHES", "AHEA", "AHLA", "AEAE", "ARER",
    "AREE", "AREJ",
];

fn ppe_major_requirements() -> Vec<Requirement> {
    vec![
        Requirement::SingleCourse {
            category: Some("Common Foundations".to_string()),
            possibilities: vec!["PHIL 1433".to_string()],
        },
        Requirement::Restriction {
            category: Some("Common Foundations".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["APPF".to_string()]),
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::Restriction {
            category: Some("Common Foundations".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["APPT".to_string()]),
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::Restriction {
            category: Some("Common Foundations".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["APPP".to_string()]),
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::SingleCourse {
            category: Some("Common Foundations".to_string()),
            possibilities: vec!["ECON 0100".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Common Foundations".to_string()),
            possibilities: vec!["ECON 0200".to_string()],
        },
        Requirement::AnyOf {
            category: Some("Common Foundations".to_string()),
            possibilities: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["PSYC 1210".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["PSYC 1230".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["PSYC 1440".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["PSYC 2737".to_string()],
                },
            ],
        },
        Requirement::SingleCourse {
            category: Some("Common Core".to_string()),
            possibilities: vec!["PPE 3001".to_string()],
        },
        Requirement::SingleCourse {
            category: Some("Common Core".to_string()),
            possibilities: vec!["PPE 3002".to_string()],
        },
        Requirement::AnyOf {
            category: Some("Common Core".to_string()),
            possibilities: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["PPE 3003".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["PPE 3004".to_string()],
                },
            ],
        },
        Requirement::AnyOf {
            category: Some("Advanced Interdisciplinary Seminar in PPE".to_string()),
            possibilities: PPE_ADVANCED_SEMINARS
                .iter()
                .map(|code| Requirement::SingleCourse {
                    category: None,
                    possibilities: vec![(*code).to_string()],
                })
                .collect(),
        },
    ]
}

fn ppe_concentration_requirement(concentration_name: &str) -> Requirement {
    let requirements = ppe_concentrations()
        .get(concentration_name)
        .unwrap_or_else(|| panic!("unknown PPE concentration: {concentration_name}"))
        .clone();
    let number = requirements.iter().map(requirement_slot_cu).sum();
    Requirement::Concentration {
        category: Some(concentration_name.to_string()),
        number,
        requirements,
    }
}

fn ppe_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    BTreeMap::from([
        (
            "Choice and Behaviour".to_string(),
            vec![Requirement::Restriction {
                category: Some("Choice and Behaviour".to_string()),
                department: None,
                cu: None,
                level: None,
                attr: Some(vec!["APPC".to_string()]),
                excluding: None,
                number: 5,
                no_school: None,
            }],
        ),
        (
            "Distributive Justice".to_string(),
            vec![Requirement::Restriction {
                category: Some("Distributive Justice".to_string()),
                department: None,
                cu: None,
                level: None,
                attr: Some(vec!["APPJ".to_string()]),
                excluding: None,
                number: 5,
                no_school: None,
            }],
        ),
        (
            "Globalization".to_string(),
            vec![
                Requirement::Restriction {
                    category: Some("Globalization".to_string()),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(vec!["APPG".to_string()]),
                    excluding: None,
                    number: 4,
                    no_school: None,
                },
                Requirement::Restriction {
                    category: Some("Globalization".to_string()),
                    department: None,
                    cu: None,
                    level: None,
                    attr: Some(
                        PPE_GLOBALIZATION_INTERNATIONAL_ATTRS
                            .iter()
                            .map(|a| (*a).to_string())
                            .collect(),
                    ),
                    excluding: Some(vec!["AUFS".to_string()]),
                    number: 1,
                    no_school: None,
                },
            ],
        ),
        (
            "Public Policy and Governance".to_string(),
            vec![Requirement::Restriction {
                category: Some("Public Policy and Governance".to_string()),
                department: None,
                cu: None,
                level: None,
                attr: Some(vec!["APPU".to_string()]),
                excluding: None,
                number: 5,
                no_school: None,
            }],
        ),
    ])
}

pub fn ppe_concentration_names() -> Vec<String> {
    cas_concentration_names("PPE")
}

pub fn create_ppe_major(concentration_name: String) -> Major {
    let schedule_hints = HashMap::from([
        ("PHIL 1433".to_string(), Y2F.into()),
        ("ECON 0100".to_string(), Y1F.into()),
        ("ECON 0200".to_string(), Y1S.into()),
        ("PPE 3001".to_string(), Y3F.into()),
        ("PPE 3002".to_string(), Y3S.into()),
        ("PPE 3003".to_string(), Y3S.into()),
        ("PPE 3004".to_string(), Y3S.into()),
    ]);
    let mut major_requirements = ppe_major_requirements();
    major_requirements.push(ppe_concentration_requirement(&concentration_name));
    create_cas_major(CasMajorConfig {
        short_name: "PPE".to_string(),
        name: "Philosophy, Politics, and Economics".to_string(),
        major_requirements,
        auto_completed_sectors: vec![],
        concentrations: Some(ppe_concentrations()),
        schedule_hints,
    })
}

pub fn create_cis_cas_major() -> Major {
    let schedule_hints = HashMap::from([
        ("CIS 1100".to_string(), Y1F.into()),
        ("CIS 1200".to_string(), Y1S.into()),
        ("CIS 1600".to_string(), Y1F.into()),
        ("CIS 1210".to_string(), Y2F.into()),
        ("CIS 2400".to_string(), Y2S.into()),
        ("CIS 2620".to_string(), Y3F.into()),
        ("CIS 3200".to_string(), Y3S.into()),
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

const NEUR_INTRO_BIO_ELECTIVES: &[&str] = &[
    "BIOL 2810",
    "BIOL 2010",
    "BIOL 2311",
    "BIOL 3310",
    "BIOL 2210",
    "BIOL 2410",
    "BIOL 2610",
];

fn neur_major_requirements() -> Vec<Requirement> {
    let intro_bio_electives: Vec<String> = NEUR_INTRO_BIO_ELECTIVES
        .iter()
        .map(|code| code.to_string())
        .collect();

    vec![
        Requirement::AnyOf {
            category: Some("Introductory Chemistry".to_string()),
            possibilities: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["CHEM 1011".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["CHEM 1012".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["CHEM 1151".to_string()],
                },
            ],
        },
        Requirement::AnyOf {
            category: Some("Introductory Chemistry".to_string()),
            possibilities: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["CHEM 1021".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["CHEM 1022".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["CHEM 1161".to_string()],
                },
            ],
        },
        Requirement::AnyOf {
            category: Some("Introductory Biology".to_string()),
            possibilities: vec![
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["BIOL 1101".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["BIOL 1102".to_string()],
                        },
                    ],
                },
                Requirement::AllOf {
                    category: None,
                    requirements: vec![
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["BIOL 1121".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["BIOL 1123".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: vec!["BIOL 1124".to_string()],
                        },
                        Requirement::SingleCourse {
                            category: None,
                            possibilities: intro_bio_electives,
                        },
                    ],
                },
            ],
        },
        Requirement::SingleCourse {
            category: Some("Introduction to Brain & Behavior".to_string()),
            possibilities: vec!["NRSC 1110".to_string()],
        },
        Requirement::Restriction {
            category: Some("Neural Systems and Behavior".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["ABBS".to_string()]),
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::Restriction {
            category: Some("Cellular Neuroscience".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["ABBU".to_string()]),
            excluding: None,
            number: 1,
            no_school: None,
        },
        Requirement::AnyOf {
            category: Some("Neurobiology".to_string()),
            possibilities: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["NRSC 2110".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["BIOL 2110".to_string()],
                },
            ],
        },
        Requirement::AnyOf {
            category: Some("Statistics".to_string()),
            possibilities: vec![
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["BIOL 2510".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["STAT 1010".to_string()],
                },
                Requirement::SingleCourse {
                    category: None,
                    possibilities: vec!["STAT 1110".to_string()],
                },
            ],
        },
        Requirement::Restriction {
            category: Some("Neuroscience Electives".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["ABBE".to_string()]),
            excluding: None,
            number: 3,
            no_school: None,
        },
        Requirement::Restriction {
            category: Some("Neuroscience Electives".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec!["ABBE".to_string(), "ABBM".to_string()]),
            excluding: None,
            number: 5,
            no_school: None,
        },
    ]
}

pub fn create_neur_major() -> Major {
    let schedule_hints = HashMap::from([
        ("CHEM 1011".to_string(), Y1F.into()),
        ("CHEM 1012".to_string(), Y1F.into()),
        ("CHEM 1151".to_string(), Y1F.into()),
        ("CHEM 1021".to_string(), Y1S.into()),
        ("CHEM 1022".to_string(), Y1S.into()),
        ("CHEM 1161".to_string(), Y1S.into()),
        ("BIOL 1101".to_string(), Y1F.into()),
        ("BIOL 1102".to_string(), Y1S.into()),
        ("BIOL 1121".to_string(), Y1F.into()),
        ("BIOL 1123".to_string(), Y1S.into()),
        ("BIOL 1124".to_string(), Y2F.into()),
        ("NRSC 1110".to_string(), Y2F.into()),
        ("NRSC 2110".to_string(), Y3F.into()),
        ("BIOL 2110".to_string(), Y3F.into()),
        ("BIOL 2510".to_string(), Y2S.into()),
        ("STAT 1010".to_string(), Y2S.into()),
        ("STAT 1110".to_string(), Y2S.into()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "NEUR".to_string(),
        name: "Neuroscience".to_string(),
        major_requirements: neur_major_requirements(),
        auto_completed_sectors: vec![],
        concentrations: None,
        schedule_hints,
    })
}
