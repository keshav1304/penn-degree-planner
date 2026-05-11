use std::collections::{BTreeMap, HashMap};

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

pub fn all_majors () -> BTreeMap<String, Vec<String>> {
    BTreeMap::from([
        ("College of Arts and Sciences (CAS)".to_string(), ["Not implemented (NA)"].map(String::from).to_vec()),
        ("School of Engineering and Applied Science (SEAS)".to_string(), ["Electrical Engineering (EE)", "Computer Science, BSE (CIS)", "Mechanical Engineering and Applied Mechanics (MEAM)", "Material Science and Engineering (MSE)", "Artificial Intelligence (AI)", "Computer Engineering (CE)"].map(String::from).to_vec()),
        ("The Wharton School (WH)".to_string(), ["Foreign Language Required (FL)", "Foreign Language Exempt (NO_FL)", "M&T - Foreign Language Exempt (NOFL_MT)"].map(String::from).to_vec()),
        ("School of Nursing (NURS)".to_string(), ["Not implemented (NA)"].map(String::from).to_vec()),
    ])
}

pub fn resolve_major(school: &str, major: &str, concentration: &Option<String>) -> Option<Major> {
    match school {
        "SEAS" => {
            match major {
                "EE" => Some(seas_data::create_ee_major()),
                "MEAM" => {
                    let conc = concentration.clone().unwrap_or("General".to_string());
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
            let conc = concentration.clone().unwrap_or("FNCE".to_string());
            match major {
                "WH_NOFL" => Some(wharton_data::create_wh_nofl_major(conc)),
                "WH_FL" => Some(wharton_data::create_wh_fl_major(conc)),
                "WH_NOFL_MT" => Some(wharton_data::create_wh_nofl_mt_major(conc)),
                _ => None,
            }
        },
        "NURS" => None,
        "CAS" => None,
        _ => None,
    }
}

/// Returns a mapping of API major code → available concentration names.
/// Each entry is keyed by "SCHOOL_MAJOR" (e.g. "SEAS_EE", "WH_WH_FL").
/// Concentrations are extracted from the Major.concentrations BTreeMap.
/// An "is_core" flag is prepended: core concentrations (Requirement::Concentration in
/// requirements) are mandatory; overlay concentrations are optional (None is a valid choice).
pub fn all_concentrations() -> BTreeMap<String, ConcentrationMeta> {
    let mut result = BTreeMap::new();

    // All known school/major pairs with their default concentrations for resolving
    let entries: Vec<(&str, &str, Option<String>)> = vec![
        ("SEAS", "EE", None),
        ("SEAS", "MEAM", Some("General".to_string())),
        ("SEAS", "MSE", None),
        ("SEAS", "CIS", None),
        ("SEAS", "AI", None),
        ("SEAS", "CE", None),
        ("WH", "WH_FL", Some("FNCE".to_string())),
        ("WH", "WH_NOFL", Some("FNCE".to_string())),
        ("WH", "WH_NOFL_MT", Some("FNCE".to_string())),
    ];

    for (school, major, default_conc) in entries {
        if let Some(major_data) = resolve_major(school, major, &default_conc) {
            if let Some(conc_map) = &major_data.concentrations {
                if conc_map.is_empty() {
                    continue;
                }
                let is_core = major_data.requirements.iter()
                    .any(|r| matches!(r, Requirement::Concentration { .. }));
                let names: Vec<String> = conc_map.keys().cloned().collect();
                let key = format!("{}", major);
                result.insert(key, ConcentrationMeta { names, is_core });
            }
        }
    }

    result
}

#[derive(Debug, serde::Serialize)]
pub struct ConcentrationMeta {
    pub names: Vec<String>,
    pub is_core: bool,
}