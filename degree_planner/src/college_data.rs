use std::collections::BTreeMap;

use crate::Requirement;
use crate::Major;

pub fn create_econ_major() -> Major {
    return Major {
        short_name: "ECON".to_string(), 
        name: "Economics".to_string(), 
        requirements: vec![
            // Foundational College
            Requirement::Restriction { category: Some("Introduction".to_string()), department: Some("WRIT".to_string()), cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },    
        
            // Introduction
            Requirement::SingleCourse { category: Some("Introduction".to_string()), possibilities: vec!["ECON 0100".to_string()] },
            Requirement::SingleCourse { category: Some("Introduction".to_string()), possibilities: vec!["ECON 0200".to_string()] },

            // Intermediate
            Requirement::SingleCourse { category: Some("Intermediate".to_string()), possibilities: vec!["ECON 2100".to_string()] },
            Requirement::SingleCourse { category: Some("Intermediate".to_string()), possibilities: vec!["ECON 2200".to_string()] },

            // Statistics
            Requirement::AnyOf {
                category: Some("Statistics".to_string()),
                possibilities: vec![
                    Requirement::SingleCourse { category: Some("Statistics".to_string()), possibilities: vec!["ECON 2300".to_string()] },
                    Requirement::AllOf { category: Some("Statistics".to_string()), requirements: vec![
                        Requirement::SingleCourse { category: None, possibilities: vec!["STAT 4300".to_string()] },
                        Requirement::SingleCourse { category: None, possibilities: vec!["STAT 4310".to_string()] },
                    ] },
                ]
            }

            // Econometrics
            Requirement::SingleCourse { category: Some("Econometrics".to_string()), possibilities: vec!["ECON 2310".to_string()] },

            // ECON Electives
            Requirement::Restriction { category: Some("ECON Electives".to_string()), department: Some("ECON".to_string()), cu: None, level: Some(4000), attr: None, excluding: None, number: 1, no_school: None },

            // Mathematics
            Requirement::AnyOf {
                category: Some("Mathematics".to_string()),
                possibilities: vec![
                    Requirement::AllOf {
                        category: None,
                        requirements: vec![
                            Requirement::SingleCourse { category: None, possibilities: vec!["MATH 1070".to_string()] },
                            Requirement::SingleCourse { category: None, possibilities: vec!["MATH 1080".to_string()] },
                        ]
                    },
                    Requirement::AllOf {
                        category: None,
                        requirements: vec![
                            Requirement::SingleCourse { category: None, possibilities: vec!["MATH 1400".to_string()] },
                            Requirement::SingleCourse { category: None, possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string()] },
                        ]
                    },
                ]
            }
        ],
        concentrations: None,
    }
}