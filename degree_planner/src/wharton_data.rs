use std::collections::BTreeMap;

use crate::Requirement;
use crate::Major;

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

pub fn create_wh_fl_major(concentration_name: String) -> Major {
    let wh_concentrations = create_wh_concentrations();
    let mut bb_options = ["ACCT", "BEPP", "MGMT", "MKTG", "HCMG", "REAL", "OIDD", "STAT", "LGST", "FNCE"].map(String::from).to_vec();
    bb_options.retain(|x| *x != concentration_name);

    let mut bb_requirement_options = Vec::new();

    for option in bb_options {
        bb_requirement_options.push(Requirement::Restriction { category: None, department: Some(vec![option.to_string()]), cu: None, level: None, attr: None, excluding: Some(vec!["BEPP 1000".to_string(), "MGMT 1010".to_string(), "MKTG 1010".to_string(), "OIDD 1010".to_string(), "STAT 1010".to_string(), "STAT 1020".to_string()]), number: 1, no_school: None });
    }

    return Major {
        short_name: "WH".to_string(), 
        name: "Wharton Undergraduate".to_string(), 
        requirements: vec![
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
            Requirement::AnyOf { category: Some("Business Breadth".to_string()), possibilities: bb_requirement_options.clone() },

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
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 5, no_school: None }
        ].into_iter().chain(wh_concentrations.get(&concentration_name).unwrap().clone()).collect(),
        concentrations: Some(create_wh_concentrations()),
    }
}

pub fn create_wh_nofl_major(concentration_name: String) -> Major {
    let wh_concentrations = create_wh_concentrations();
    let mut bb_options = ["FNCE", "ACCT", "BEPP", "MGMT", "MKTG", "HCMG", "REAL", "OIDD", "STAT", "LGST"].map(String::from).to_vec();
    bb_options.retain(|x| *x != concentration_name);

    let mut bb_requirement_options = Vec::new();

    for option in bb_options {
        bb_requirement_options.push(Requirement::Restriction { category: None, department: Some(vec![option.to_string()]), cu: None, level: None, attr: None, excluding: Some(vec!["BEPP 1000".to_string(), "MGMT 1010".to_string(), "MKTG 1010".to_string(), "OIDD 1010".to_string(), "STAT 1010".to_string(), "STAT 1020".to_string()]), number: 1, no_school: None });
    }

    return Major {
        short_name: "WH".to_string(), 
        name: "Wharton Undergraduate".to_string(), 
        requirements: vec![
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
            Requirement::AnyOf { category: Some("Business Breadth 1".to_string()), possibilities: bb_requirement_options.clone() },
            Requirement::AnyOf { category: Some("Business Breadth 2".to_string()), possibilities: bb_requirement_options.clone() },
            Requirement::AnyOf { category: Some("Business Breadth 3".to_string()), possibilities: bb_requirement_options.clone() },

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
        ].into_iter().chain(wh_concentrations.get(&concentration_name).unwrap().clone()).collect(),
        concentrations: Some(wh_concentrations),
    }
}

pub fn create_wh_nofl_mt_major(concentration_name: String) -> Major {
    let wh_concentrations = create_wh_concentrations();
    let mut bb_options = ["FNCE", "ACCT", "BEPP", "MGMT", "MKTG", "HCMG", "REAL", "OIDD", "STAT", "LGST"].map(String::from).to_vec();
    bb_options.retain(|x| *x != concentration_name);

    let mut bb_requirement_options = Vec::new();

    for option in bb_options {
        bb_requirement_options.push(Requirement::Restriction { category: None, department: Some(vec![option.to_string()]), cu: None, level: None, attr: None, excluding: Some(vec!["BEPP 1000".to_string(), "MGMT 1010".to_string(), "MKTG 1010".to_string(), "OIDD 1010".to_string(), "STAT 1010".to_string(), "STAT 1020".to_string(), "MGMT 3010".to_string()]), number: 1, no_school: None });
    }

    return Major {
        short_name: "WH".to_string(), 
        name: "Wharton Undergraduate".to_string(), 
        requirements: vec![
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
            Requirement::AnyOf { category: Some("Business Breadth - I".to_string()), possibilities: bb_requirement_options.clone() },
            Requirement::AnyOf { category: Some("Business Breadth - II".to_string()), possibilities: bb_requirement_options.clone() },
            
            // Jerome Fisher M&T
            Requirement::SingleCourse { category: Some("M&T Soph Course".to_string()), possibilities: vec!["MGMT 2370".to_string()] },
            Requirement::SingleCourse { category: Some("M&T Freshman Course".to_string()), possibilities: vec!["OIDD 2340".to_string()] },

            // Liberal Arts and Sciences (foreign language not required)
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Humanities and Social Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUHM".to_string(), "WUSS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Humanities and Social Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUHM".to_string(), "WUSS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Cross Cultural".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Cross Cultural".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string(), "WUCU".to_string()]), excluding: None, number: 1, no_school: None },

            
        ].into_iter().chain(wh_concentrations.get(&concentration_name).unwrap().clone()).collect(),
        concentrations: Some(wh_concentrations)
    }
}