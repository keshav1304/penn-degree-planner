use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::Requirement;
use crate::college_data;
use crate::seas_data;
use crate::seas_grad_data;
use crate::wharton_data;

#[derive(Debug)]
pub struct Major {
    pub short_name: String,
    pub name: String,
    pub requirements: Vec<Requirement>,
    pub concentrations: Option<BTreeMap<String, Vec<Requirement>>>,
    /// Maps top-level requirement index (`"0"`, `"1"`, …) → target `(year, semester)`.
    pub schedule_hints: HashMap<String, (i32, String)>,
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
            display_name: "College of Arts and Sciences".to_string(),
            majors: vec![
                MajorCatalogEntry {
                    display_name: "Economics".to_string(),
                    api_code: "ECON".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Mathematical Economics".to_string(),
                    api_code: "MECON".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Computer Science (2nd major only)".to_string(),
                    api_code: "CIS".to_string(),
                },
            ],
        },
        SchoolCatalogEntry {
            school_code: "SEAS".to_string(),
            display_name: "SEAS Undergraduate".to_string(),
            majors: vec![
                MajorCatalogEntry {
                    display_name: "Electrical Engineering".to_string(),
                    api_code: "EE".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Computer Science, BSE".to_string(),
                    api_code: "CIS".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Mechanical Engineering and Applied Mechanics".to_string(),
                    api_code: "MEAM".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Materials Science and Engineering".to_string(),
                    api_code: "MSE".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Artificial Intelligence".to_string(),
                    api_code: "AI".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Computer Engineering".to_string(),
                    api_code: "CMPE".to_string(),
                },
            ],
        },
        SchoolCatalogEntry {
            school_code: "SEAS_MS".to_string(),
            display_name: "SEAS Masters".to_string(),
            majors: vec![
                MajorCatalogEntry {
                    display_name: "Electrical Engineering, MSE".to_string(),
                    api_code: "MS_EE".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Robotics, MSE".to_string(),
                    api_code: "MS_ROBO".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Mechanical Engineering and Applied Mechanics, MSE".to_string(),
                    api_code: "MS_MEAM".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Computer Science, MSE".to_string(),
                    api_code: "MS_CIS".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Materials Science and Engineering, MSE".to_string(),
                    api_code: "MS_MSE".to_string(),
                },
            ],
        },
        SchoolCatalogEntry {
            school_code: "WH".to_string(),
            display_name: "The Wharton School".to_string(),
            majors: vec![
                MajorCatalogEntry {
                    display_name: "Foreign Language Required".to_string(),
                    api_code: "WH_FL".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "Foreign Language Exempt".to_string(),
                    api_code: "WH_NOFL".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "M&T - Foreign Language Exempt".to_string(),
                    api_code: "WH_NOFL_MT".to_string(),
                },
                MajorCatalogEntry {
                    display_name: "M&T - Foreign Language Required".to_string(),
                    api_code: "WH_FL_MT".to_string(),
                },
            ],
        },
        SchoolCatalogEntry {
            school_code: "NURS".to_string(),
            display_name: "School of Nursing".to_string(),
            majors: vec![MajorCatalogEntry {
                display_name: "Not implemented".to_string(),
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

/// Normalize concentration list for a school (e.g. Wharton allows up to two).
pub fn normalize_degree_concentrations(school: &str, concentrations: &[String]) -> Vec<String> {
    if school == "WH" {
        wharton_data::normalize_wh_concentrations(concentrations)
    } else {
        concentrations.to_vec()
    }
}

/// Returns concentration options for the UI. Overlay-style majors (EE, MSE) include "None".
pub fn concentrations_for(school: &str, major: &str) -> Vec<String> {
    let optional_overlay = school == "SEAS" && matches!(major, "EE" | "MSE" | "CIS" | "CMPE");

    let mut names = match school {
        "SEAS" => seas_data::concentration_names_for(major),
        "WH" if matches!(major, "WH_FL" | "WH_NOFL" | "WH_NOFL_MT" | "WH_FL_MT") => {
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
        ("SEAS", vec!["EE", "MEAM", "MSE", "CIS", "AI", "CMPE"]),
        ("WH", vec!["WH_FL", "WH_NOFL", "WH_NOFL_MT", "WH_FL_MT"]),
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
    let major = match school {
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
                "CMPE" => Some(seas_data::create_cmpe_major()),
                "CBE" => None,
                _ => None,
            }
        },
        "SEAS_MS" => {
            match major {
                "MS_EE" => Some(seas_grad_data::create_ms_ee_major()),
                "MS_ROBO" => Some(seas_grad_data::create_ms_robo_major()),
                "MS_MEAM" => Some(seas_grad_data::create_ms_meam_major()),
                "MS_CIS" => Some(seas_grad_data::create_ms_cis_major()),
                "MS_MSE" => Some(seas_grad_data::create_ms_mse_major()),
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
                "WH_FL_MT" => Some(wharton_data::create_wh_fl_mt_major(concs)),
                _ => None,
            }
        },
        "NURS" => None,
        "CAS" => match major {
            "ECON" => Some(college_data::create_econ_major()),
            "MECON" => Some(college_data::create_mathecon_major()),
            "CIS" => Some(college_data::create_cis_cas_major()),
            _ => None,
        },
        _ => None,
    };
    major.map(normalize_major)
}

fn normalize_major(major: Major) -> Major {
    Major {
        requirements: crate::requirement::expand_restriction_slots(major.requirements),
        concentrations: major.concentrations.map(|map| {
            map.into_iter()
                .map(|(name, requirements)| {
                    (name, crate::requirement::expand_restriction_slots(requirements))
                })
                .collect()
        }),
        ..major
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_major;

    #[test]
    fn resolves_ms_robo() {
        assert!(resolve_major("SEAS_MS", "MS_ROBO", &[]).is_some());
    }

    #[test]
    fn wh_nofl_mt_stat_has_business_breadth() {
        let major = resolve_major("WH", "WH_NOFL_MT", &["STAT".to_string()])
            .expect("WH_NOFL_MT");
        let bb_count = major
            .requirements
            .iter()
            .filter(|r| r.get_category().to_lowercase().contains("business breadth"))
            .count();
        assert_eq!(bb_count, 1, "requirements: {:?}", 
            major.requirements.iter().map(|r| r.get_category()).collect::<Vec<_>>());
    }
}