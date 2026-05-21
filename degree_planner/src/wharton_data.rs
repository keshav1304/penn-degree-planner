use std::collections::BTreeMap;

use crate::Requirement;
use crate::Major;
use crate::requirement::MappedRequirement;

pub fn concentration_names() -> Vec<String> {
    create_wh_concentrations().keys().cloned().collect()
}

pub fn create_wh_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    BTreeMap::from([
        (
            "FNCE".to_string(), 
            vec![
                Requirement::Restriction { 
                    category: Some("Concentration - FNCE".to_string()), department: Some(vec!["FNCE".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["FNCE 1010", "FNCE 1000"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - FNCE".to_string()), department: Some(vec!["FNCE".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["FNCE 1010", "FNCE 1000"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - FNCE".to_string()), department: Some(vec!["FNCE".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["FNCE 1010", "FNCE 1000"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - FNCE".to_string()), department: Some(vec!["FNCE".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["FNCE 1010", "FNCE 1000"].map(String::from).to_vec()), number: 1, no_school: None 
                },
            ]
        ),
        (
            "STAT".to_string(), 
            vec![
                Requirement::Restriction { 
                    category: Some("Concentration - STAT".to_string()), department: Some(vec!["STAT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["STAT 1010", "STAT 1020", "STAT 4300", "STAT 4310"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - STAT".to_string()), department: Some(vec!["STAT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["STAT 1010", "STAT 1020", "STAT 4300", "STAT 4310"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - STAT".to_string()), department: Some(vec!["STAT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["STAT 1010", "STAT 1020", "STAT 4300", "STAT 4310"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - STAT".to_string()), department: Some(vec!["STAT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["STAT 1010", "STAT 1020", "STAT 4300", "STAT 4310"].map(String::from).to_vec()), number: 1, no_school: None 
                },
            ]
        ),
        (
            "OIDD".to_string(), 
            vec![
                Requirement::Restriction { 
                    category: Some("Concentration - OIDD".to_string()), department: Some(vec!["OIDD".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["OIDD 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - OIDD".to_string()), department: Some(vec!["OIDD".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["OIDD 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - OIDD".to_string()), department: Some(vec!["OIDD".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["OIDD 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - OIDD".to_string()), department: Some(vec!["OIDD".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["OIDD 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
            ]
        ),
        (
            "MKTG".to_string(), 
            vec![
                Requirement::Restriction { 
                    category: Some("Concentration - MKTG".to_string()), department: Some(vec!["MKTG".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["MKTG 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - MKTG".to_string()), department: Some(vec!["MKTG".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["MKTG 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - MKTG".to_string()), department: Some(vec!["MKTG".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["MKTG 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - MKTG".to_string()), department: Some(vec!["MKTG".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["MKTG 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
            ]
        ),
        (
            "MGMT".to_string(), 
            vec![
                Requirement::Restriction { 
                    category: Some("Concentration - MGMT".to_string()), department: Some(vec!["MGMT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["MGMT 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - MGMT".to_string()), department: Some(vec!["MGMT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["MGMT 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - MGMT".to_string()), department: Some(vec!["MGMT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["MGMT 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - MGMT".to_string()), department: Some(vec!["MGMT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["MGMT 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
            ]
        ),
        (
            "ACCT".to_string(), 
            vec![
                Requirement::Restriction { 
                    category: Some("Concentration - ACCT".to_string()), department: Some(vec!["ACCT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["ACCT 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - ACCT".to_string()), department: Some(vec!["ACCT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["ACCT 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - ACCT".to_string()), department: Some(vec!["ACCT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["ACCT 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - ACCT".to_string()), department: Some(vec!["ACCT".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["ACCT 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
            ]
        ),
        (
            "BEPP".to_string(), 
            vec![
                Requirement::Restriction { 
                    category: Some("Concentration - BEPP".to_string()), department: Some(vec!["BEPP".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["BEPP 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - BEPP".to_string()), department: Some(vec!["BEPP".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["BEPP 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - BEPP".to_string()), department: Some(vec!["BEPP".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["BEPP 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - BEPP".to_string()), department: Some(vec!["BEPP".to_string()]), 
                    cu: None, level: None, attr: None, excluding: Some(["BEPP 1010"].map(String::from).to_vec()), number: 1, no_school: None 
                },
            ]
        ),
        (
            "BUAN".to_string(), 
            vec![
                Requirement::Restriction { 
                    category: Some("Concentration - BUAN - WUBD".to_string()), department: None, 
                    cu: None, level: None, attr: Some(vec!["WUBD".to_string()]), excluding: None, number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - BUAN - WUBC".to_string()), department: None, 
                    cu: None, level: None, attr: Some(vec!["WUBC".to_string()]), excluding: None, number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - BUAN - WUBO".to_string()), department: None, 
                    cu: None, level: None, attr: Some(vec!["WUBC".to_string()]), excluding: None, number: 1, no_school: None 
                },
                Requirement::Restriction { 
                    category: Some("Concentration - BUAN".to_string()), department: None, 
                    cu: None, level: None, attr: Some(vec!["WUBD".to_string(), "WUBC".to_string(), "WUBO".to_string(), "WUBN".to_string()]), excluding: None, number: 1, no_school: None 
                },
            ]
        ),
    ])
}

/// Up to two distinct Wharton concentrations; unknown names are dropped.
pub fn normalize_wh_concentrations(concentrations: &[String]) -> Vec<String> {
    let catalog = create_wh_concentrations();
    let mut out = Vec::new();
    for c in concentrations {
        if catalog.contains_key(c) && !out.contains(c) {
            out.push(c.clone());
            if out.len() >= 2 {
                break;
            }
        }
    }
    out
}

fn bb_standard_exclusions(mt: bool) -> Vec<String> {
    let mut ex = vec![
        "BEPP 1000".to_string(),
        "MGMT 1010".to_string(),
        "MKTG 1010".to_string(),
        "OIDD 1010".to_string(),
        "STAT 1010".to_string(),
        "STAT 1020".to_string(),
    ];
    if mt {
        ex.push("MGMT 3010".to_string());
    }
    ex
}

fn bb_department_options(
    concentrations: &[String],
    pool: &[&str],
    exclusions: &[String],
) -> Vec<Requirement> {
    let mut depts: Vec<String> = pool.iter().map(|s| s.to_string()).collect();
    if concentrations.len() < 2 {
        for c in concentrations {
            depts.retain(|d| d != c);
        }
    }
    depts
        .into_iter()
        .map(|dept| Requirement::Restriction {
            category: None,
            department: Some(vec![dept]),
            cu: None,
            level: None,
            attr: None,
            excluding: Some(exclusions.to_vec()),
            number: 1,
            no_school: None,
        })
        .collect()
}

fn business_breadth_requirements(
    concentrations: &[String],
    pool: &[&str],
    slot_labels: &[&str],
    mt: bool,
) -> Vec<Requirement> {
    let exclusions = bb_standard_exclusions(mt);
    let opts = bb_department_options(concentrations, pool, &exclusions);
    slot_labels
        .iter()
        .map(|label| Requirement::AnyOf {
            category: Some(label.to_string()),
            possibilities: opts.clone(),
        })
        .collect()
}

fn wh_concentration_requirements(concentrations: &[String]) -> Vec<Requirement> {
    let catalog = create_wh_concentrations();
    let mut reqs = Vec::new();
    for name in concentrations {
        if let Some(chain) = catalog.get(name) {
            reqs.extend(chain.clone());
        }
    }
    reqs
}

pub fn create_wh_fl_major(concentrations: Vec<String>) -> Major {
    let concentrations = normalize_wh_concentrations(&concentrations);
    let concs = if concentrations.is_empty() {
        vec!["FNCE".to_string()]
    } else {
        concentrations
    };
    let wh_concentrations = create_wh_concentrations();
    let bb_pool = ["ACCT", "BEPP", "MGMT", "MKTG", "HCMG", "REAL", "OIDD", "STAT", "LGST", "FNCE"];
    let bb_reqs = business_breadth_requirements(&concs, &bb_pool, &["Business Breadth"], false);

    let requirements: Vec<Requirement> = vec![
            // First-Year Foundations
            Requirement::AnyOf { category: Some("First-Year Foundations".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["BEPP 1000".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0200".to_string()] },
                ] }
            ] },
            Requirement::SingleCourse { category: Some("First-Year Foundations".to_string()), possibilities: vec!["MATH 1400".to_string(), "MATH 1070".to_string()] },
            Requirement::Restriction { category: Some("First-Year Foundations".to_string()), department: Some(vec!["WRIT".to_string()]), cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },

            // Leadership Journey
            Requirement::SingleCourse { category: Some("Leadership Journey".to_string()), possibilities: vec!["WH 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Leadership Journey".to_string()), possibilities: vec!["WH 2010".to_string(), "WH 2011".to_string()] },
            Requirement::SingleCourse { category: Some("Leadership Journey".to_string()), possibilities: vec!["MGMT 3010".to_string()] },
            Requirement::Restriction { category: Some("Leadership Journey".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCP".to_string()]), excluding: None, number: 1, no_school: None },

            // Fundamentals
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1020".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["BEPP 2500".to_string(), "BEPP 2508".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1000".to_string(), "FNCE 1008".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1010".to_string(), "FNCE 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["LGST 1000".to_string(), "LGST 1010".to_string(), "LGST 1008".to_string(), "LGST 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MGMT 1010".to_string(), "MKTG 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MKTG 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["OIDD 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 1010".to_string(), "STAT 4300".to_string(), "ESE 3010".to_string(), "STAT 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 1020".to_string(), "STAT 4310".to_string(), "ESE 4020".to_string(), "STAT 1028".to_string()] },

            // Flex Fundamentals
            Requirement::Restriction { category: Some("Flex Fundamentals".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUGE".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Flex Fundamentals".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUTI".to_string()]), excluding: None, number: 1, no_school: None },
            
            // Business Breadth
        ]
        .into_iter()
        .chain(bb_reqs)
        .chain(vec![
            // Liberal Arts and Sciences (foreign language required)
            // WUHM - language
            // flex gen-ed - language
            // wunm - 1
            // wuss - 1
            // flex gen-ed - 2
            
            // wucn - 2 (double count above)

            // wucu or wucn - 1
            Requirement::DoubleCount {
                category: Some("Liberal Arts and Sciences".to_string()), 
                double_counting_requirements: vec![
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string()]), excluding: None, number: 1, no_school: None },
                ],
                base_requirements: vec![
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["WUFL".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["WUFL".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["WUNM".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["WUSS".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: Some("WH".to_string()) },
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: Some("WH".to_string()) },
                ]
            },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Cross Cultural".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string(), "WUCU".to_string()]), excluding: None, number: 1, no_school: None },

            // Unrestricted Electives
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 5, no_school: None },
        ])
        .chain(wh_concentration_requirements(&concs))
        .collect();
    Major::new(
        "WH",
        "Wharton Undergraduate",
        "WH_FL",
        requirements,
        Some(wh_concentrations),
    )
    .with_post_validate(post_validate_overlaps)
}

pub fn create_wh_nofl_major(concentrations: Vec<String>) -> Major {
    let concentrations = normalize_wh_concentrations(&concentrations);
    let concs = if concentrations.is_empty() {
        vec!["FNCE".to_string()]
    } else {
        concentrations
    };
    let wh_concentrations = create_wh_concentrations();
    let bb_pool = ["FNCE", "ACCT", "BEPP", "MGMT", "MKTG", "HCMG", "REAL", "OIDD", "STAT", "LGST"];
    let bb_reqs = business_breadth_requirements(
        &concs,
        &bb_pool,
        &["Business Breadth 1", "Business Breadth 2", "Business Breadth 3"],
        false,
    );

    let requirements: Vec<Requirement> = vec![
             // First-Year Foundations
            Requirement::AnyOf { category: Some("First-Year Foundations - Econ".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["BEPP 1000".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0200".to_string()] },
                ] }
            ] },
            Requirement::SingleCourse { category: Some("First-Year Foundations - Math".to_string()), possibilities: vec!["MATH 1400".to_string(), "MATH 1070".to_string()] },
            Requirement::Restriction { category: Some("First-Year Foundations - Writing Sem".to_string()), department: Some(vec!["WRIT".to_string()]), cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },

            // Leadership Journey
            Requirement::SingleCourse { category: Some("Leadership Journey - WH 1010".to_string()), possibilities: vec!["WH 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Leadership Journey - WH 2010/2011".to_string()), possibilities: vec!["WH 2010".to_string(), "WH 2011".to_string()] },
            Requirement::SingleCourse { category: Some("Leadership Journey - MGMT 3010".to_string()), possibilities: vec!["MGMT 3010".to_string()] },
            Requirement::Restriction { category: Some("Undergraduate Capstone".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCP".to_string()]), excluding: None, number: 1, no_school: None },

            // Fundamentals
            Requirement::SingleCourse { category: Some("Fundamentals - ACCT 1010".to_string()), possibilities: vec!["ACCT 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - ACCT 1020".to_string()), possibilities: vec!["ACCT 1020".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - BEPP 2500/2508".to_string()), possibilities: vec!["BEPP 2500".to_string(), "BEPP 2508".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - FNCE 1000/1008".to_string()), possibilities: vec!["FNCE 1000".to_string(), "FNCE 1008".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - FNCE 1010/1018".to_string()), possibilities: vec!["FNCE 1010".to_string(), "FNCE 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - LGST 1000/1010/1008/1018".to_string()), possibilities: vec!["LGST 1000".to_string(), "LGST 1010".to_string(), "LGST 1008".to_string(), "LGST 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - MGMT 1010/MKTG 1018".to_string()), possibilities: vec!["MGMT 1010".to_string(), "MKTG 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - MKTG 1010".to_string()), possibilities: vec!["MKTG 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - OIDD 1010".to_string()), possibilities: vec!["OIDD 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - STAT I".to_string()), possibilities: vec!["STAT 1010".to_string(), "STAT 4300".to_string(), "ESE 3010".to_string(), "STAT 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - STAT II".to_string()), possibilities: vec!["STAT 1020".to_string(), "STAT 4310".to_string(), "ESE 4020".to_string(), "STAT 1028".to_string()] },

            // Flex Fundamentals
            Requirement::Restriction { category: Some("Flex Fundamentals - WUGE".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUGE".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Flex Fundamentals - WUTI".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUTI".to_string()]), excluding: None, number: 1, no_school: None },
            
            // Business Breadth
        ]
        .into_iter()
        .chain(bb_reqs)
        .chain(vec![
            // Liberal Arts and Sciences (foreign language not required)
            // wuhm - 1
            // wunm - 1
            // wuss - 1
            // flex gen-ed - 3

            // wucn - 2 (double count above)

            // wucu or wucn - 1
            Requirement::DoubleCount {
                category: Some("Liberal Arts and Sciences - SSH".to_string()), 
                double_counting_requirements: vec![
                    Requirement::Restriction { category: Some("Liberal Arts and Sciences - Non-US CCP 1".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: Some("Liberal Arts and Sciences - Non-US CCP 2".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string()]), excluding: None, number: 1, no_school: None },
                ],
                base_requirements: vec![
                    Requirement::Restriction { category: Some("Wharton Humanities".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUHM".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: Some("Wharton Natural Science & Math".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUNM".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: Some("Wharton Social Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUSS".to_string()]), excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: Some("Non-Wharton Course - 1".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: Some("WH".to_string()) },
                    Requirement::Restriction { category: Some("Non-Wharton Course - 2".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: Some("WH".to_string()) },
                    Requirement::Restriction { category: Some("Non-Wharton Course - 3".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: Some("WH".to_string()) },
                ]
            },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - CCP".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string(), "WUCU".to_string()]), excluding: None, number: 1, no_school: None },

            // Unrestricted Electives
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
        ])
        .chain(wh_concentration_requirements(&concs))
        .collect();
    Major::new(
        "WH",
        "Wharton Undergraduate",
        "WH_NOFL",
        requirements,
        Some(wh_concentrations),
    )
    .with_post_validate(post_validate_overlaps)
}

pub fn create_wh_nofl_mt_major(concentrations: Vec<String>) -> Major {
    let concentrations = normalize_wh_concentrations(&concentrations);
    let concs = if concentrations.is_empty() {
        vec!["FNCE".to_string()]
    } else {
        concentrations
    };
    let wh_concentrations = create_wh_concentrations();
    let bb_pool = ["FNCE", "ACCT", "BEPP", "MGMT", "MKTG", "HCMG", "REAL", "OIDD", "STAT", "LGST"];
    let bb_reqs = business_breadth_requirements(
        &concs,
        &bb_pool,
        &["Business Breadth - I", "Business Breadth - II"],
        true,
    );

    let requirements: Vec<Requirement> = vec![
             // First-Year Foundations
            Requirement::AnyOf { category: Some("First-Year Foundations - Econ".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["BEPP 1000".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0200".to_string()] },
                ] }
            ] },
            Requirement::SingleCourse { category: Some("First-Year Foundations - Math".to_string()), possibilities: vec!["MATH 1400".to_string(), "MATH 1070".to_string()] },
            Requirement::Restriction { category: Some("First-Year Foundations - Writing Sem".to_string()), department: Some(vec!["WRIT".to_string()]), cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },

            // Leadership Journey
            Requirement::SingleCourse { category: Some("Leadership Journey - WH 1010".to_string()), possibilities: vec!["WH 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Leadership Journey - WH 2010/2011".to_string()), possibilities: vec!["WH 2010".to_string(), "WH 2011".to_string()] },
            Requirement::SingleCourse { category: Some("Leadership Journey - MGMT 3010".to_string()), possibilities: vec!["MGMT 3010".to_string()] },

            // Fundamentals
            Requirement::SingleCourse { category: Some("Fundamentals - ACCT 1010".to_string()), possibilities: vec!["ACCT 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - ACCT 1020".to_string()), possibilities: vec!["ACCT 1020".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - BEPP 2500/2508".to_string()), possibilities: vec!["BEPP 2500".to_string(), "BEPP 2508".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - FNCE 1000/1008".to_string()), possibilities: vec!["FNCE 1000".to_string(), "FNCE 1008".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - FNCE 1010/1018".to_string()), possibilities: vec!["FNCE 1010".to_string(), "FNCE 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - MGMT 1010/MKTG 1018".to_string()), possibilities: vec!["MGMT 1010".to_string(), "MKTG 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - MKTG 1010".to_string()), possibilities: vec!["MKTG 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - STAT I".to_string()), possibilities: vec!["STAT 4300".to_string(), "ESE 3010".to_string(), "STAT 1018".to_string()] },
            Requirement::SingleCourse { category: Some("Fundamentals - STAT II".to_string()), possibilities: vec!["STAT 4310".to_string(), "ESE 4020".to_string(), "STAT 1028".to_string()] },

            // Flex Fundamentals
            Requirement::Restriction { category: Some("Flex Fundamentals - GEBS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUGE".to_string()]), excluding: None, number: 1, no_school: None },
            
            // Business Breadth
        ]
        .into_iter()
        .chain(bb_reqs)
        .chain(vec![
            // Jerome Fisher M&T
            Requirement::SingleCourse { category: Some("M&T Soph Course".to_string()), possibilities: vec!["MGMT 2370".to_string()] },
            Requirement::SingleCourse { category: Some("M&T Freshman Course".to_string()), possibilities: vec!["OIDD 2340".to_string()] },

            // Liberal Arts and Sciences (foreign language not required)
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Humanities and Social Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUHM".to_string(), "WUSS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Humanities and Social Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUHM".to_string(), "WUSS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Cross Cultural".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Cross Cultural".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string(), "WUCU".to_string()]), excluding: None, number: 1, no_school: None },
        ])
        .chain(wh_concentration_requirements(&concs))
        .collect();
    Major::new(
        "WH",
        "Wharton Undergraduate",
        "WH_NOFL_MT",
        requirements,
        Some(wh_concentrations),
    )
    .with_post_validate(post_validate_overlaps)
}

// ─── Post-validation: Wharton-only double-count rules ───

const MT_MGMT2370: &str = "MGMT 2370";

fn course_department(course_id: &str) -> Option<String> {
    course_id
        .split_whitespace()
        .next()
        .map(|d| d.to_string())
}

fn requirement_matches_concentration(req: &Requirement, conc_name: &str) -> bool {
    let cat = req.get_category().to_lowercase();
    cat.contains(&format!("concentration - {}", conc_name.to_lowercase()))
}

fn is_business_breadth_requirement(req: &Requirement) -> bool {
    req.get_category()
        .to_lowercase()
        .contains("business breadth")
}

fn is_mt_mgmt2370_soph_requirement(req: &Requirement) -> bool {
    if let Requirement::SingleCourse { category, possibilities } = req {
        let mt_soph = category
            .as_ref()
            .map(|c| c.to_lowercase().contains("m&t soph"))
            .unwrap_or(false);
        return mt_soph && possibilities.iter().any(|p| p == MT_MGMT2370);
    }
    false
}

fn mapped_has_course(mapped: &MappedRequirement, course: &str) -> bool {
    mapped.course_ids.iter().any(|c| c == course)
}

fn apply_mt_mgmt2370_overlap(
    concentrations: &[String],
    fulfilled: &mut Vec<MappedRequirement>,
    unfulfilled: &mut Vec<MappedRequirement>,
) {
    let mgmt_is_conc = concentrations.iter().any(|c| c == "MGMT");

    let soph_fulfilled_with_2370 = fulfilled.iter().any(|m| {
        is_mt_mgmt2370_soph_requirement(&m.requirement) && mapped_has_course(m, MT_MGMT2370)
    });

    if mgmt_is_conc {
        if !soph_fulfilled_with_2370 {
            return;
        }
        if let Some(idx) = unfulfilled.iter().position(|m| {
            requirement_matches_concentration(&m.requirement, "MGMT")
        }) {
            let mapped = unfulfilled.remove(idx);
            fulfilled.push(MappedRequirement {
                requirement: mapped.requirement,
                course_ids: vec![MT_MGMT2370.to_string()],
                instance_id: mapped.instance_id,
            });
        }
        return;
    }

    if soph_fulfilled_with_2370 {
        if let Some(idx) = unfulfilled
            .iter()
            .position(|m| is_business_breadth_requirement(&m.requirement))
        {
            let mapped = unfulfilled.remove(idx);
            fulfilled.push(MappedRequirement {
                requirement: mapped.requirement,
                course_ids: vec![MT_MGMT2370.to_string()],
                instance_id: mapped.instance_id,
            });
            return;
        }
    }

    for mapped in fulfilled.clone() {
        if !is_business_breadth_requirement(&mapped.requirement) {
            continue;
        }
        if !mapped_has_course(&mapped, MT_MGMT2370) {
            continue;
        }
        if let Some(idx) = unfulfilled
            .iter()
            .position(|m| is_mt_mgmt2370_soph_requirement(&m.requirement))
        {
            let mt = unfulfilled.remove(idx);
            fulfilled.push(MappedRequirement {
                requirement: mt.requirement,
                course_ids: vec![MT_MGMT2370.to_string()],
                instance_id: mt.instance_id,
            });
            return;
        }
    }
}

fn apply_double_concentration_bb_overlap(
    concentrations: &[String],
    fulfilled: &mut Vec<MappedRequirement>,
    unfulfilled: &mut Vec<MappedRequirement>,
) {
    if concentrations.len() < 2 {
        return;
    }

    for mapped in fulfilled.iter() {
        if !is_business_breadth_requirement(&mapped.requirement) {
            continue;
        }
        for bb_course in &mapped.course_ids {
            let Some(dept) = course_department(bb_course) else {
                continue;
            };
            if !concentrations.contains(&dept) {
                continue;
            }
            if let Some(idx) = unfulfilled.iter().position(|m| {
                requirement_matches_concentration(&m.requirement, &dept)
            }) {
                let mapped = unfulfilled.remove(idx);
                fulfilled.push(MappedRequirement {
                    requirement: mapped.requirement,
                    course_ids: vec![bb_course.clone()],
                    instance_id: mapped.instance_id,
                });
                return;
            }
        }
    }

    for mapped in fulfilled.clone() {
        for course in &mapped.course_ids {
            let Some(dept) = course_department(course) else {
                continue;
            };
            if !concentrations.contains(&dept) {
                continue;
            }
            if !requirement_matches_concentration(&mapped.requirement, &dept) {
                continue;
            }
            if let Some(idx) = unfulfilled
                .iter()
                .position(|m| is_business_breadth_requirement(&m.requirement))
            {
                let mapped = unfulfilled.remove(idx);
                fulfilled.push(MappedRequirement {
                    requirement: mapped.requirement,
                    course_ids: vec![course.clone()],
                    instance_id: mapped.instance_id,
                });
                return;
            }
        }
    }
}

/// Registered on all Wharton majors via `Major::post_validate`.
pub fn post_validate_overlaps(
    major_key: &str,
    concentrations: &[String],
    fulfilled: &mut Vec<MappedRequirement>,
    unfulfilled: &mut Vec<MappedRequirement>,
) {
    if major_key == "WH_NOFL_MT" {
        apply_mt_mgmt2370_overlap(concentrations, fulfilled, unfulfilled);
    }
    if concentrations.len() >= 2 {
        apply_double_concentration_bb_overlap(concentrations, fulfilled, unfulfilled);
    }
}