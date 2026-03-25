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
        ("School of Engineering and Applied Science (SEAS)".to_string(), ["Electrical Engineering (EE)", "Computer Science, BSE (CIS)", "Mechanical Engineering and Applied Mechanics (MEAM)", "Material Science and Engineering (MSE)"].map(String::from).to_vec()),
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
                "CIS" => None,
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