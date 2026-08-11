use std::collections::{BTreeMap, HashMap};
use crate::Major;
use crate::Requirement;
use crate::penn_data::requirement_builders::{
    all_of, any_of, any_of_opt, attr_pool_constraint, attr_restriction, code,
    concentration, course_pool, repeat_req, restriction, single,
    unrestricted_elective,
};
use crate::requirement::{PoolConstraint, PoolCoverageInfo};
use crate::schedule_template::{ScheduleHint, Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S};
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

/// Default sector auto-completions by CAS major code.
/// Sourced from Penn College "Arts and Sciences C.U., Total C.U., & Sector Completed by Major Concentration".
fn cas_default_auto_completed_sectors(short_name: &str) -> Vec<String> {
    match short_name {
        "AFRC" => vec![SECTOR_HUM_SOC_SCI.to_string()],
        "ANCH" => vec![SECTOR_HISTORY.to_string()],
        "ANTH" => vec![SECTOR_LIVING_WORLD.to_string()],
        "ARCH" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "ARTH" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "BIOC" => vec![SECTOR_PHYSICAL_WORLD.to_string()],
        "BIOL" => vec![SECTOR_LIVING_WORLD.to_string(), SECTOR_PHYSICAL_WORLD.to_string()],
        "BIOP" => vec![SECTOR_LIVING_WORLD.to_string(), SECTOR_PHYSICAL_WORLD.to_string()],
        "CHEM" => vec![SECTOR_PHYSICAL_WORLD.to_string()],
        "CIMS" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "CLST" => vec![SECTOR_HISTORY.to_string()],
        "COGS" => vec![SECTOR_NAT_SCI.to_string()],
        "COML" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "COMM" => vec![SECTOR_SOCIETY.to_string()],
        "CRIM" => vec![SECTOR_SOCIETY.to_string()],
        "DSGN" => vec![SECTOR_HUM_SOC_SCI.to_string()],
        "EALC" => vec![SECTOR_HISTORY.to_string()],
        "ECON" => vec![SECTOR_SOCIETY.to_string()],
        "EESC" => vec![SECTOR_LIVING_WORLD.to_string(), SECTOR_PHYSICAL_WORLD.to_string()],
        "ENGL" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "ENVS" => vec![SECTOR_PHYSICAL_WORLD.to_string()],
        "FIGS" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "FNAR" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "GSWS" => vec![SECTOR_SOCIETY.to_string()],
        "HISP" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "HIST" => vec![SECTOR_HISTORY.to_string()],
        "HSOC" => vec![SECTOR_HUM_SOC_SCI.to_string()],
        "INDM" => vec![],
        "INST" => vec![SECTOR_SOCIETY.to_string(), SECTOR_HUM_SOC_SCI.to_string()],
        "INTR" => vec![SECTOR_SOCIETY.to_string()],
        "JWST" => vec![SECTOR_HISTORY.to_string()],
        "LALS" => vec![SECTOR_HISTORY.to_string()],
        "LAWS" => vec![SECTOR_SOCIETY.to_string()],
        "LGIC" => vec![SECTOR_PHYSICAL_WORLD.to_string()],
        "LING" => vec![SECTOR_NAT_SCI.to_string()],
        "MATH" => vec![SECTOR_NAT_SCI.to_string()],
        "MECON" => vec![SECTOR_SOCIETY.to_string()],
        "MELC" => vec![SECTOR_HISTORY.to_string()],
        "MMES" => vec![SECTOR_HISTORY.to_string()],
        "MUSC" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "NEUR" => vec![SECTOR_LIVING_WORLD.to_string(), SECTOR_PHYSICAL_WORLD.to_string()],
        "NUTR" => vec![SECTOR_LIVING_WORLD.to_string(), SECTOR_PHYSICAL_WORLD.to_string()],
        "PHIL" => vec![SECTOR_HISTORY.to_string()],
        "PHYS" => vec![SECTOR_PHYSICAL_WORLD.to_string()],
        "PPE" => vec![SECTOR_SOCIETY.to_string()],
        "PSCI" => vec![SECTOR_SOCIETY.to_string()],
        "PSYC" => vec![SECTOR_LIVING_WORLD.to_string()],
        "REES" => vec![SECTOR_HUM_SOC_SCI.to_string()],
        "RELS" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "SAST" => vec![SECTOR_HISTORY.to_string()],
        "SOCI" => vec![SECTOR_SOCIETY.to_string()],
        "STSC" => vec![SECTOR_HUM_SOC_SCI.to_string()],
        "THAR" => vec![SECTOR_ARTS_LETTERS.to_string()],
        "URBS" => vec![SECTOR_HUM_SOC_SCI.to_string()],
        "VIST" => vec![SECTOR_NAT_SCI.to_string()],
        _ => vec![],
    }
}

fn cas_concentration_sector_override(short_name: &str, concentration: &str) -> Option<Vec<String>> {
    let sectors = match (short_name, concentration) {
        ("ANTH", "Medical Anthropology & Global Health") => vec![SECTOR_HUM_SOC_SCI.to_string()],
        ("MATH", "Biological Mathematics") => {
            vec![SECTOR_LIVING_WORLD.to_string(), SECTOR_NAT_SCI.to_string()]
        }
        ("PHYS", "Biological Science") => {
            vec![SECTOR_LIVING_WORLD.to_string(), SECTOR_PHYSICAL_WORLD.to_string()]
        }
        _ => return None,
    };
    Some(sectors)
}

/// Sector attribute codes auto-completed when a CAS major is declared.
pub fn cas_auto_completed_sectors_for(
    short_name: &str,
    concentration: Option<&str>,
) -> Vec<String> {
    if let Some(conc) = concentration {
        if let Some(sectors) = cas_concentration_sector_override(short_name, conc) {
            return sectors;
        }
    }
    cas_default_auto_completed_sectors(short_name)
}

/// Patch gen-ed pool constraints after resolving a concentration-specific sector mapping.
pub fn apply_cas_auto_completed_sectors(major: &mut Major, concentration: Option<&str>) {
    let sectors = cas_auto_completed_sectors_for(&major.short_name, concentration);
    for req in &mut major.requirements {
        if let Requirement::CoursePool { category, constraints, .. } = req {
            if category.as_deref() == Some(CAS_GENED_POOL_CATEGORY) {
                *constraints = cas_pool_constraints(&sectors);
                return;
            }
        }
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

pub const CAS_UNRESTRICTED_ELECTIVES_CATEGORY: &str = "Unrestricted Electives";

/// Two or more CAS majors = one College degree with multiple majors (double major).
pub fn is_cas_college_double_major(degree_schools: &[String]) -> bool {
    degree_schools.len() >= 2 && degree_schools.iter().all(|s| s == "CAS")
}

pub fn is_cas_unrestricted_elective_requirement(req: &Requirement) -> bool {
    req.get_category() == CAS_UNRESTRICTED_ELECTIVES_CATEGORY
}

/// Top-level unrestricted elective slot (`2`, `3`, …), not pool flex/constraint keys.
pub fn is_cas_unrestricted_elective_instance_scope(scope: &str, major: &Major) -> bool {
    let Ok(idx) = scope.parse::<usize>() else {
        return false;
    };
    major
        .requirements
        .get(idx)
        .is_some_and(is_cas_unrestricted_elective_requirement)
}

/// College-wide requirement shared across CAS double majors (writing, gen-ed, unrestricted).
pub fn is_cas_college_wide_requirement_scope(scope: &str, major: &Major) -> bool {
    is_cas_college_shared_instance_scope(scope)
        || is_cas_unrestricted_elective_instance_scope(scope, major)
}

/// Combined major CU after subtracting cross-major overlap savings (1 CU per shared course).
pub fn cas_effective_combined_major_cu(cas_majors: &[&Major], overlap_cu_savings: i32) -> i32 {
    let nominal: i32 = cas_majors.iter().map(|m| cas_major_pool_major_cu(m)).sum();
    (nominal - overlap_cu_savings.max(0)).max(0)
}

/// CU remaining in a CAS degree after writing (1) and major requirements.
pub fn cas_degree_remaining_after_major(effective_major_cu: i32) -> i32 {
    (CAS_DEGREE_CU - 1 - effective_major_cu).max(0)
}

/// Gen-ed course slots still needed after major auto-sectors (open FA + remaining sectors).
pub fn cas_open_gen_ed_slot_count(auto_completed_sectors: &[String]) -> i32 {
    cas_pool_constraints(auto_completed_sectors).len() as i32
}

/// Shared gen-ed pool flex placeholders for one CAS college degree.
/// Sized to open coverage needs, capped by remaining CU after writing + major(s).
pub fn cas_shared_gened_flex_slots(effective_major_cu: i32, open_gen_ed_slots: i32) -> i32 {
    cas_degree_remaining_after_major(effective_major_cu).min(open_gen_ed_slots.max(0))
}

/// Residual unrestricted electives after writing, major(s), and gen-ed course slots.
pub fn cas_shared_unrestricted_elective_count(effective_major_cu: i32, open_gen_ed_slots: i32) -> i32 {
    let remaining = cas_degree_remaining_after_major(effective_major_cu);
    let gen_ed = cas_shared_gened_flex_slots(effective_major_cu, open_gen_ed_slots);
    (remaining - gen_ed).max(0)
}

pub fn cas_unrestricted_elective_instance_ids(major: &Major) -> Vec<String> {
    major
        .requirements
        .iter()
        .enumerate()
        .filter(|(_, req)| is_cas_unrestricted_elective_requirement(req))
        .map(|(idx, _)| idx.to_string())
        .collect()
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

/// Major-specific CU inside a CAS degree's gen-ed pool (`CoursePool::fixed_slots`).
pub fn cas_major_pool_major_cu(major: &Major) -> i32 {
    major
        .requirements
        .iter()
        .find_map(|req| {
            if let Requirement::CoursePool { fixed_slots, .. } = req {
                Some(fixed_slots.iter().map(requirement_slot_cu).sum::<i32>())
            } else {
                None
            }
        })
        .unwrap_or(0)
}

/// Shared flexible pool slots for a CAS college double major (one writing + one gen-ed pool).
pub fn cas_double_major_shared_flexible_slots(
    cas_majors: &[&Major],
    overlap_cu_savings: i32,
) -> i32 {
    cas_cross_degree_gened_flex_cap(cas_majors, overlap_cu_savings)
}

/// Union of auto-completed sector attrs across CAS majors (college-wide gen-ed).
pub fn cas_college_auto_completed_sectors(cas_majors: &[&Major]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for major in cas_majors {
        for attr in cas_auto_completed_sectors_for(&major.short_name, None) {
            if seen.insert(attr.clone()) {
                out.push(attr);
            }
        }
    }
    out
}

/// Max gen-ed pool flex placeholders on the schedule when CAS is in a cross-degree plan.
pub fn cas_cross_degree_gened_flex_cap(cas_majors: &[&Major], overlap_cu_savings: i32) -> i32 {
    let gen_ed_rows = cas_gened_requirement_row_count() as i32;
    match cas_majors.len() {
        0 => 0,
        1 => {
            let major = cas_majors[0];
            let pool_flex = cas_gened_pool(major)
                .and_then(|(idx, _)| {
                    if let Requirement::CoursePool { flexible_slots, .. } = &major.requirements[idx] {
                        Some(*flexible_slots)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            if pool_flex > 0 {
                return pool_flex.min(gen_ed_rows);
            }
            let autos = cas_auto_completed_sectors_for(&major.short_name, None);
            let open = cas_open_gen_ed_slot_count(&autos);
            let effective = cas_major_pool_major_cu(major);
            cas_shared_gened_flex_slots(effective, open).min(gen_ed_rows)
        }
        _ => {
            let effective =
                cas_effective_combined_major_cu(cas_majors, overlap_cu_savings);
            let autos = cas_college_auto_completed_sectors(cas_majors);
            let open = cas_open_gen_ed_slot_count(&autos);
            cas_shared_gened_flex_slots(effective, open).min(gen_ed_rows)
        }
    }
}

pub fn cas_pool_flexible_slot_index(scope: &str) -> Option<usize> {
    let colon_p = scope.find(":p")?;
    scope[colon_p + 2..].parse().ok()
}

pub fn is_cas_excess_shared_flexible_slot(scope: &str, cap: i32) -> bool {
    if !scope.contains(":p") || !is_cas_college_shared_instance_scope(scope) {
        return false;
    }
    cas_pool_flexible_slot_index(scope)
        .is_some_and(|idx| idx >= cap as usize)
}

pub fn is_cas_excess_shared_flexible_schedule_slot(slot_id: &str, cap: i32) -> bool {
    let Some(rest) = slot_id.strip_prefix("req:") else {
        return false;
    };
    let scope = rest.split(":R:").next().unwrap_or(rest);
    is_cas_excess_shared_flexible_slot(scope, cap)
}

pub const CAS_GENED_POOL_CATEGORY: &str = "General Education";
pub fn cas_gened_pool(major: &Major) -> Option<(usize, String)> {
    for (idx, req) in major.requirements.iter().enumerate() {
        if let Requirement::CoursePool { category, .. } = req {
            if category.as_deref() == Some(CAS_GENED_POOL_CATEGORY) {
                return Some((idx, CAS_GENED_POOL_CATEGORY.to_string()));
            }
        }
    }
    None
}

/// Pool coverage constraint (`{pool}:c{n}`), not a fixed/flex placeholder.
pub fn is_cas_gened_pool_constraint_key(slot_key: &str, pool_idx: usize) -> bool {
    crate::requirement::is_pool_constraint_instance_id(Some(slot_key))
        && slot_key.starts_with(&format!("{pool_idx}:"))
}

/// Flexible gen-ed placeholder (`{pool}:p{n}`).
pub fn is_cas_gened_pool_flex_key(slot_key: &str, pool_idx: usize) -> bool {
    slot_key.starts_with(&format!("{pool_idx}:p"))
}

pub fn cas_gened_overlap_display_label(major: &Major) -> Option<String> {
    cas_gened_pool(major).map(|(_, cat)| cat.to_string())
}

pub fn is_cas_gened_flex_schedule_slot(slot_id: &str, pool_idx: usize) -> bool {
    let Some(rest) = slot_id.strip_prefix("req:") else {
        return false;
    };
    let scope = rest.split(":R:").next().unwrap_or(rest);
    is_cas_gened_pool_flex_key(scope, pool_idx)
}

pub fn cas_gened_requirement_row_count() -> usize {
    FOUNDATIONAL_APPROACHES.len() + SECTORS.len()
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
    restriction(1)
        .category("Writing Seminar")
        .departments(&["WRIT"])
        .into()
}

fn cas_foundational_approach(label: &str, attr: &str) -> Requirement {
    restriction(1)
        .category(&format!("Foundational Approaches - {label}"))
        .attr(&[attr])
        .into()
}

fn cas_sector_requirement(label: &str, attr: &str) -> Requirement {
    restriction(1)
        .category(&format!("Sectors of Knowledge - {label}"))
        .attr(&[attr])
        .into()
}

/// Coverage constraints: FAs + non-auto-completed Sectors.
pub fn cas_pool_constraints(auto_completed_sectors: &[String]) -> Vec<PoolConstraint> {
    let mut constraints = Vec::new();

    for (label, attr) in FOUNDATIONAL_APPROACHES {
        constraints.push(attr_pool_constraint(
            &format!("Foundational Approaches - {label}"),
            attr,
            1,
            "cas:fa",
        ));
    }

    for (label, attr) in SECTORS {
        if !auto_completed_sectors.iter().any(|s| s == attr) {
            constraints.push(attr_pool_constraint(
                &format!("Sectors of Knowledge - {label}"),
                attr,
                1,
                "cas:sector",
            ));
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

fn cas_unrestricted_elective() -> Requirement {
    unrestricted_elective(CAS_UNRESTRICTED_ELECTIVES_CATEGORY)
}

/// Assemble a full CAS degree: writing + major/gen-ed pool (no pre-sized flex/unrestricted).
///
/// Flexible gen-ed placeholders and unrestricted electives are materialized after assignment
/// by [`crate::requirement::assign_cas_college`] so residual CU reaches [`CAS_DEGREE_CU`].
pub fn create_cas_major(config: CasMajorConfig) -> Major {
    let auto_completed_sectors = if config.auto_completed_sectors.is_empty() {
        cas_default_auto_completed_sectors(&config.short_name)
    } else {
        config.auto_completed_sectors
    };

    let requirements = vec![
        cas_writing_requirement(),
        course_pool(
            "General Education",
            config.major_requirements,
            0, // flex sized after college assignment
            cas_pool_constraints(&auto_completed_sectors),
        ),
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

/// Append gen-ed flex capacity and unrestricted elective slots onto a CAS major after assignment.
pub fn materialize_cas_college_structure(
    major: &mut Major,
    flexible_slots: i32,
    unrestricted_count: i32,
    college_constraints: Option<Vec<PoolConstraint>>,
) {
    major
        .requirements
        .retain(|req| !is_cas_unrestricted_elective_requirement(req));
    for req in &mut major.requirements {
        if let Requirement::CoursePool {
            category,
            flexible_slots: flex,
            constraints,
            ..
        } = req
        {
            if category.as_deref() == Some(CAS_GENED_POOL_CATEGORY) {
                *flex = flexible_slots.max(0);
                if let Some(ref c) = college_constraints {
                    *constraints = c.clone();
                }
                break;
            }
        }
    }
    for _ in 0..unrestricted_count.max(0) {
        major.requirements.push(cas_unrestricted_elective());
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
            "Race, Gender, and Health",
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

const ANCIENT_HISTORY_DEPTS: &[&str] = &["ANCH", "CLST", "GREK", "LATN"];

fn ancient_history_pool_slot(category: Option<String>, level: Option<i32>) -> Requirement {
    let mut possibilities: Vec<Requirement> = ANCIENT_HISTORY_DEPTS
        .iter()
        .map(|dept| {
            let mut b = restriction(1).departments(&[dept]);
            if let Some(l) = level {
                b = b.level(l);
            }
            b.into()
        })
        .collect();
    let mut attr_b = restriction(1).attr(&["AANP"]);
    if let Some(l) = level {
        attr_b = attr_b.level(l);
    }
    possibilities.push(attr_b.into());
    any_of_opt(category, possibilities)
}

fn ancient_history_pool_slots(
    category: &str,
    count: usize,
    level: Option<i32>,
) -> Vec<Requirement> {
    (0..count)
        .map(|_| ancient_history_pool_slot(Some(category.to_string()), level))
        .collect()
}

fn anch_major_requirements() -> Vec<Requirement> {
    let core_courses = [
        "ANCH 0100",
        "ANCH 0101",
        "ANCH 0102",
        "ANCH 1100",
        "CLST 1300",
        "CLST 1500",
    ];
    let core_class = single("Core Classes", &core_courses);
    let mut requirements = repeat_req(&core_class, 2);
    requirements.extend(ancient_history_pool_slots("Graeco-Roman World", 2, None));
    requirements.extend(ancient_history_pool_slots(
        "Graeco-Roman World",
        2,
        Some(3000),
    ));
    requirements.extend(ancient_history_pool_slots(
        "Advanced Topics and Area Groupings",
        4,
        None,
    ));
    requirements.extend(ancient_history_pool_slots(
        "Advanced Topics and Area Groupings",
        2,
        Some(3000),
    ));
    requirements
}

pub fn create_anch_major() -> Major {
    create_cas_major(CasMajorConfig {
        short_name: "ANCH".to_string(),
        name: "Ancient History".to_string(),
        major_requirements: anch_major_requirements(),
        auto_completed_sectors: vec![],
        concentrations: None,
        schedule_hints: HashMap::new(),
    })
}

fn econ_major_requirements() -> Vec<Requirement> {
    let econ_elective: Requirement = restriction(1)
        .category("ECON Electives")
        .departments(&["ECON"])
        .level(4000)
        .into();
    vec![
        single("Introductory Economics", &["ECON 0100"]),
        single("Introductory Economics", &["ECON 0200"]),
        single("Intermediate Economics", &["ECON 2100"]),
        single("Intermediate Economics", &["ECON 2200"]),
        any_of(
            "Statistics",
            vec![
                code(&["ECON 2300"]),
                all_of(
                    None,
                    vec![code(&["STAT 4300"]), code(&["STAT 4310"])],
                ),
            ],
        ),
        single("Econometrics", &["ECON 2310"]),
        econ_elective.clone(),
        econ_elective.clone(),
        econ_elective.clone(),
        econ_elective,
        any_of(
            "Mathematics",
            vec![
                all_of(
                    None,
                    vec![
                        code(&["MATH 1400"]),
                        code(&["MATH 1410", "MATH 1610"]),
                    ],
                ),
                all_of(
                    None,
                    vec![code(&["MATH 1070"]), code(&["MATH 1080"])],
                ),
            ],
        ),
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
    let econ_elective: Requirement = restriction(1)
        .category("ECON Electives")
        .attr(&["AMAE"])
        .into();
    let math_elective: Requirement = restriction(1)
        .category("Math Electives")
        .attr(&["AMAM"])
        .into();
    vec![
        single("Introductory Economics", &["ECON 0100"]),
        single("Introductory Economics", &["ECON 0200"]),
        single("Intermediate Economics", &["ECON 2100"]),
        single("Intermediate Economics", &["ECON 2200"]),
        single("Intermediate Economics", &["ECON 6100"]),
        any_of(
            "Stat Core",
            vec![
                all_of(
                    None,
                    vec![
                        code(&["ECON 2300"]),
                        code(&[
                            "MATH 5460",
                            "ECON 2310",
                            "ECON 4310",
                            "ECON 4320",
                        ]),
                    ],
                ),
                all_of(
                    None,
                    vec![code(&["STAT 4300"]), code(&["STAT 4310"])],
                ),
                all_of(
                    None,
                    vec![code(&["ESE 3010"]), code(&["ESE 4020"])],
                ),
                all_of(
                    None,
                    vec![code(&["STAT 4300"]), code(&["ESE 2310"])],
                ),
            ],
        ),
        econ_elective.clone(),
        econ_elective,
        single(
            "Math Core",
            &["MATH 1410", "MATH 1610", "MATH 1080"],
        ),
        single("Math Core", &["MATH 3000"]),
        any_of(
            "Math Core",
            vec![
                all_of(
                    None,
                    vec![code(&["MATH 3600"]), code(&["MATH 3610"])],
                ),
                all_of(
                    None,
                    vec![code(&["MATH 5080"]), code(&["MATH 5090"])],
                ),
            ],
        ),
        math_elective.clone(),
        math_elective.clone(),
        math_elective,
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
    vec![
        single("Core Courses", &["CIS 1100"]),
        single("Core Courses", &["CIS 1200"]),
        single("Core Courses", &["CIS 1600"]),
        single("Core Courses", &["CIS 1210"]),
        single("Core Courses", &["CIS 2400"]),
        single("Core Courses", &["CIS 2620"]),
        single("Core Courses", &["CIS 3200"]),
        single("Project Electives", CIS_PROJECT_ELECTIVES),
        single("Project Electives", CIS_PROJECT_ELECTIVES),
        any_of(
            "CIS Elective",
            vec![
                restriction(1).departments(&["CIS", "NETS"]).into(),
                code(&["ESE 3500"]),
            ],
        ),
        any_of(
            "CIS Elective",
            vec![
                restriction(1)
                    .departments(&["CIS", "NETS"])
                    .level(2000)
                    .into(),
                code(&["ESE 3500"]),
            ],
        ),
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
        single("Common Foundations", &["PHIL 1433"]),
        restriction(1)
            .category("Common Foundations")
            .attr(&["APPF"])
            .into(),
        restriction(1)
            .category("Common Foundations")
            .attr(&["APPT"])
            .into(),
        restriction(1)
            .category("Common Foundations")
            .attr(&["APPP"])
            .into(),
        single("Common Foundations", &["ECON 0100"]),
        single("Common Foundations", &["ECON 0200"]),
        any_of(
            "Common Foundations",
            vec![
                code(&["PSYC 1210"]),
                code(&["PSYC 1230"]),
                code(&["PSYC 1440"]),
                code(&["PSYC 2737"]),
            ],
        ),
        single("Common Core", &["PPE 3001"]),
        single("Common Core", &["PPE 3002"]),
        any_of(
            "Common Core",
            vec![code(&["PPE 3003"]), code(&["PPE 3004"])],
        ),
        any_of(
            "Advanced Interdisciplinary Seminar in PPE",
            PPE_ADVANCED_SEMINARS
                .iter()
                .map(|c| code(&[*c]))
                .collect(),
        ),
    ]
}

fn ppe_concentration_requirement(concentration_name: &str) -> Requirement {
    let requirements = ppe_concentrations()
        .get(concentration_name)
        .unwrap_or_else(|| panic!("unknown PPE concentration: {concentration_name}"))
        .clone();
    let number = requirements.iter().map(requirement_slot_cu).sum();
    concentration(concentration_name, number, requirements)
}

fn ppe_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    BTreeMap::from([
        (
            "Choice and Behaviour".to_string(),
            vec![restriction(5)
                .category("Choice and Behaviour")
                .attr(&["APPC"])
                .into()],
        ),
        (
            "Distributive Justice".to_string(),
            vec![restriction(5)
                .category("Distributive Justice")
                .attr(&["APPJ"])
                .into()],
        ),
        (
            "Globalization".to_string(),
            vec![
                restriction(4)
                    .category("Globalization")
                    .attr(&["APPG"])
                    .into(),
                restriction(1)
                    .category("Globalization")
                    .attr(PPE_GLOBALIZATION_INTERNATIONAL_ATTRS)
                    .excluding(&["AUFS"])
                    .into(),
            ],
        ),
        (
            "Public Policy and Governance".to_string(),
            vec![restriction(5)
                .category("Public Policy and Governance")
                .attr(&["APPU"])
                .into()],
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

fn phys_core_requirements() -> Vec<Requirement> {
    vec![
        single("Calculus", &["MATH 1400"]),
        any_of(
            "Calculus",
            vec![code(&["MATH 1410"]), code(&["MATH 1610"])],
        ),
        single("Linear Algebra", &["MATH 2200", "ESE 2030"]),
        single("Differential Equations", &["MATH 2300"]),
        any_of(
            "Introductory Physics",
            vec![code(&["PHYS 0150"]), code(&["PHYS 0170"])],
        ),
        any_of(
            "Introductory Physics",
            vec![code(&["PHYS 0151"]), code(&["PHYS 0171"])],
        ),
        single("Intermediate Physics", &["PHYS 1230"]),
        single("Electromagnetism", &["PHYS 3361"]),
        single("Electromagnetism", &["PHYS 3362"]),
        single("Quantum Mechanics", &["PHYS 4411"]),
    ]
}

const PHYS_BUSINESS_DEPTS: &[&str] = &["ACCT", "ECON", "FNCE", "MGMT", "STAT"];
const PHYS_CIS_EXCLUDE: &[&str] = &["CIS 2970", "CIS 2980"];

fn phys_business_tech_elective_slot() -> Requirement {
    any_of(
        "Electives in Business and Technology",
        vec![
            restriction(1)
                .departments(PHYS_BUSINESS_DEPTS)
                .excluding(PHYS_CIS_EXCLUDE)
                .into(),
            restriction(1)
                .attr(&["APHE"])
                .excluding(PHYS_CIS_EXCLUDE)
                .into(),
        ],
    )
}

fn phys_cis_elective_slot() -> Requirement {
    any_of(
        "Elective in Computer and Information Science",
        vec![
            restriction(1).departments(&["CIS"]).level(1000).into(),
            code(&["ENGR 1050"]),
            code(&["PHYS 2260"]),
            code(&["PHYS 3358"]),
            code(&["PHYS 3359"]),
        ],
    )
}

fn phys_computer_techniques_elective_slot() -> Requirement {
    any_of(
        "Computer Techniques Electives",
        vec![
            restriction(1)
                .departments(&["CIS"])
                .level(1000)
                .excluding(PHYS_CIS_EXCLUDE)
                .into(),
            code(&["ENGR 1050"]),
        ],
    )
}

fn phys_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    let aphl_elective = attr_restriction("Astrophysics Electives", "APHL");
    let phys_lab_elective = attr_restriction("Physics Laboratory Elective", "APHL");
    BTreeMap::from([
        (
            "Astrophysics".to_string(),
            vec![
                single("Modern Physics", &["PHYS 1250"]),
                single("Analytical Mechanics", &["PHYS 3351"]),
                single("Astrophysics", &["ASTR 1211"]),
                single("Astrophysics", &["ASTR 1212"]),
                single("Thermodynamics and Statistical Mechanics", &["PHYS 4401"]),
                aphl_elective.clone(),
                aphl_elective.clone(),
                attr_restriction("Astrophysics Electives", "APHA"),
            ],
        ),
        (
            "Business & Technology".to_string(),
            vec![
                single("Modern Physics", &["PHYS 1250"]),
                single("Analytical Mechanics", &["PHYS 3351"]),
                any_of(
                    "Laboratory",
                    vec![code(&["PHYS 3364"]), code(&["PHYS 4414"])],
                ),
                phys_cis_elective_slot(),
                phys_business_tech_elective_slot(),
                phys_business_tech_elective_slot(),
                phys_business_tech_elective_slot(),
                phys_business_tech_elective_slot(),
            ],
        ),
        (
            "Biological Science".to_string(),
            vec![
                any_of(
                    "Modern Physics",
                    vec![code(&["PHYS 1240"]), code(&["PHYS 1250"])],
                ),
                single("Introductory Biology", &["BIOL 1121"]),
                single("Introductory Biology Laboratory", &["BIOL 1123"]),
                single("Molecular Biology and Genetics", &["BIOL 2210"]),
                any_of(
                    "Biochemistry / Cell Biology",
                    vec![code(&["BIOL 2810"]), code(&["BIOL 2010"])],
                ),
                any_of(
                    "Biological Physics",
                    vec![code(&["PHYS 2280"]), code(&["PHYS 5580"])],
                ),
                single("Thermodynamics and Statistical Mechanics", &["PHYS 4401"]),
                restriction(1)
                    .category("Biology Elective (2000 Level)")
                    .departments(&["BIOL"])
                    .level(2000)
                    .into(),
                restriction(1)
                    .category("Biology Elective (3000+ Level)")
                    .departments(&["BIOL"])
                    .level(3000)
                    .into(),
            ],
        ),
        (
            "Chemical Principles".to_string(),
            vec![
                single("Modern Physics", &["PHYS 1250"]),
                single("Analytical Mechanics", &["PHYS 3351"]),
                single("Thermodynamics and Statistical Mechanics", &["PHYS 4401"]),
                any_of(
                    "Introductory Chemistry I",
                    vec![
                        code(&["CHEM 1011"]),
                        code(&["CHEM 1012"]),
                        code(&["CHEM 1151"]),
                    ],
                ),
                any_of(
                    "Introductory Chemistry II",
                    vec![
                        code(&["CHEM 1021"]),
                        code(&["CHEM 1022"]),
                        code(&["CHEM 1161"]),
                    ],
                ),
                any_of(
                    "Advanced Chemistry",
                    vec![
                        all_of(
                            None,
                            vec![code(&["CHEM 2210"]), code(&["CHEM 2220"])],
                        ),
                        all_of(
                            None,
                            vec![code(&["CHEM 2410"]), code(&["CHEM 2420"])],
                        ),
                    ],
                ),
            ],
        ),
        (
            "Computer Techniques".to_string(),
            vec![
                single("Modern Physics", &["PHYS 1250"]),
                single("Analytical Mechanics", &["PHYS 3351"]),
                single("Thermodynamics and Statistical Mechanics", &["PHYS 4401"]),
                phys_lab_elective,
                phys_computer_techniques_elective_slot(),
                phys_computer_techniques_elective_slot(),
                phys_computer_techniques_elective_slot(),
            ],
        ),
        (
            "Physical Theory and Experimental Technique".to_string(),
            vec![
                single("Modern Physics", &["PHYS 1250"]),
                single("Analytical Mechanics", &["PHYS 3351"]),
                single("Thermodynamics and Statistical Mechanics", &["PHYS 4401"]),
                single("Quantum Mechanics", &["PHYS 4412"]),
                attr_restriction("Physics Laboratory Elective", "APHL"),
                any_of(
                    "Advanced Physics / Astrophysics Elective",
                    vec![
                        restriction(1).departments(&["ASTR"]).level(3000).into(),
                        restriction(1).departments(&["PHYS"]).level(3000).into(),
                    ],
                ),
            ],
        ),
    ])
}

fn phys_concentration_requirement(concentration_name: &str) -> Requirement {
    let requirements = phys_concentrations()
        .get(concentration_name)
        .cloned()
        .unwrap_or_default();
    let number = requirements.iter().map(requirement_slot_cu).sum();
    concentration(
        &format!("Concentration - {concentration_name}"),
        number,
        requirements,
    )
}

pub fn phys_concentration_names() -> Vec<String> {
    cas_concentration_names("PHYS")
}

pub fn create_phys_major(concentration_name: String) -> Major {
    let schedule_hints = HashMap::from([
        ("MATH 1400".to_string(), Y1F.into()),
        ("MATH 1410".to_string(), Y1S.into()),
        ("MATH 1610".to_string(), Y1S.into()),
        ("PHYS 0150".to_string(), Y1F.into()),
        ("PHYS 0170".to_string(), Y1F.into()),
        ("PHYS 0151".to_string(), Y1S.into()),
        ("PHYS 0171".to_string(), Y1S.into()),
        ("MATH 2200".to_string(), Y2F.into()),
        ("ESE 2030".to_string(), Y2F.into()),
        ("PHYS 1230".to_string(), Y2F.into()),
        ("MATH 2300".to_string(), Y2S.into()),
        ("PHYS 1250".to_string(), Y2S.into()),
        ("PHYS 1240".to_string(), Y2S.into()),
        ("PHYS 3351".to_string(), Y3F.into()),
        ("PHYS 3361".to_string(), Y3F.into()),
        ("PHYS 3362".to_string(), Y3S.into()),
        ("PHYS 4411".to_string(), Y4F.into()),
        ("ASTR 1211".to_string(), Y3S.into()),
        ("ASTR 1212".to_string(), Y4F.into()),
        ("PHYS 4401".to_string(), Y4F.into()),
        ("PHYS 4412".to_string(), Y4S.into()),
        ("CHEM 1011".to_string(), Y1F.into()),
        ("CHEM 1012".to_string(), Y1F.into()),
        ("CHEM 1151".to_string(), Y1F.into()),
        ("CHEM 1021".to_string(), Y1S.into()),
        ("CHEM 1022".to_string(), Y1S.into()),
        ("CHEM 1161".to_string(), Y1S.into()),
        ("CHEM 2210".to_string(), Y3F.into()),
        ("CHEM 2220".to_string(), Y3S.into()),
        ("CHEM 2410".to_string(), Y3F.into()),
        ("CHEM 2420".to_string(), Y3S.into()),
        ("ENGR 1050".to_string(), Y2S.into()),
        ("BIOL 1121".to_string(), Y2F.into()),
        ("BIOL 1123".to_string(), Y2F.into()),
        ("BIOL 2210".to_string(), Y3F.into()),
    ]);
    let mut major_requirements = phys_core_requirements();
    major_requirements.push(phys_concentration_requirement(&concentration_name));
    create_cas_major(CasMajorConfig {
        short_name: "PHYS".to_string(),
        name: "Physics".to_string(),
        major_requirements,
        auto_completed_sectors: vec![SECTOR_PHYSICAL_WORLD.to_string()],
        concentrations: Some(phys_concentrations()),
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
    vec![
        any_of(
            "Introductory Chemistry",
            vec![
                code(&["CHEM 1011"]),
                code(&["CHEM 1012"]),
                code(&["CHEM 1151"]),
            ],
        ),
        any_of(
            "Introductory Chemistry",
            vec![
                code(&["CHEM 1021"]),
                code(&["CHEM 1022"]),
                code(&["CHEM 1161"]),
            ],
        ),
        any_of(
            "Introductory Biology",
            vec![
                all_of(
                    None,
                    vec![code(&["BIOL 1101"]), code(&["BIOL 1102"])],
                ),
                all_of(
                    None,
                    vec![
                        code(&["BIOL 1121"]),
                        code(&["BIOL 1123"]),
                        code(&["BIOL 1124"]),
                        code(NEUR_INTRO_BIO_ELECTIVES),
                    ],
                ),
            ],
        ),
        single("Introduction to Brain & Behavior", &["NRSC 1110"]),
        attr_restriction("Neural Systems and Behavior", "ABBS"),
        attr_restriction("Cellular Neuroscience", "ABBU"),
        any_of(
            "Neurobiology",
            vec![code(&["NRSC 2110"]), code(&["BIOL 2110"])],
        ),
        any_of(
            "Statistics",
            vec![
                code(&["BIOL 2510"]),
                code(&["STAT 1010"]),
                code(&["STAT 1110"]),
            ],
        ),
        restriction(3)
            .category("Neuroscience Electives")
            .attr(&["ABBE"])
            .into(),
        restriction(5)
            .category("Neuroscience Electives")
            .attr(&["ABBE", "ABBM"])
            .into(),
    ]
}

const BIOL_LPS_EXCLUDED: &[&str] = &[
    "BIOL 2001",
    "BIOL 2201",
    "BIOL 2301",
    "BIOL 2701",
    "BIOL 2801",
    "BIOL 3004",
    "BIOL 3006",
    "BIOL 3313",
];

const BIOL_ALLIED_SCIENCES: &[&str] = &[
    "CHEM 1011",
    "CHEM 1021",
    "CHEM 1012",
    "CHEM 1022",
    "CHEM 1101",
    "CHEM 1102",
    "PHYS 0101",
    "PHYS 0102",
    "PHYS 0150",
    "PHYS 0151",
    "MATH 1300",
    "MATH 1400",
    "MATH 1410",
    "BIOL 2510",
    "STAT 1110",
    "STAT 1020",
    "CIS 1200",
    "CIS 1600",
];

const BIOL_INTERMEDIATE_GROUP_1: &[&str] = &[
    "BIOL 2010",
    "BIOL 2110",
    "BIOL 2210",
    "BIOL 2810",
    "CHEM 2510",
];

const BIOL_INTERMEDIATE_GROUP_2: &[&str] = &[
    "BIOL 2140",
    "BIOL 3310",
    "BIOL 2311",
    "BIOL 2410",
    "BIOL 2610",
];

const BIOL_RELATED_ATTRS: &[&str] = &[
    "ABB2", "ABXD", "ABAM", "ABCM", "ABAN", "ABCB", "ABEE", "ABGD", "ABGG", "ABMD", "ABMI",
    "ABMC",
];

fn biol_elective_or_related() -> Requirement {
    any_of(
        "Additional Biology",
        vec![
            restriction(1)
                .departments(&["BIOL"])
                .level(2000)
                .max_level(5999)
                .excluding(BIOL_LPS_EXCLUDED)
                .into(),
            restriction(1).attr(BIOL_RELATED_ATTRS).into(),
        ],
    )
}

fn biol_major_requirements() -> Vec<Requirement> {
    let mut requirements = vec![
        any_of(
            "Introductory Biology",
            vec![
                all_of(
                    None,
                    vec![code(&["BIOL 1101"]), code(&["BIOL 1102"])],
                ),
                all_of(
                    None,
                    vec![
                        code(&["BIOL 1121"]),
                        code(&["BIOL 1123"]),
                        code(&["BIOL 1124"]),
                        restriction(1)
                            .departments(&["BIOL"])
                            .level(2000)
                            .max_level(5999)
                            .excluding(BIOL_LPS_EXCLUDED)
                            .into(),
                    ],
                ),
            ],
        ),
    ];
    requirements.extend(repeat_req(
        &single(
            "Physical Sciences, Calculus, Statistics, and Computer Science",
            BIOL_ALLIED_SCIENCES,
        ),
        4,
    ));
    requirements.extend(repeat_req(
        &single("Intermediate Biology (Group 1)", BIOL_INTERMEDIATE_GROUP_1),
        2,
    ));
    requirements.extend(repeat_req(
        &single("Intermediate Biology (Group 2)", BIOL_INTERMEDIATE_GROUP_2),
        2,
    ));
    requirements.extend(repeat_req(&biol_elective_or_related(), 6));
    requirements
}

fn bioc_major_requirements() -> Vec<Requirement> {
    vec![
        single("Mathematics", &["MATH 1400", "MATH 1610", "MATH 2200"]),
        single("Mathematics", &["MATH 1410", "MATH 2300", "MATH 3000"]),
        single("General Chemistry", &["CHEM 1012", "CHEM 1151", "CHEM 1011"]),
        single("General Chemistry", &["CHEM 1022", "CHEM 1161", "CHEM 1021"]),
        all_of(
            Some("General Chemistry Laboratories".to_string()),
            vec![code(&["CHEM 1101"]), code(&["CHEM 1102"])],
        ),
        single("Organic Chemistry with Laboratories", &["CHEM 2411"]),
        single("Organic Chemistry with Laboratories", &["CHEM 2421"]),
        single("Physical Chemistry", &["CHEM 2210"]),
        single("Physical Chemistry", &["CHEM 2220"]),
        single("Biological Chemistry", &["CHEM 2510"]),
        single("Biological Chemistry", &["CHEM 5510"]),
        single("Biological Chemistry", &["CHEM 5520"]),
        single("Physics", &["PHYS 0150", "PHYS 0170"]),
        single("Physics", &["PHYS 0151", "PHYS 0171"]),
        // BCHE 4597 is 2 CU across year 4 (fall + spring).
        single("Research", &["BCHE 4597"]),
        single("Research", &["BCHE 4597"]),
    ]
}

pub fn create_bioc_major() -> Major {
    let schedule_hints = HashMap::from([
        ("MATH 1400".to_string(), Y1F.into()),
        ("MATH 1610".to_string(), Y1F.into()),
        ("MATH 2200".to_string(), Y1F.into()),
        ("MATH 1410".to_string(), Y1S.into()),
        ("MATH 2300".to_string(), Y1S.into()),
        ("MATH 3000".to_string(), Y1S.into()),
        ("CHEM 1011".to_string(), Y1F.into()),
        ("CHEM 1012".to_string(), Y1F.into()),
        ("CHEM 1151".to_string(), Y1F.into()),
        ("CHEM 1021".to_string(), Y1S.into()),
        ("CHEM 1022".to_string(), Y1S.into()),
        ("CHEM 1161".to_string(), Y1S.into()),
        ("CHEM 1101".to_string(), Y1F.into()),
        ("CHEM 1102".to_string(), Y1S.into()),
        ("CHEM 2411".to_string(), Y2F.into()),
        ("CHEM 2421".to_string(), Y2S.into()),
        ("CHEM 2510".to_string(), Y2S.into()),
        ("PHYS 0150".to_string(), Y2F.into()),
        ("PHYS 0170".to_string(), Y2F.into()),
        ("PHYS 0151".to_string(), Y2S.into()),
        ("PHYS 0171".to_string(), Y2S.into()),
        ("CHEM 2210".to_string(), Y3F.into()),
        ("CHEM 2220".to_string(), Y3S.into()),
        ("CHEM 5510".to_string(), Y3F.into()),
        ("CHEM 5520".to_string(), Y3S.into()),
        ("BCHE 4597".to_string(), Y4F.into()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "BIOC".to_string(),
        name: "Biochemistry".to_string(),
        major_requirements: bioc_major_requirements(),
        auto_completed_sectors: vec![],
        concentrations: None,
        schedule_hints,
    })
}

pub fn create_biol_major() -> Major {
    let schedule_hints = HashMap::from([
        ("BIOL 1101".to_string(), Y1F.into()),
        ("BIOL 1102".to_string(), Y1S.into()),
        ("BIOL 1121".to_string(), Y1F.into()),
        ("BIOL 1123".to_string(), Y1S.into()),
        ("BIOL 1124".to_string(), Y1S.into()),
        ("CHEM 1011".to_string(), Y1F.into()),
        ("CHEM 1012".to_string(), Y1F.into()),
        ("CHEM 1021".to_string(), Y1S.into()),
        ("CHEM 1022".to_string(), Y1S.into()),
        ("CHEM 1101".to_string(), Y1F.into()),
        ("CHEM 1102".to_string(), Y1S.into()),
        ("MATH 1400".to_string(), Y1F.into()),
        ("MATH 1410".to_string(), Y1S.into()),
        ("PHYS 0150".to_string(), Y2F.into()),
        ("PHYS 0151".to_string(), Y2S.into()),
        ("BIOL 2010".to_string(), Y2F.into()),
        ("BIOL 2210".to_string(), Y2F.into()),
        ("BIOL 2810".to_string(), Y2S.into()),
        ("CHEM 2510".to_string(), Y2S.into()),
        ("BIOL 2410".to_string(), Y2S.into()),
        ("BIOL 2610".to_string(), Y3F.into()),
        ("BIOL 2510".to_string(), Y2S.into()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "BIOL".to_string(),
        name: "Biology".to_string(),
        major_requirements: biol_major_requirements(),
        auto_completed_sectors: vec![],
        concentrations: None,
        schedule_hints,
    })
}

fn chem_major_requirements() -> Vec<Requirement> {
    vec![
        any_of(
            "General Chemistry",
            vec![
                code(&["CHEM 1011"]),
                code(&["CHEM 1012"]),
                code(&["CHEM 1151"]),
            ],
        ),
        any_of(
            "General Chemistry",
            vec![
                code(&["CHEM 1021"]),
                code(&["CHEM 1022"]),
                code(&["CHEM 1161"]),
            ],
        ),
        single(
            "General Chemistry Laboratories",
            &["CHEM 1101"],
        ),
        single(
            "General Chemistry Laboratories",
            &["CHEM 1102"],
        ),
        single("Organic Chemistry with Laboratories", &["CHEM 2411"]),
        single("Organic Chemistry with Laboratories", &["CHEM 2421"]),
        single("Calculus", &["MATH 1400"]),
        any_of(
            "Calculus",
            vec![code(&["MATH 1410"]), code(&["MATH 1610"])],
        ),
        single("Physics", &["PHYS 0150"]),
        single("Physics", &["PHYS 0151"]),
        single("Physical Chemistry and Laboratories", &["CHEM 2210"]),
        single("Physical Chemistry and Laboratories", &["CHEM 2220"]),
        single("Physical Chemistry and Laboratories", &["CHEM 2230"]),
        single("Biological Chemistry", &["CHEM 2510"]),
        single("Inorganic Chemistry", &["CHEM 2610"]),
        single("One Advanced Laboratory", &["CHEM 2460"]),
    ]
}

pub fn create_chem_major() -> Major {
    let schedule_hints = HashMap::from([
        ("CHEM 1011".to_string(), Y1F.into()),
        ("CHEM 1012".to_string(), Y1F.into()),
        ("CHEM 1151".to_string(), Y1F.into()),
        ("CHEM 1021".to_string(), Y1S.into()),
        ("CHEM 1022".to_string(), Y1S.into()),
        ("CHEM 1161".to_string(), Y1S.into()),
        ("CHEM 1101".to_string(), Y1F.into()),
        ("CHEM 1102".to_string(), Y1S.into()),
        ("MATH 1400".to_string(), Y1F.into()),
        ("MATH 1410".to_string(), Y1S.into()),
        ("MATH 1610".to_string(), Y1S.into()),
        ("PHYS 0150".to_string(), Y2F.into()),
        ("PHYS 0151".to_string(), Y2S.into()),
        ("CHEM 2411".to_string(), Y2F.into()),
        ("CHEM 2421".to_string(), Y2S.into()),
        ("CHEM 2210".to_string(), Y3F.into()),
        ("CHEM 2220".to_string(), Y3S.into()),
        ("CHEM 2230".to_string(), Y4F.into()),
        ("CHEM 2510".to_string(), Y3F.into()),
        ("CHEM 2610".to_string(), Y3S.into()),
        ("CHEM 2460".to_string(), Y4S.into()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "CHEM".to_string(),
        name: "Chemistry".to_string(),
        major_requirements: chem_major_requirements(),
        auto_completed_sectors: vec![],
        concentrations: None,
        schedule_hints,
    })
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

const PSYC_APPROVED_STATISTICS: &[&str] = &[
    "ANTH 3454",
    "BIOL 2510",
    "CRIM 1200",
    "ECON 2300",
    "NURS 2300",
    "SOCI 2010",
    "STAT 1010",
    "STAT 1020",
    "STAT 1110",
    "STAT 1120",
    "STAT 4310",
];

const PSYC_COGNATE_ELECTIVES: &[&str] = &[
    "ANTH 1040",
    "ASAM 1800",
    "ECON 0120",
    "EDUC 2535",
    "EDUC 2541",
    "EDUC 2551",
    "EDUC 3545",
    "GSWS 3440",
    "LING 0750",
    "LING 2700",
    "NRSC 2233",
    "NRSC 2240",
    "NRSC 3310",
    "NRSC 4430",
    "NRSC 4469",
    "NRSC 4470",
    "NRSC 4482",
    "OIDD 2900",
    "PHIL 4840",
    "PHIL 4843",
    "PPE 3001",
    "PPE 3003",
    "PSYC 2750",
    "STAT 1020",
    "STAT 1120",
];

fn psyc_elective_cognate_slot() -> Requirement {
    any_of(
        "Psychology Electives",
        vec![
            code(PSYC_COGNATE_ELECTIVES),
            restriction(1)
                .departments(&["PSYC"])
                .level(1000)
                .max_level(4000)
                .excluding(&["PSYC 4997"])
                .into(),
            restriction(1)
                .attr(&["APMR"])
                .excluding(&["PSYC 4997"])
                .into(),
        ],
    )
}

fn psyc_major_requirements() -> Vec<Requirement> {
    let psyc_elective: Requirement = restriction(1)
        .category("Psychology Electives")
        .departments(&["PSYC"])
        .level(1000)
        .max_level(4000)
        .excluding(&["PSYC 4997"])
        .into();

    let mut requirements = vec![
        any_of(
            "Introductory Psychology",
            vec![
                code(&["PSYC 0001"]),
                restriction(1)
                    .departments(&["PSYC"])
                    .level(1000)
                    .max_level(4999)
                    .into(),
            ],
        ),
        single(
            "Biological Basis of Behavior",
            &["NRSC 1110", "PSYC 1210", "PSYC 1230"],
        ),
        attr_restriction("Biological Basis of Behavior", "APCI"),
        single(
            "Cognitive Basis of Behavior",
            &["PSYC 1310", "PSYC 1333", "PSYC 1340", "PSYC 1777"],
        ),
        attr_restriction("Cognitive Basis of Behavior", "APCC"),
        single(
            "Social Science Bases of Behavior",
            &["PSYC 1440", "PSYC 1462", "PSYC 1777"],
        ),
        attr_restriction("Social Science Bases of Behavior", "APCS"),
        restriction(1)
            .category("Research Experience")
            .departments(&["PSYC"])
            .level(4000)
            .max_level(4999)
            .excluding(&["PSYC 4997"])
            .into(),
        single("Statistics", PSYC_APPROVED_STATISTICS),
    ];
    requirements.extend([
        psyc_elective_cognate_slot(),
        psyc_elective_cognate_slot(),
        psyc_elective.clone(),
        psyc_elective,
    ]);
    requirements
}

pub fn create_psyc_major() -> Major {
    let schedule_hints = HashMap::from([
        ("PSYC 0001".to_string(), Y1F.into()),
        ("PSYC 1210".to_string(), Y2F.into()),
        ("PSYC 1230".to_string(), Y2F.into()),
        ("PSYC 1310".to_string(), Y2F.into()),
        ("PSYC 1333".to_string(), Y2F.into()),
        ("PSYC 1340".to_string(), Y2S.into()),
        ("PSYC 1440".to_string(), Y2S.into()),
        ("PSYC 1462".to_string(), Y3F.into()),
        ("PSYC 1777".to_string(), Y3F.into()),
        ("STAT 1110".to_string(), Y2S.into()),
        ("STAT 1120".to_string(), Y2S.into()),
        ("STAT 4310".to_string(), Y3S.into()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "PSYC".to_string(),
        name: "Psychology".to_string(),
        major_requirements: psyc_major_requirements(),
        auto_completed_sectors: vec![],
        concentrations: None,
        schedule_hints,
    })
}

const DSGN_INTEGRATIVE_STUDIOS: &[&str] = &[
    "DSGN 1011",
    "DSGN 1020",
    "DSGN 1070",
    "DSGN 1200",
    "DSGN 2040",
    "DSGN 2070",
    "DSGN 2220",
    "DSGN 2230",
    "DSGN 2260",
    "DSGN 2510",
    "DSGN 2530",
    "DSGN 2550",
    "DSGN 2570",
    "DSGN 2580",
];

fn dsgn_theory_slot() -> Requirement {
    any_of(
        "Design Theory",
        vec![
            restriction(1)
                .departments(&["DSGN"])
                .level(3000)
                .max_level(3999)
                .into(),
            restriction(1)
                .departments(&["FNAR"])
                .level(3000)
                .max_level(3999)
                .into(),
            restriction(1).attr(&["ADTH"]).into(),
        ],
    )
}

fn dsgn_art_design_elective_slot() -> Requirement {
    any_of(
        "Art and Design Elective",
        vec![
            restriction(1).departments(&["DSGN"]).into(),
            restriction(1).departments(&["FNAR"]).into(),
            restriction(1).attr(&["ADEL"]).into(),
        ],
    )
}

fn dsgn_major_requirements() -> Vec<Requirement> {
    vec![
        single("Core Studio", &["DSGN 0010"]),
        single("Core Studio", &["DSGN 0020"]),
        single("Integrative Design Studio", DSGN_INTEGRATIVE_STUDIOS),
        single("Integrative Design Studio", DSGN_INTEGRATIVE_STUDIOS),
        single("Integrative Design Studio", DSGN_INTEGRATIVE_STUDIOS),
        any_of(
            "Art History",
            vec![
                restriction(1)
                    .departments(&["ARTH"])
                    .max_level(4999)
                    .into(),
                restriction(1).attr(&["ADAH"]).into(),
            ],
        ),
        dsgn_theory_slot(),
        dsgn_theory_slot(),
        single("Design Senior Seminar", &["DSGN 4020"]),
        single("Design Senior Seminar", &["DSGN 4030"]),
        dsgn_art_design_elective_slot(),
        dsgn_art_design_elective_slot(),
        dsgn_art_design_elective_slot(),
        dsgn_art_design_elective_slot(),
    ]
}

pub fn create_dsgn_major() -> Major {
    let schedule_hints = HashMap::from([
        ("DSGN 0010".to_string(), Y1F.into()),
        ("DSGN 0020".to_string(), Y1S.into()),
        ("DSGN 4020".to_string(), Y4F.into()),
        ("DSGN 4030".to_string(), Y4S.into()),
    ]);
    create_cas_major(CasMajorConfig {
        short_name: "DSGN".to_string(),
        name: "Design".to_string(),
        major_requirements: dsgn_major_requirements(),
        auto_completed_sectors: vec![],
        concentrations: None,
        schedule_hints,
    })
}

fn hsoc_hsoc_stsc_elective_slot() -> Requirement {
    any_of(
        "HSOC or STSC Electives",
        vec![
            restriction(1).departments(&["HSOC"]).into(),
            restriction(1).departments(&["STSC"]).into(),
            restriction(1).attr(&["AHSM"]).into(),
        ],
    )
}

fn hsoc_capstone_slot() -> Requirement {
    any_of(
        "Capstone Research Requirement",
        vec![
            restriction(1).departments(&["HSOC"]).level(4000).into(),
            restriction(1).departments(&["STSC"]).level(4000).into(),
        ],
    )
}

fn hsoc_major_requirements() -> Vec<Requirement> {
    let mut requirements = vec![
        single("Foundation Requirement", &["HSOC 0480", "HSOC 0490"]),
        single(
            "Quantitative Methods",
            &["HSOC 2002", "SOCI 2000", "HSOC 2202", "SOCI 2220"],
        ),
        single(
            "Core Discipline",
            &["HSOC 0400", "HSOC 1411", "HSOC 1401"],
        ),
        single("Core Discipline", &["HSOC 1382", "HSOC 1222"]),
    ];
    requirements.extend(repeat_req(&hsoc_hsoc_stsc_elective_slot(), 3));
    requirements.push(hsoc_capstone_slot());
    requirements
}

fn hsoc_bioethics_concentration_requirements() -> Vec<Requirement> {
    vec![
        attr_restriction("Bioethics & Society - Core Discipline", "AHSB"),
        attr_restriction("Bioethics & Society - Core Course", "AHSI"),
        attr_restriction("Bioethics & Society - Philosophical & Religious Foundations", "AHSP"),
        attr_restriction("Bioethics & Society - Social & Institutional Contexts", "AHSS"),
        attr_restriction(
            "Bioethics & Society - Technologies, Practices & Practitioners",
            "AHST",
        ),
        attr_restriction("Bioethics & Society - Law, Politics & Public Policy", "AHSL"),
    ]
}

fn hsoc_disease_culture_concentration_requirements() -> Vec<Requirement> {
    vec![restriction(6)
        .category("Disease & Culture")
        .attr(&["AHSD"])
        .into()]
}

fn hsoc_global_health_concentration_requirements() -> Vec<Requirement> {
    vec![
        attr_restriction("Global Health - Core Course on World Health", "AHSW"),
        attr_restriction("Global Health - Regional Course", "AHSR"),
        restriction(4)
            .category("Global Health - Concentration Electives")
            .attr(&["AHSG"])
            .into(),
    ]
}

fn hsoc_health_care_markets_finance_concentration_requirements() -> Vec<Requirement> {
    vec![
        attr_restriction("Health Care Markets & Finance - Core Course", "AHFC"),
        restriction(5)
            .category("Health Care Markets & Finance")
            .attr(&["AHFI"])
            .into(),
    ]
}

fn hsoc_health_policy_law_concentration_requirements() -> Vec<Requirement> {
    vec![
        attr_restriction("Health Policy & Law - Political Economy", "AHSO"),
        attr_restriction("Health Policy & Law - Health Policy", "AHSC"),
        attr_restriction("Health Policy & Law - Law & Society", "AHSA"),
        attr_restriction("Health Policy & Law - Philosophical/Ethical", "AHSH"),
        restriction(2)
            .category("Health Policy & Law - Concentration Electives")
            .attr(&["AHSE"])
            .into(),
    ]
}

fn hsoc_public_health_concentration_requirements() -> Vec<Requirement> {
    vec![
        attr_restriction("Public Health - Core Course", "AHPH"),
        restriction(5)
            .category("Public Health Electives")
            .attr(&["AHPE"])
            .into(),
    ]
}

fn hsoc_race_gender_health_concentration_requirements() -> Vec<Requirement> {
    vec![restriction(6)
        .category("Race, Gender, and Health")
        .attr(&["AHSN"])
        .into()]
}

fn hsoc_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    BTreeMap::from([
        (
            "Bioethics and Society".to_string(),
            hsoc_bioethics_concentration_requirements(),
        ),
        (
            "Disease and Culture".to_string(),
            hsoc_disease_culture_concentration_requirements(),
        ),
        (
            "Global Health".to_string(),
            hsoc_global_health_concentration_requirements(),
        ),
        (
            "Health Care Markets & Finance".to_string(),
            hsoc_health_care_markets_finance_concentration_requirements(),
        ),
        (
            "Health Policy & Law".to_string(),
            hsoc_health_policy_law_concentration_requirements(),
        ),
        (
            "Public Health".to_string(),
            hsoc_public_health_concentration_requirements(),
        ),
        (
            "Race, Gender, and Health".to_string(),
            hsoc_race_gender_health_concentration_requirements(),
        ),
    ])
}

fn hsoc_concentration_requirement(concentration_name: &str) -> Option<Requirement> {
    let requirements = hsoc_concentrations()
        .get(concentration_name)
        .unwrap_or_else(|| panic!("unknown HSOC concentration: {concentration_name}"))
        .clone();
    if requirements.is_empty() {
        return None;
    }
    let number = requirements.iter().map(requirement_slot_cu).sum();
    Some(concentration(concentration_name, number, requirements))
}

pub fn hsoc_concentration_names() -> Vec<String> {
    cas_concentration_names("HSOC")
}

pub fn create_hsoc_major(concentration_name: String) -> Major {
    let schedule_hints = HashMap::from([
        ("HSOC 0480".to_string(), Y1F.into()),
        ("HSOC 0490".to_string(), Y1F.into()),
        ("HSOC 0400".to_string(), Y2F.into()),
        ("HSOC 1382".to_string(), Y2F.into()),
        ("HSOC 1222".to_string(), Y2F.into()),
        ("HSOC 2002".to_string(), Y3F.into()),
        ("HSOC 2202".to_string(), Y3F.into()),
    ]);
    let mut major_requirements = hsoc_major_requirements();
    if let Some(conc) = hsoc_concentration_requirement(&concentration_name) {
        major_requirements.push(conc);
    }
    create_cas_major(CasMajorConfig {
        short_name: "HSOC".to_string(),
        name: "Health and Societies".to_string(),
        major_requirements,
        auto_completed_sectors: vec![],
        concentrations: Some(hsoc_concentrations()),
        schedule_hints,
    })
}

const MATH_BIO_ELECTIVE_COURSES: &[&str] = &["BIOL 2210", "BIOL 2410", "BIOL 2610"];
const MATH_BIO_ADVANCED_COURSES: &[&str] = &["BIOL 4517", "BIOL 4231", "BIOL 4536", "BIOL 5536"];

fn math_bio_additional_science() -> Requirement {
    any_of(
        "Biological Mathematics",
        vec![
            all_of(
                None,
                vec![code(&["CHEM 1011"]), code(&["CHEM 1101"])],
            ),
            all_of(
                None,
                vec![code(&["CHEM 1011"]), code(&["CHEM 1102"])],
            ),
            code(&["CHEM 1151"]),
            code(&["PHYS 0151"]),
        ],
    )
}

fn math_biological_mathematics_requirements() -> Vec<Requirement> {
    let advanced_bio = single("Advanced Biology", MATH_BIO_ADVANCED_COURSES);
    vec![
        any_of(
            "Biological Mathematics",
            vec![
                all_of(
                    None,
                    vec![
                        code(&["BIOL 1121"]),
                        code(&["BIOL 1124"]),
                        advanced_bio.clone(),
                        advanced_bio.clone(),
                        advanced_bio,
                    ],
                ),
                all_of(
                    None,
                    vec![
                        code(&["BIOL 1101"]),
                        code(&["BIOL 1102"]),
                        code(&["BIOL 4231"]),
                    ],
                ),
            ],
        ),
        single("Biological Mathematics", MATH_BIO_ELECTIVE_COURSES),
        single("Biological Mathematics", MATH_BIO_ELECTIVE_COURSES),
        math_bio_additional_science(),
    ]
}

fn math_calculus_1400() -> Requirement {
    single("Calculus", &["MATH 1400"])
}

fn math_calculus_1410_or_1610() -> Requirement {
    any_of(
        "Calculus",
        vec![code(&["MATH 1410"]), code(&["MATH 1610"])],
    )
}

fn math_algebra_requirement() -> Requirement {
    any_of(
        "Algebra",
        vec![
            all_of(
                None,
                vec![code(&["MATH 3700"]), code(&["MATH 3710"])],
            ),
            all_of(
                None,
                vec![code(&["MATH 5020"]), code(&["MATH 5030"])],
            ),
        ],
    )
}

fn math_analysis_requirement() -> Requirement {
    any_of(
        "Analysis",
        vec![
            all_of(
                None,
                vec![code(&["MATH 3600"]), code(&["MATH 3610"])],
            ),
            all_of(
                None,
                vec![code(&["MATH 5080"]), code(&["MATH 5090"])],
            ),
        ],
    )
}

fn math_differential_equations_requirement() -> Requirement {
    any_of(
        "Differential Equations",
        vec![
            code(&["MATH 2300"]),
            code(&["MATH 4200"]),
            code(&["MATH 4250"]),
        ],
    )
}

fn math_elective_restriction(departments: Option<&[&str]>, attrs: Option<&[&str]>) -> Requirement {
    let mut b = restriction(1);
    if let Some(depts) = departments {
        b = b.departments(depts).level(3000);
    }
    if let Some(a) = attrs {
        b = b.attr(a);
    }
    b.into()
}

fn math_elective_slot(allow_cognate: bool) -> Requirement {
    let mut possibilities = vec![
        math_elective_restriction(Some(&["MATH"]), None),
        math_elective_restriction(None, Some(&["AMMR"])),
    ];
    if allow_cognate {
        possibilities.push(math_elective_restriction(None, Some(&["AMOR"])));
    }
    any_of("Mathematics Electives", possibilities)
}

fn math_general_elective_slots(count: i32) -> Vec<Requirement> {
    (0..count)
        .map(|i| math_elective_slot(i == 0))
        .collect()
}

fn math_general_mathematics_requirements() -> Vec<Requirement> {
    let mut requirements = vec![
        math_calculus_1400(),
        math_calculus_1410_or_1610(),
        single("Complex Analysis", &["MATH 4100"]),
        single("Advanced Linear Algebra", &["MATH 3000"]),
        single("Advanced Linear Algebra", &["MATH 3001"]),
        math_differential_equations_requirement(),
        math_algebra_requirement(),
        math_analysis_requirement(),
    ];
    requirements.extend(math_general_elective_slots(3));
    requirements
}

fn math_biological_mathematics_math_requirements() -> Vec<Requirement> {
    vec![
        math_calculus_1400(),
        math_calculus_1410_or_1610(),
        single("Advanced Linear Algebra", &["MATH 3000"]),
        single("Advanced Linear Algebra", &["MATH 3001"]),
        math_analysis_requirement(),
        math_algebra_requirement(),
        single("Statistics", &["MATH 3200"]),
        single("Statistics", &["STAT 4310"]),
        math_differential_equations_requirement(),
    ]
}

fn math_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    BTreeMap::from([
        ("General Mathematics".to_string(), vec![]),
        (
            "Biological Mathematics".to_string(),
            math_biological_mathematics_requirements(),
        ),
    ])
}

fn math_concentration_requirement(concentration_name: &str) -> Option<Requirement> {
    let requirements = math_concentrations()
        .get(concentration_name)
        .unwrap_or_else(|| panic!("unknown MATH concentration: {concentration_name}"))
        .clone();
    if requirements.is_empty() {
        return None;
    }
    let number = requirements.iter().map(requirement_slot_cu).sum();
    Some(concentration(concentration_name, number, requirements))
}

pub fn math_concentration_names() -> Vec<String> {
    cas_concentration_names("MATH")
}

pub fn create_math_major(concentration_name: String) -> Major {
    let schedule_hints = HashMap::from([
        ("MATH 1400".to_string(), Y1F.into()),
        ("MATH 1410".to_string(), Y1S.into()),
        ("MATH 1610".to_string(), Y1S.into()),
        ("MATH 3000".to_string(), Y2F.into()),
        ("MATH 3001".to_string(), Y2S.into()),
        ("MATH 3200".to_string(), Y3F.into()),
        ("STAT 4310".to_string(), Y3S.into()),
        ("MATH 3600".to_string(), Y3F.into()),
        ("MATH 3610".to_string(), Y4F.into()),
        ("MATH 5080".to_string(), Y3F.into()),
        ("MATH 5090".to_string(), Y4F.into()),
        ("MATH 3700".to_string(), Y3S.into()),
        ("MATH 3710".to_string(), Y4F.into()),
        ("MATH 5020".to_string(), Y3S.into()),
        ("MATH 5030".to_string(), Y4F.into()),
        ("MATH 4100".to_string(), Y3F.into()),
        ("MATH 2300".to_string(), Y3F.into()),
        ("MATH 4200".to_string(), Y3F.into()),
        ("MATH 4250".to_string(), Y3F.into()),
        ("BIOL 1121".to_string(), Y1F.into()),
        ("BIOL 1124".to_string(), Y1S.into()),
        ("BIOL 1101".to_string(), Y1F.into()),
        ("BIOL 1102".to_string(), Y1S.into()),
        ("BIOL 4231".to_string(), Y3F.into()),
        ("BIOL 2210".to_string(), Y2F.into()),
        ("BIOL 2410".to_string(), Y2S.into()),
        ("BIOL 2610".to_string(), Y3F.into()),
        ("CHEM 1011".to_string(), Y2F.into()),
        ("CHEM 1101".to_string(), Y2F.into()),
        ("CHEM 1102".to_string(), Y2S.into()),
        ("PHYS 0151".to_string(), Y2S.into()),
    ]);
    let mut major_requirements = if concentration_name == "General Mathematics" {
        math_general_mathematics_requirements()
    } else {
        math_biological_mathematics_math_requirements()
    };
    if let Some(conc) = math_concentration_requirement(&concentration_name) {
        major_requirements.push(conc);
    }
    create_cas_major(CasMajorConfig {
        short_name: "MATH".to_string(),
        name: "Mathematics".to_string(),
        major_requirements,
        auto_completed_sectors: vec![],
        concentrations: Some(math_concentrations()),
        schedule_hints,
    })
}

fn math_minor_calculus_requirements() -> Vec<Requirement> {
    vec![
        any_of(
            "Calculus",
            vec![code(&["MATH 1400"]), code(&["MATH 1070"])],
        ),
        any_of(
            "Calculus",
            vec![
                code(&["MATH 1410"]),
                code(&["MATH 1080"]),
                code(&["MATH 1610"]),
            ],
        ),
    ]
}

fn math_minor_linear_algebra_and_proofs() -> Requirement {
    any_of(
        "Linear Algebra and Intro to Proofs",
        vec![
            code(&["MATH 3000"]),
            all_of(
                None,
                vec![
                    single(
                        "Linear Algebra",
                        &["MATH 2200", "ESE 2030", "CIS 5150"],
                    ),
                    single(
                        "Introduction to Proofs",
                        &["MATH 2030", "MATH 1610", "CIS 1600"],
                    ),
                ],
            ),
        ],
    )
}

fn math_minor_proof_based() -> Requirement {
    restriction(1)
        .category("Proof Based Math")
        .departments(&["MATH"])
        .level(3000)
        .max_level(5999)
        .into()
}

fn math_minor_elective_restriction(
    departments: Option<&[&str]>,
    attrs: Option<&[&str]>,
) -> Requirement {
    let mut b = restriction(1);
    if let Some(depts) = departments {
        b = b.departments(depts).level(2000);
    }
    if let Some(a) = attrs {
        b = b.attr(a);
    }
    b.into()
}

fn math_minor_elective_slot(allow_amor: bool) -> Requirement {
    let mut possibilities = vec![
        math_minor_elective_restriction(Some(&["MATH"]), None),
        math_minor_elective_restriction(None, Some(&["AMMR"])),
    ];
    if allow_amor {
        possibilities.push(math_minor_elective_restriction(None, Some(&["AMOR"])));
    }
    any_of("Mathematics Electives", possibilities)
}

fn math_minor_elective_slots(count: i32) -> Vec<Requirement> {
    (0..count)
        .map(|i| math_minor_elective_slot(i == 0))
        .collect()
}

/// Mathematics minor (7–8 CU) per Penn catalog.
pub fn create_math_minor() -> Major {
    let mut requirements = math_minor_calculus_requirements();
    requirements.push(math_minor_linear_algebra_and_proofs());
    requirements.push(math_minor_proof_based());
    requirements.extend(math_minor_elective_slots(3));

    Major {
        short_name: "MATH".to_string(),
        name: "Mathematics".to_string(),
        requirements,
        concentrations: None,
        schedule_hints: HashMap::new(),
    }
}
