use std::collections::BTreeMap;

use serde::Serialize;

use crate::Requirement;
use crate::seas_data;
use crate::wharton_data;

#[derive(Debug)]
pub struct Major {
    pub short_name: String,
    pub name: String,
    pub requirements: Vec<Requirement>,
    pub concentrations: Option<BTreeMap<String, Vec<Requirement>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MajorCatalogEntry {
    pub display_name: String,
    pub api_code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchoolCatalogEntry {
    pub school_code: String,
    pub display_name: String,
    pub majors: Vec<MajorCatalogEntry>,
}

/// Canonical school/major list for the UI and `/all_majors`.
pub fn degree_catalog() -> Vec<SchoolCatalogEntry> {
    vec![
        SchoolCatalogEntry {
            school_code: "CAS".to_string(),
            display_name: "College of Arts and Sciences (CAS)".to_string(),
            majors: vec![MajorCatalogEntry {
                display_name: "Not implemented (NA)".to_string(),
                api_code: "NA".to_string(),
            }],
        },
        SchoolCatalogEntry {
            school_code: "SEAS".to_string(),
            display_name: "School of Engineering and Applied Science (SEAS)".to_string(),
            majors: vec![
                MajorCatalogEntry {
                    display_name: "Electrical Engineering (EE)".to_string(),
                    api_code: "EE".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Computer Science, BSE (CIS)".to_string(),
                    api_code: "CIS".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Mechanical Engineering and Applied Mechanics (MEAM)".to_string(),
                    api_code: "MEAM".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Material Science and Engineering (MSE)".to_string(),
                    api_code: "MSE".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Artificial Intelligence (AI)".to_string(),
                    api_code: "AI".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Computer Engineering (CE)".to_string(),
                    api_code: "CE".to_string(),
                },
            ],
        },
        SchoolCatalogEntry {
            school_code: "WH".to_string(),
            display_name: "The Wharton School (WH)".to_string(),
            majors: vec![
                MajorCatalogEntry {
                    display_name: "Foreign Language Required (FL)".to_string(),
                    api_code: "WH_FL".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Foreign Language Exempt (NO_FL)".to_string(),
                    api_code: "WH_NOFL".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "M&T - Foreign Language Exempt (NOFL_MT)".to_string(),
                    api_code: "WH_NOFL_MT".to_string(),
                },
            ],
        },
        SchoolCatalogEntry {
            school_code: "NURS".to_string(),
            display_name: "School of Nursing (NURS)".to_string(),
            majors: vec![MajorCatalogEntry {
                display_name: "Not implemented (NA)".to_string(),
                api_code: "NA".to_string(),
            }],
        },
    ]
}

pub fn all_majors() -> BTreeMap<String, Vec<String>> {
    degree_catalog()
        .into_iter()
        .map(|school| {
            (
                school.display_name,
                school.majors.into_iter().map(|m| m.display_name).collect(),
            )
        })
        .collect()
}

/// Returns concentration options for the UI. Overlay-style majors (EE, MSE) include "None".
pub fn concentrations_for(school: &str, major: &str) -> Vec<String> {
    let optional_overlay = school == "SEAS" && matches!(major, "EE" | "MSE");

    let mut names = match school {
        "SEAS" => seas_data::concentration_names_for(major),
        "WH" if matches!(major, "WH_FL" | "WH_NOFL" | "WH_NOFL_MT") => {
            wharton_data::concentration_names()
        }
        _ => vec![],
    };

    if optional_overlay && !names.is_empty() {
        names.insert(0, "None".to_string());
    }

    names
}

pub fn all_concentrations() -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();

    for (school, majors) in [
        ("SEAS", vec!["EE", "MEAM", "MSE", "CIS", "AI", "CE"]),
        ("WH", vec!["WH_FL", "WH_NOFL", "WH_NOFL_MT"]),
    ] {
        for major in majors {
            let concs = concentrations_for(school, major);
            if !concs.is_empty() {
                map.insert(format!("{school}:{major}"), concs);
            }
        }
    }

    map
}

pub fn resolve_major(school: &str, major: &str, concentrations: &[String]) -> Option<Major> {
    match school {
        "SEAS" => {
            match major {
                "EE" => Some(seas_data::create_ee_major()),
                "MEAM" => {
                    let conc = concentrations
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "General".to_string());
                    Some(seas_data::create_meam_major(conc))
                },
                "MSE" => Some(seas_data::create_mse_major()),
                "CIS" => Some(seas_data::create_cis_major()),
                "AI" => Some(seas_data::create_ai_major()),
                "CE" => Some(seas_data::create_compe_major()),
                "CBE" => None,
                _ => None,
            }
        },
        "WH" => {
            let concs = wharton_data::normalize_wh_concentrations(concentrations);
            let concs = if concs.is_empty() {
                vec!["FNCE".to_string()]
            } else {
                concs
            };
            match major {
                "WH_NOFL" => Some(wharton_data::create_wh_nofl_major(concs)),
                "WH_FL" => Some(wharton_data::create_wh_fl_major(concs)),
                "WH_NOFL_MT" => Some(wharton_data::create_wh_nofl_mt_major(concs)),
                _ => None,
            }
        },
        "NURS" => None,
        "CAS" => None,
        _ => None,
    }
}