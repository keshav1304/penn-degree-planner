use std::collections::HashMap;

use crate::Major;
use crate::Requirement;
use crate::penn_data::requirement_builders::{any_of, code, restriction, single};
use crate::schedule_template::{
    insert_fixed_course_hints, scheduled, ScheduleHint, Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F,
    Y4S,
};

// --- Shared dept slices ---

const PLANET_SECTOR_DEPTS: &[&str] = &["ASTR", "BIOL", "EESC", "ENVS"];

const SOCIETIES_SECTOR_DEPTS: &[&str] = &[
    "ANCH", "ANEL", "ANTH", "ARTH", "CLST", "COMM", "CRIM", "EALC", "ECON", "EDUC", "HIST",
    "MMES", "NELC", "PPE", "PSCI", "PSYC", "SOCI",
];

const GLOBAL_ARTS_SECTOR_DEPTS: &[&str] = &[
    "ASLD", "AMHR", "ARAB", "ARCH", "ARTH", "ALAN", "BENG", "BCS", "CHIC", "CHIN", "CIMS", "COML",
    "CZCH", "DTCH", "ENGL", "FILP", "FNAR", "FREN", "GRMN", "GREK", "GUJR", "HEBR", "HIND", "HUNG",
    "IGBO", "INDO", "IRIS", "ITAL", "JPAN", "KAND", "KORN", "LATN", "LING", "MALG", "MLYM", "MRTI",
    "MUSC", "PASH", "PERS", "PLSH", "PRTG", "PUNJ", "QUEC", "RUSS", "SKRT", "SPAN", "SARB", "SWAH",
    "SWED", "TAML", "TELU", "THAI", "THAR", "TIBT", "TIGR", "TURK", "TWI", "UKRN", "URDU", "VIET",
    "VLST", "WOLF", "YDSH", "YORB", "ZULU",
];

const UNIVERSALITY_SECTOR_DEPTS: &[&str] =
    &["AFRC", "ASAM", "GSWS", "JWST", "LALS", "RELS", "REES", "SAST", "URBS"];

// --- Domain helpers ---

fn nurs_dept_restriction(label: &str, depts: &[&str], min: i32, max: i32) -> Requirement {
    restriction(1)
        .category(label)
        .departments(depts)
        .level(min)
        .max_level(max)
        .into()
}

fn nurs_dept_sector(label: &str, depts: &[&str], min: i32, max: i32, alternates: &[&str]) -> Requirement {
    let mut possibilities: Vec<Requirement> = alternates.iter().map(|c| code(&[*c])).collect();
    possibilities.push(nurs_dept_restriction(label, depts, min, max));
    any_of(label, possibilities)
}

fn nurs_writing_requirement() -> Requirement {
    any_of(
        "Writing Requirement",
        vec![
            restriction(1).attr(&["AUWR"]).into(),
            restriction(1)
                .departments(&["WRIT"])
                .level(1)
                .max_level(991)
                .into(),
        ],
    )
}

fn nurs_language_slot(label: &str) -> Requirement {
    any_of(
        label,
        vec![
            restriction(1).attr(&["WUFL"]).into(),
            code(&["SPAN 0105", "SPAN 0205"]),
        ],
    )
}

fn nurs_free_elective_slot(label: &str) -> Requirement {
    restriction(1).category(label).into()
}

fn nurs_exploration_requirement() -> Requirement {
    restriction(1)
        .category("Exploration Course Requirement")
        .no_school("NURS")
        .into()
}

fn nurs_case_study_requirement() -> Requirement {
    restriction(1)
        .category("Nursing Case Study")
        .departments(&["NURS"])
        .level(3510)
        .max_level(3690)
        .into()
}

fn nurs_health_policy_requirement() -> Requirement {
    any_of(
        "Health Policy Requirement",
        vec![
            single("Health Policy Requirement", &["NURS 3340"]),
            single("Health Policy Requirement", &["NURS 4000"]),
            single("Health Policy Requirement", &["NURS 5400"]),
        ],
    )
}

fn nurs_ethics_requirement() -> Requirement {
    any_of(
        "Ethics Requirement",
        vec![
            single("Ethics Requirement", &["NURS 3300"]),
            single("Ethics Requirement", &["PHIL 1342"]),
            single("Ethics Requirement", &["NURS 5250"]),
            single("Ethics Requirement", &["BIOE 4010"]),
            single("Ethics Requirement", &["BIOE 4020"]),
        ],
    )
}

fn nurs_nune_elective_slot(n: u8) -> Requirement {
    restriction(1)
        .category(&format!("Nutrition Major Elective (NUNE) {n}"))
        .attr(&["NUNE"])
        .into()
}

fn nurs_duje_sector_requirement() -> Requirement {
    nurs_dept_sector(
        "Diversity, Universality, Justice, & Equity",
        UNIVERSALITY_SECTOR_DEPTS,
        1,
        4999,
        &["NURS 3160"],
    )
}

/// Penn catalog Plan of Study grid (2026–27 Nursing BSN).
fn build_bsn_scheduled(language_required: bool) -> (Vec<Requirement>, HashMap<String, ScheduleHint>) {
    let elective_slot = |n: u8| {
        if language_required {
            nurs_language_slot(&format!("Language Requirement {n}"))
        } else {
            nurs_free_elective_slot(&format!("Free Elective {n}"))
        }
    };

    scheduled(vec![
        // First Year — Fall (4.00 CU)
        (Y1F, single("Science Requirements", &["NURS 0061"])),
        (Y1F, single("Science Requirements", &["NURS 0068"])),
        (Y1F, single("Nursing Foundational Courses", &["NURS 1010"])),
        (Y1F, nurs_writing_requirement()),
        (Y1F, elective_slot(1)),
        // First Year — Spring (5.50 CU)
        (Y1S, single("Science Requirements", &["NURS 0065"])),
        (Y1S, single("Science Requirements", &["NURS 1630"])),
        (Y1S, single("Nursing Foundational Courses", &["NURS 1020"])),
        (
            Y1S,
            nurs_dept_sector(
                "The Planet & Our Climate",
                PLANET_SECTOR_DEPTS,
                1,
                4999,
                &["NURS 3520"],
            ),
        ),
        (Y1S, elective_slot(2)),
        // Second Year — Fall (4.00 CU)
        (Y2F, single("Science Requirements", &["NURS 1640"])),
        (Y2F, single("Nursing Foundational Courses", &["NURS 1030"])),
        (
            Y2F,
            nurs_dept_sector(
                "Societies, Histories, and Traditions",
                SOCIETIES_SECTOR_DEPTS,
                1,
                4999,
                &["NURS 3060", "NURS 3130"],
            ),
        ),
        // Second Year — Spring (4.50 CU)
        (Y2S, single("Science Requirements", &["NURS 1650"])),
        (Y2S, single("Nursing Clinical Courses", &["NURS 2150"])),
        (
            Y2S,
            nurs_dept_sector(
                "Global Arts, Letters, & Cultures",
                GLOBAL_ARTS_SECTOR_DEPTS,
                100,
                4999,
                &["NURS 3050", "NURS 3160"],
            ),
        ),
        // Third Year — Fall (5.00 CU)
        (Y3F, single("Nursing Clinical Courses", &["NURS 2450"])),
        (Y3F, single("Nursing Clinical Courses", &["NURS 2550"])),
        (Y3F, nurs_health_policy_requirement()),
        (Y3F, single("Non-Clinical Courses", &["NURS 2300"])),
        // Third Year — Spring (5.00 CU)
        (Y3S, single("Nursing Clinical Courses", &["NURS 2350"])),
        (Y3S, single("Nursing Clinical Courses", &["NURS 2250"])),
        (Y3S, nurs_ethics_requirement()),
        (Y3S, single("Non-Clinical Courses", &["NURS 5470"])),
        // Fourth Year — Fall (4.50 CU)
        (Y4F, single("Nursing Clinical Courses", &["NURS 3820"])),
        (Y4F, nurs_case_study_requirement()),
        (
            Y4F,
            nurs_dept_sector(
                "Universality in Thought and Action",
                UNIVERSALITY_SECTOR_DEPTS,
                1,
                4999,
                &["NURS 3160"],
            ),
        ),
        (Y4F, nurs_exploration_requirement()),
        // Fourth Year — Spring (3.50 CU)
        (Y4S, single("Nursing Clinical Courses", &["NURS 3900"])),
        (Y4S, single("Non-Clinical Courses", &["NURS 3890"])),
    ])
}

/// Fixed placements for catalog-mandated NURS courses (Plan of Study grid).
fn apply_bsn_catalog_fixed_hints(hints: &mut HashMap<String, ScheduleHint>) {
    insert_fixed_course_hints(
        hints,
        &[
            ("NURS 0061", Y1F),
            ("NURS 0068", Y1F),
            ("NURS 1010", Y1F),
            ("NURS 0065", Y1S),
            ("NURS 1630", Y1S),
            ("NURS 1020", Y1S),
            ("NURS 1640", Y2F),
            ("NURS 1030", Y2F),
            ("NURS 1650", Y2S),
            ("NURS 2150", Y2S),
            ("NURS 2450", Y3F),
            ("NURS 2550", Y3F),
            ("NURS 2300", Y3F),
            ("NURS 2350", Y3S),
            ("NURS 2250", Y3S),
            ("NURS 5470", Y3S),
            ("NURS 3820", Y4F),
            ("NURS 3900", Y4S),
            ("NURS 3890", Y4S),
        ],
    );
}

// --- Majors ---

fn create_bsn_variant(short_name: &str, name: &str, language_required: bool) -> Major {
    let (requirements, mut schedule_hints) = build_bsn_scheduled(language_required);
    apply_bsn_catalog_fixed_hints(&mut schedule_hints);
    Major {
        short_name: short_name.to_string(),
        name: name.to_string(),
        requirements,
        concentrations: None,
        schedule_hints,
    }
}

pub fn create_bsn_major() -> Major {
    create_bsn_variant("BSN", "Nursing, BSN", true)
}

pub fn create_bsn_nofl_major() -> Major {
    create_bsn_variant(
        "BSN_NOFL",
        "Nursing, BSN (Language Exempt)",
        false,
    )
}

/// Penn catalog Plan of Study grid (2026–27 Nutrition Science BSN).
fn build_nutr_bsn_scheduled(language_required: bool) -> (Vec<Requirement>, HashMap<String, ScheduleHint>) {
    let elective_slot = |n: u8| {
        if language_required {
            nurs_language_slot(&format!("Language Requirement {n}"))
        } else {
            nurs_free_elective_slot(&format!("Free Elective {n}"))
        }
    };

    scheduled(vec![
        // First Year — Fall (4.00 CU)
        (Y1F, single("Science Requirements", &["NURS 0061"])),
        (Y1F, single("Science Requirements", &["NURS 0068"])),
        (Y1F, single("Nursing Foundational Courses", &["NURS 1010"])),
        (Y1F, nurs_writing_requirement()),
        (Y1F, elective_slot(1)),
        // First Year — Spring (5.50 CU)
        (Y1S, single("Science Requirements", &["NURS 0065"])),
        (Y1S, single("Science Requirements", &["NURS 1630"])),
        (Y1S, single("Nursing Foundational Courses", &["NURS 1020"])),
        (
            Y1S,
            nurs_dept_sector(
                "The Planet & Our Climate",
                PLANET_SECTOR_DEPTS,
                1,
                4999,
                &["NURS 3520"],
            ),
        ),
        (Y1S, elective_slot(2)),
        // Second Year — Fall (5.00 CU)
        (Y2F, single("Science Requirements", &["NURS 1640"])),
        (Y2F, single("Nursing Foundational Courses", &["NURS 1030"])),
        (
            Y2F,
            nurs_dept_sector(
                "Societies, Histories, and Traditions",
                SOCIETIES_SECTOR_DEPTS,
                1,
                4999,
                &["NURS 3060", "NURS 3130"],
            ),
        ),
        (Y2F, nurs_nune_elective_slot(1)),
        // Second Year — Spring (5.50 CU)
        (Y2S, single("Science Requirements", &["NURS 1650"])),
        (Y2S, single("Nursing Clinical Courses", &["NURS 2150"])),
        (
            Y2S,
            nurs_dept_sector(
                "Global Arts, Letters, & Cultures",
                GLOBAL_ARTS_SECTOR_DEPTS,
                100,
                4999,
                &["NURS 3050", "NURS 3160"],
            ),
        ),
        (Y2S, nurs_nune_elective_slot(2)),
        // Third Year — Fall (6.00 CU)
        (Y3F, single("Nursing Clinical Courses", &["NURS 2450"])),
        (Y3F, single("Nursing Clinical Courses", &["NURS 2550"])),
        (Y3F, nurs_health_policy_requirement()),
        (Y3F, single("Non-Clinical Courses", &["NURS 2300"])),
        (Y3F, nurs_nune_elective_slot(3)),
        // Third Year — Spring (6.00 CU)
        (Y3S, single("Nursing Clinical Courses", &["NURS 2350"])),
        (Y3S, single("Nursing Clinical Courses", &["NURS 2250"])),
        (Y3S, nurs_ethics_requirement()),
        (Y3S, single("Non-Clinical Courses", &["NURS 5470"])),
        (Y3S, single("Required Nutrition Science Courses", &["NURS 5240"])),
        // Fourth Year — Fall (5.50 CU)
        (Y4F, single("Nursing Clinical Courses", &["NURS 3820"])),
        (Y4F, nurs_case_study_requirement()),
        (Y4F, nurs_duje_sector_requirement()),
        (Y4F, nurs_exploration_requirement()),
        (Y4F, single("Required Nutrition Science Courses", &["NURS 5230"])),
        // Fourth Year — Spring (5.50 CU)
        (Y4S, single("Nursing Clinical Courses", &["NURS 3900"])),
        (Y4S, single("Non-Clinical Courses", &["NURS 3890"])),
        (Y4S, single("Required Nutrition Science Courses", &["NURS 3120"])),
        (Y4S, nurs_nune_elective_slot(4)),
    ])
}

fn apply_nutr_bsn_catalog_fixed_hints(hints: &mut HashMap<String, ScheduleHint>) {
    apply_bsn_catalog_fixed_hints(hints);
    insert_fixed_course_hints(
        hints,
        &[
            ("NURS 5240", Y3S),
            ("NURS 5230", Y4F),
            ("NURS 3120", Y4S),
        ],
    );
}

fn create_nutr_bsn_variant(short_name: &str, name: &str, language_required: bool) -> Major {
    let (requirements, mut schedule_hints) = build_nutr_bsn_scheduled(language_required);
    apply_nutr_bsn_catalog_fixed_hints(&mut schedule_hints);
    Major {
        short_name: short_name.to_string(),
        name: name.to_string(),
        requirements,
        concentrations: None,
        schedule_hints,
    }
}

pub fn create_nutr_bsn_major() -> Major {
    create_nutr_bsn_variant("NUTR_BSN", "Nutrition Science, BSN", true)
}

pub fn create_nutr_bsn_nofl_major() -> Major {
    create_nutr_bsn_variant(
        "NUTR_BSN_NOFL",
        "Nutrition Science, BSN (Language Exempt)",
        false,
    )
}
