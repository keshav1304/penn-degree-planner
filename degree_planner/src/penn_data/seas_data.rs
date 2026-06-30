use std::collections::{BTreeMap, HashMap};
use crate::Major;
use crate::Requirement;
use crate::requirement::PoolConstraint;
use crate::schedule_template::{
    schedule_hints_from_array, scheduled, insert_fixed_course_hints, ScheduleHint, Semester,
    Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S,
};

/// Mandatory Y4 Fall / Y4 Spring placement for SEAS senior design course codes.
fn apply_seas_senior_design_fixed_hints(hints: &mut HashMap<String, ScheduleHint>) {
    insert_fixed_course_hints(
        hints,
        &[
            ("CIS 4000", Y4F),
            ("CIS 4100", Y4F),
            ("ESE 4500", Y4F),
            ("MEAM 4450", Y4F),
            ("BE 4950", Y4F),
            ("MSE 4950", Y4F),
            ("CBE 4000", Y4F),
            ("CIS 4010", Y4S),
            ("CIS 4110", Y4S),
            ("ESE 4510", Y4S),
            ("MEAM 4460", Y4S),
            ("BE 4960", Y4S),
            ("MSE 4960", Y4S),
            ("CBE 4590", Y4S),
        ],
    );
}

fn seas_schedule_hints(schedule: &[Semester]) -> HashMap<String, ScheduleHint> {
    let mut hints = schedule_hints_from_array(schedule);
    apply_seas_senior_design_fixed_hints(&mut hints);
    hints
}

/// Concentration names for a SEAS major (empty if the major has none).
pub fn concentration_names_for(major_code: &str) -> Vec<String> {
    let concentrations = match major_code {
        "EE" => create_ee_major().concentrations,
        "CIS" => create_cis_major().concentrations,
        "CMPE" => create_cmpe_major().concentrations,
        "MEAM" => create_meam_major("General".to_string()).concentrations,
        "MSE" => create_mse_major().concentrations,
        "BE" => create_be_major().concentrations,
        _ => return vec![],
    };
    concentrations
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

pub fn create_ee_major() -> Major {
    let (requirements, mut schedule_hints) = scheduled(vec![
            // Engineering
            (Y1F, Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1100".to_string()] }),
            (Y1F, Requirement::AnyOf { 
                category: Some("Engineering".to_string()), 
                possibilities: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 1110".to_string(), "MEAM 1010".to_string()] },
                    Requirement::Restriction { category: Some("Sub for ESE 1110".to_string()), department:Some(vec!["ESE".to_string()]), number: 1, cu: None, level: None, max_level: None, attr: None, excluding: Some(["ESE 1120", "ESE 2030", "ESE 3010", "ESE 4020"].map(String::from).to_vec()), no_school: None }
                ] 
            }),
            (Y1S, Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1200".to_string(), "CIS 2400".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 2150".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 2180".to_string()] }),
            (Y2S, Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 2240".to_string()] }),
            
            // Intermediate or Advanced ESE Elective
            (Y2S, Requirement::Restriction { 
                category: Some("Intermediate or Advanced ESE Elective".to_string()), 
                department:Some(vec!["ESE".to_string()]), number:1, cu: None, level: Some(2000), max_level: None, attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), 
                no_school: None 
            }),
            
            // Advanced ESE courses
            (Y3F, Requirement::Restriction { 
                category: Some("Advanced ESE courses".to_string()), 
                department:Some(vec!["ESE".to_string()]), number:1, cu: None, level: Some(3000), max_level: None, attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            }),
            (Y3F, Requirement::Restriction { 
                category: Some("Advanced ESE courses".to_string()), 
                department:Some(vec!["ESE".to_string()]), number:1, cu: None, level: Some(3000), max_level: None, attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            }),
            (Y3S, Requirement::Restriction { 
                category: Some("Advanced ESE courses".to_string()), 
                department:Some(vec!["ESE".to_string()]), number:1, cu: None, level: Some(3000), max_level: None, attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            }),

            (Y4F, Requirement::AnyOf { 
                category: Some("Advanced ESE courses".to_string()),
                possibilities: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["CIS 5200".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["BE 5210".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["CIS 4710".to_string()] },
                    Requirement::Restriction { 
                        category: None, 
                        department: Some(vec!["ESE".to_string()]), number: 1, cu: None, level: Some(3000), max_level: None, attr: None, excluding: None, 
                        no_school: None 
                    },
                ] 
            }),

            // Design and Project courses
            (Y3S, Requirement::AnyOf { 
                category: Some("Design and Project courses".to_string()), 
                possibilities: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 4210".to_string()] },
                    Requirement::AllOf { category: Some("Design and Project courses (Research)".to_string()), requirements: vec![
                        Requirement::SingleCourse { category: None, possibilities: vec!["ESE 2900".to_string()] },
                        Requirement::SingleCourse { category: None, possibilities: vec!["ESE 2910".to_string()] }
                    ] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 3190".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 3360".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 3500".to_string()] },
                    Requirement::AllOf { category: None, requirements: vec![
                        Requirement::SingleCourse { category: None, possibilities: vec!["BE 4700".to_string()] },
                        Requirement::Restriction { category: Some("Extra half-credit course with BE 4700".to_string()), department: None, cu: Some(5), level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None  }
                    ] },
                ] 
            }),
            (Y4F, Requirement::SingleCourse { category: Some("Design and Project courses".to_string()), possibilities: vec!["ESE 4500".to_string()] }),
            (Y4S, Requirement::SingleCourse { category: Some("Design and Project courses".to_string()), possibilities: vec!["ESE 4510".to_string()] }),

            // Math and Natural Science
            (Y1F, Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 2400".to_string(), "ESE 2030".to_string()] }),
            (Y2S, Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 3010".to_string()] }),
            (Y1F, Requirement::AnyOf {
                category: Some("Math and Natural Science".to_string()), 
                possibilities: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0150".to_string()] },    
                    Requirement::AllOf {
                        category: None, 
                        requirements: vec![
                            Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1100".to_string()] },
                            Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1470".to_string(), "BIOL 1124".to_string(), "PHYS 0050".to_string(), "CHEM 1101".to_string()] },
                        ]
                    },
                    Requirement::AllOf {
                        category: None, 
                        requirements: vec![
                            Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0140".to_string()] },
                            Requirement::SingleCourse { category: None, possibilities: vec!["BIOL 1124".to_string(), "PHYS 0050".to_string(), "MEAM 1470".to_string(), "CHEM 1101".to_string()] },
                        ]
                    },
                    Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0170".to_string()] },
                ]
            }),
            (Y1S, Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 1120".to_string()] }),
            (Y1S, Requirement::SingleCourse { 
                category: Some("Math and Natural Science".to_string()), 
                possibilities: vec![
                    "CHEM 1012".to_string(),
                    "EAS 0091".to_string(),
                    "BIOL 1121".to_string(),
                    "BIOL 1101".to_string()
                ]
            }),
            (Y3F, Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUMA".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y3S, Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None }),

            // Professional Electives
            (Y3F, Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string(), "EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y3S, Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string(), "EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y4S, Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string(), "EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y3S, Requirement::AnyOf {
                category: Some("Professional Electives".to_string()), 
                possibilities: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 4000".to_string(), "EAS 5450".to_string(), "ESE 5950".to_string(), "MGMT 2370".to_string(), "OIDD 2360".to_string()] },
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string(), "EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None },
                ]
            }),

            // General Electives
            (Y2F, Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["EAS 2030".to_string(), "LAWM 5060".to_string()] }),
            (Y2S, Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y3F, Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y4F, Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y4S, Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y3S, Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y3S, Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None }),
            (Y4S, Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None }),
    ]);

    apply_seas_senior_design_fixed_hints(&mut schedule_hints);

    return Major {
        short_name: "EE".to_string(),
        name: "Electrical Engineering".to_string(),
        requirements,
        schedule_hints,
        concentrations: Some(BTreeMap::from([
            (
                "Data Science".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["ESE 3060".to_string()] },
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["ESE 4020".to_string()] },
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["NETS 3120".to_string(), "ESE 5140".to_string(), "CIS 5200".to_string(), "CIS 5450".to_string(), "ESE 5460".to_string(), "ESE 6500".to_string()] },
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["NETS 3120".to_string(), "ESE 5140".to_string(), "CIS 5200".to_string(), "CIS 5450".to_string(), "ESE 5460".to_string(), "ESE 6500".to_string()] }
                ]
            ),
            (
                "Mixed-Signal and RF Integrated Circuits".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Mixed-Signal and RF Integrated Circuits".to_string()), possibilities: vec!["ESE 3190".to_string()] },
                    Requirement::SingleCourse { category: Some("Mixed-Signal and RF Integrated Circuits".to_string()), possibilities: vec!["ESE 3700".to_string()] },
                    Requirement::AnyOf {
                        category: Some("Mixed-Signal and RF Integrated Circuits".to_string()), 
                        possibilities: vec![
                            Requirement::AllOf {
                                category: None,
                                requirements: vec![
                                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 5730".to_string()] },
                                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 5750".to_string()] },
                                ]
                            },
                            Requirement::AllOf {
                                category: None,
                                requirements: vec![
                                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 4190".to_string(), "ESE 5780".to_string(), "ESE 5800".to_string(), "ESE 6680".to_string(), "ESE 6710".to_string(), "ESE 6720".to_string()] },
                                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 4190".to_string(), "ESE 5780".to_string(), "ESE 5800".to_string(), "ESE 6680".to_string(), "ESE 6710".to_string(), "ESE 6720".to_string()] },
                                ]
                            },
                        ]
                    }
                ]
            ),
            (
                "System-on-A-Chip Design".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("System-on-A-Chip Design".to_string()), possibilities: vec!["ESE 3700".to_string()] },
                    Requirement::AnyOf {
                        category: Some("System-on-A-Chip Design".to_string()),
                        possibilities: vec![
                            Requirement::AllOf {
                                category: None,
                                requirements: vec![
                                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 5730".to_string()] },
                                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 5750".to_string()] },
                                    Requirement::SingleCourse { category: None, possibilities: vec!["CIS 4710".to_string(), "ESE 5320".to_string(), "ESE 5390".to_string()] },
                                ]
                            },
                            Requirement::AllOf {
                                category: None,
                                requirements: vec![
                                    Requirement::SingleCourse { category: None, possibilities: vec!["CIS 4710".to_string()] },
                                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 5320".to_string()] },
                                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 5390".to_string()] },
                                ],
                            },
                        ]
                    }
                ]
            ),
            (
                "Photonics and Quantum Technology".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Photonics and Quantum Technology".to_string()), possibilities: vec!["ESE 3200".to_string()] },
                    Requirement::SingleCourse { category: Some("Photonics and Quantum Technology".to_string()), possibilities: vec!["ESE 3300".to_string()] },
                    Requirement::SingleCourse { category: Some("Photonics and Quantum Technology".to_string()), possibilities: vec!["ESE 5090".to_string(), "ESE 5100".to_string(), "ESE 5130".to_string(), "ESE 5230".to_string(), "ESE 5360".to_string(), "ESE 6730".to_string()] },
                    Requirement::SingleCourse { category: Some("Photonics and Quantum Technology".to_string()), possibilities: vec!["ESE 5090".to_string(), "ESE 5100".to_string(), "ESE 5130".to_string(), "ESE 5230".to_string(), "ESE 5360".to_string(), "ESE 6730".to_string()] },
                ]
            ),
            (
                "Microsystems and Nanotechnology".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Microsystems and Nanotechnology".to_string()), possibilities: vec!["ESE 5250".to_string()] },
                    Requirement::SingleCourse { category: Some("Microsystems and Nanotechnology".to_string()), possibilities: vec!["ESE 3300".to_string(), "ESE 5100".to_string(), "ESE 5210".to_string(), "ESE 5290".to_string(), "ESE 5360".to_string(), "ESE 6210".to_string(), "ESE 6250".to_string()] },
                    Requirement::SingleCourse { category: Some("Microsystems and Nanotechnology".to_string()), possibilities: vec!["ESE 3300".to_string(), "ESE 5100".to_string(), "ESE 5210".to_string(), "ESE 5290".to_string(), "ESE 5360".to_string(), "ESE 6210".to_string(), "ESE 6250".to_string()] },
                    Requirement::SingleCourse { category: Some("Microsystems and Nanotechnology".to_string()), possibilities: vec!["ESE 3300".to_string(), "ESE 5100".to_string(), "ESE 5210".to_string(), "ESE 5290".to_string(), "ESE 5360".to_string(), "ESE 6210".to_string(), "ESE 6250".to_string()] },
                ]
            ),
            (
                "Robotics".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Robotics".to_string()), possibilities: vec!["ESE 4210".to_string()] },
                    Requirement::SingleCourse { category: Some("Robotics".to_string()), possibilities: vec!["ESE 5000".to_string(), "ESE 5050".to_string(), "MEAM 5200".to_string(), "ESE 6150".to_string(), "ESE 6190".to_string(), "ESE 6250".to_string(), "ESE 6500".to_string(), "MEAM 6200".to_string()] },
                    Requirement::SingleCourse { category: Some("Robotics".to_string()), possibilities: vec!["ESE 5000".to_string(), "ESE 5050".to_string(), "MEAM 5200".to_string(), "ESE 6150".to_string(), "ESE 6190".to_string(), "ESE 6250".to_string(), "ESE 6500".to_string(), "MEAM 6200".to_string()] },
                    Requirement::SingleCourse { category: Some("Robotics".to_string()), possibilities: vec!["ESE 5000".to_string(), "ESE 5050".to_string(), "MEAM 5200".to_string(), "ESE 6150".to_string(), "ESE 6190".to_string(), "ESE 6250".to_string(), "ESE 6500".to_string(), "MEAM 6200".to_string()] },
                ]
            ),
        ])),
    };
}

/// MEAM schedule template — one semester per top-level requirement, in list order.
/// Based on the official MEAM four-year plan (Freshman → Senior, Fall/Spring).
const MEAM_SCHEDULE: [Semester; 32] = [
    // MEAM Core — 2020/2100/2470 (Y2F), 2030/2110/2480 (Y2S), 3470 (Y3F), 3480 (Y3S), 4450/4460 (Y4)
    Y2F, Y2S, Y2F, Y2S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S,
    // Math and Natural Science — 1400/1100·1470/CHEM (Y1F), 1410/PHYS151 (Y1S), 2400 (Y2F), ENM2510 (Y2S), electives (Y3)
    Y1F, Y1S, Y2F, Y2S, Y1F, Y1S, Y1F, Y3F, Y3S,
    // Professional Electives — ENGR1050 (Y2F), upper MEAM ×2 (Y4F), tech ×2 (Y4F/Y4S), freshman tech (Y1S)
    Y2F, Y4F, Y4S, Y1S,
    // General Electives — EAS2030 (Y2S), writing (Y1S), SS/Hum (Y1F), SS/Hum (Y3F), TBS (Y4)
    Y2S, Y1S, Y1F, Y3F, Y4F,
    // Concentration — 3020/3540 (Y3F), 3210/3330 (Y3S), upper MEAM (Y4F/Y4S)
    Y3F, Y3S, Y4F, Y4S,
];

fn with_concentration_category(req: Requirement) -> Requirement {
    match req {
        Requirement::SingleCourse { possibilities, .. } => Requirement::SingleCourse {
            category: Some("Concentration".to_string()),
            possibilities,
        },
        Requirement::CourseGroup {
            number,
            possibilities,
            ..
        } => Requirement::CourseGroup {
            category: Some("Concentration".to_string()),
            number,
            possibilities,
        },
        Requirement::Restriction {
            department,
            cu,
            level,
            max_level,
            attr,
            excluding,
            number,
            no_school,
            ..
        } => Requirement::Restriction {
            category: Some("Concentration".to_string()),
            department,
            cu,
            level,
            max_level,
            attr,
            excluding,
            number,
            no_school,
        },
        other => other,
    }
}

/// First four concentration courses as separate top-level requirements (1 CU each).
fn meam_concentration_slots(
    concentration_name: &str,
    meam_concentrations: &BTreeMap<String, Vec<Requirement>>,
) -> Vec<Requirement> {
    meam_concentrations
        .get(concentration_name)
        .expect("MEAM concentration")
        .iter()
        .take(4)
        .cloned()
        .map(with_concentration_category)
        .collect()
}

pub fn create_meam_major(concentration_name: String) -> Major {

    let meam_concentrations = BTreeMap::from([
            (
                "Dynamics, Controls, and Robotics".to_string(), 
                vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3200".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3210".to_string()] },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(3000), max_level: None, attr: None, excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(5000), max_level: None, attr: None, excluding: None, number: 1, no_school: None },

                    Requirement::Restriction { category: None, department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None  }, // tech elective from before
                ]
            ),
            (
                "Energy, Fluids and Thermal Systems".to_string(), 
                vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3020".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3330".to_string()] },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(3000), max_level: None, attr: None, excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(5000), max_level: None, attr: None, excluding: None, number: 1, no_school: None },

                    Requirement::Restriction { category: None, department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None  }, // tech elective from before
                ]
            ),
            (
                "Mechanics of Materials, Structures and Design".to_string(), 
                vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3210".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3540".to_string()] },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(3000), max_level: None, attr: None, excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(5000), max_level: None, attr: None, excluding: None, number: 1, no_school: None },

                    Requirement::Restriction { category: None, department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None }, // tech elective from before
                ]
            ),
            (
                "General".to_string(), 
                vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3020".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3210".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3330".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3540".to_string()] },

                    Requirement::Restriction { category: None, department: None, cu: None, level: Some(5000), max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, no_school: None, excluding: None }, // tech elective from before (one must be upper level if general conc.)
                ]
            ),
        ]);

    let conc_slots = meam_concentration_slots(&concentration_name, &meam_concentrations);

    let mut requirements = vec![
            // MEAM Core
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 2020".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 2030".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 2100".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 2110".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 2470".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 2480".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 3470".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 3480".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 4450".to_string()] },
            Requirement::SingleCourse { category: Some("MEAM Core".to_string()), possibilities: vec!["MEAM 4460".to_string()] },
            
            // Math and Natural Science
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 2400".to_string(), "ESE 2030".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ENM 2510".to_string(), "MATH 2410".to_string()] },
            Requirement::AnyOf { category: Some("Math and Natural Science".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0150".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1470".to_string()] },
                ] }
            ] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["PHYS 0151".to_string(), "ESE 1120".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CHEM 1012".to_string(), "BIOL 1121".to_string()] },
            Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUMA".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None },

            // Professional Electives
            Requirement::SingleCourse { category: Some("Professional Electives".to_string()), possibilities: vec!["ENGR 1050".to_string(), "CIS 1100".to_string(), "CIS 1200".to_string()] },
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(5000), max_level: None, attr: None, number: 2, excluding: Some(vec!["MEAM 5990".to_string()]), no_school: None },
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 2, excluding: None, no_school: None }, // one tech elective in concentration section
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: Some(2000), max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None }, // at max 3 prof. electives including first one at 1000 level

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["EAS 2030".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUSS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string()]), excluding: None, number: 2, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUSS".to_string(), "EUHS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUSS".to_string(), "EUHS".to_string(), "EUTB".to_string()]), excluding: None, number: 2, no_school: None },
    ];
    requirements.extend(conc_slots);

    return Major {
        short_name: "MEAM".to_string(),
        name: "Mechanical Engineering".to_string(),
        requirements,
        schedule_hints: seas_schedule_hints(&MEAM_SCHEDULE),
        concentrations: Some(meam_concentrations),
    };
}

/// MSE schedule template — one semester per top-level requirement, in list order.
const MSE_SCHEDULE: [Semester; 38] = [
    Y1F, Y2F, Y2S, Y2S, Y2F, Y2S, Y3F, Y3F, Y3F, Y3S, Y3S, Y3S, Y4F, Y4F, Y4S, // Engineering
    Y1F, Y1S, Y2F, Y2S, Y2S, Y1F, Y1S, Y2S, Y1F, Y1S, Y2F,                // Math and Natural Science
    Y2F, Y3F, Y3S, Y3S, Y4F,                                                     // Technical Electives
    Y2F, Y2S, Y3F, Y3S, Y4F, Y4S,                                               // General Electives
    Y4S,                                                                         // Free Elective
];

pub fn create_mse_major() -> Major {
        return Major {
        short_name: "MSE".to_string(),
        name: "Materials Science and Engineering".to_string(),
        requirements: vec![
            // Engineering
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 1010".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 2010".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 2020".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 2150".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 2200".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 2600".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 3010".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 3300".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 3600".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 3930".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 4050".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 4400".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 4600".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 4950".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["MSE 4960".to_string()] },

            // Math and Natural Science
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 2400".to_string(), "ESE 2030".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 2410".to_string(), "ENM 2510".to_string()] },
            Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUMA".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["PHYS 0140".to_string(), "MEAM 1100".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["PHYS 0141".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CHEM 1012".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CHEM 1101".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CHEM 1022".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MSE 2210".to_string()] },

            // Technical Electives
            Requirement::SingleCourse { category: Some("Technical Electives".to_string()), possibilities: vec!["ENGR 1050".to_string()] },
            Requirement::Restriction { category: Some("Technical Electives - MSE Elective".to_string()), department: Some(vec!["MSE".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives - MSE Elective".to_string()), department: Some(vec!["MSE".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["EAS 2030".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },

            Requirement::Restriction { category: Some("Free Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None }
        ],
        schedule_hints: seas_schedule_hints(&MSE_SCHEDULE),
        concentrations: Some(BTreeMap::from([
            (
                "Biomaterials and Biomimetics".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Biomaterials and Biomimetics".to_string()), possibilities: vec!["MSE 4300".to_string()] },
                    Requirement::SingleCourse { category: Some("Biomaterials and Biomimetics".to_string()), possibilities: vec!["MSE 5850".to_string(), "BE 5850".to_string()] },
                    Requirement::SingleCourse { category: Some("Biomaterials and Biomimetics".to_string()), possibilities: vec!["MSE 5180".to_string(), "MSE 4650".to_string(), "MSE 5650".to_string(), "MSE 0099".to_string(), "CBE 5110".to_string(), "CBE 5640".to_string(), "BE 2200".to_string(), "BE 5120".to_string(), "PHYS 2280".to_string()] },
                    Requirement::SingleCourse { category: Some("Biomaterials and Biomimetics".to_string()), possibilities: vec!["MSE 5180".to_string(), "MSE 4650".to_string(), "MSE 5650".to_string(), "MSE 0099".to_string(), "CBE 5110".to_string(), "CBE 5640".to_string(), "BE 2200".to_string(), "BE 5120".to_string(), "PHYS 2280".to_string()] },
                ],
            ),
            (
                "Electronic & Optical Devices and Sensors".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Electronic & Optical Devices and Sensors".to_string()), possibilities: vec!["MSE 5360".to_string()] },
                    Requirement::SingleCourse { category: Some("Electronic & Optical Devices and Sensors".to_string()), possibilities: vec!["MSE 4650".to_string(), "MSE 5650".to_string()] },
                    Requirement::SingleCourse { category: Some("Electronic & Optical Devices and Sensors".to_string()), possibilities: vec!["MSE 5250".to_string(), "MSE 6400".to_string(), "MSE 0099".to_string(), "ESE 2150".to_string(), "ESE 2180".to_string(), "ESE 5100".to_string(), "ESE 5210".to_string(), "ESE 5230".to_string()] },
                    Requirement::SingleCourse { category: Some("Electronic & Optical Devices and Sensors".to_string()), possibilities: vec!["MSE 5250".to_string(), "MSE 6400".to_string(), "MSE 0099".to_string(), "ESE 2150".to_string(), "ESE 2180".to_string(), "ESE 5100".to_string(), "ESE 5210".to_string(), "ESE 5230".to_string()] },
                ],
            ),
            (
                "Energy and Sustainability".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Energy and Sustainability".to_string()), possibilities: vec!["MSE 4550".to_string()] },
                    Requirement::SingleCourse { category: Some("Energy and Sustainability".to_string()), possibilities: vec!["MSE 5450".to_string()] },
                    Requirement::SingleCourse { category: Some("Energy and Sustainability".to_string()), possibilities: vec!["MSE 5020".to_string(), "MSE 0099".to_string(), "CBE 5450".to_string(), "EAS 3010".to_string(), "EAS 4010".to_string(), "EAS 4020".to_string(), "EAS 4030".to_string(), "MEAM 5020".to_string(), "MEAM 5030".to_string(), "MEAM 5800".to_string()] },
                    Requirement::SingleCourse { category: Some("Energy and Sustainability".to_string()), possibilities: vec!["MSE 5020".to_string(), "MSE 0099".to_string(), "CBE 5450".to_string(), "EAS 3010".to_string(), "EAS 4010".to_string(), "EAS 4020".to_string(), "EAS 4030".to_string(), "MEAM 5020".to_string(), "MEAM 5030".to_string(), "MEAM 5800".to_string()] },
                ],
            ),
            (
                "Nanotechnology".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Nanotechnology".to_string()), possibilities: vec!["MSE 5250".to_string()] },
                    Requirement::SingleCourse { category: Some("Nanotechnology".to_string()), possibilities: vec!["MSE 4650".to_string(), "MSE 5650".to_string()] },
                    Requirement::SingleCourse { category: Some("Nanotechnology".to_string()), possibilities: vec!["MSE 0099".to_string(), "MSE 6100".to_string(), "MSE 5360".to_string(), "ESE 3360".to_string(), "ESE 4230".to_string(), "ESE 5360".to_string(), "ESE 6210".to_string(), "MEAM 5290".to_string()] },
                    Requirement::SingleCourse { category: Some("Nanotechnology".to_string()), possibilities: vec!["MSE 0099".to_string(), "MSE 6100".to_string(), "MSE 5360".to_string(), "ESE 3360".to_string(), "ESE 4230".to_string(), "ESE 5360".to_string(), "ESE 6210".to_string(), "MEAM 5290".to_string()] },
                ],
            ),
        ])),
    };
}

/// CIS schedule template — one semester per top-level requirement, in list order.
/// Based on the official CIS BSE sample four-year plan.
const CIS_SCHEDULE: [Semester; 36] = [
    // Engineering core — 1100/1200 (Y1F), 1210 (Y1S), 2620 (Y2F), 2400 (Y2S), 3200/4480 (Y3F), 4710 (Y3S), 4000/4010 (Y4)
    Y1F, Y1F, Y1S, Y2F, Y2F, Y3F, Y3F, Y3S, Y4F, Y4S,
    // CIS Electives — soph fall/spring, junior fall/spring
    Y2F, Y2S, Y3F, Y3S,
    // Math and Natural Science — 1400 (Y1F), 1410 (Y1S), 2400 (Y2F), Math/NS (Y4F), PHYS151 (Y2F), 1600 (Y1F), PHYS150 (Y1S), STAT4300 (Y2S)
    Y1F, Y1S, Y2F, Y4F, Y2F, Y1F, Y1S, Y1S,
    // Technical Electives — soph spring, junior fall/spring, senior fall/spring (×2)
    Y2S, Y3F, Y3S, Y4F, Y4S, Y4S,
    // General Electives — Engr Ethics (Y3S), writing (Y1F), SSH/TBS across Y1–Y4
    Y3S, Y1F, Y1S, Y2F, Y2S, Y3S, Y4F,
    // Free Elective
    Y4S,
];

pub fn create_cis_major() -> Major {
    return Major {
        short_name: "CIS".to_string(),
        name: "Computer Science, BSE".to_string(),
        requirements: vec![
            // Engineering
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1100".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1200".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1210".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 2400".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 2620".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 3200".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 4480".to_string(), "CIS 5480".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 4710".to_string(), "CIS 5710".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 4000".to_string(), "CIS 4100".to_string(), ] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 4010".to_string(), "CIS 4110".to_string(), ] },

            Requirement::AnyOf {
                category: Some("Engineering".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["NETS".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
                ]
            },
            Requirement::AnyOf {
                category: Some("Engineering".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["NETS".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
                ]
            },
            Requirement::AnyOf {
                category: Some("Engineering".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["NETS".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
                ]
            },
            Requirement::AnyOf {
                category: Some("Engineering".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["NETS".to_string()]), cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
                ]
            },
            
            // Math and Natural Science
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 2400".to_string(), "ESE 2030".to_string(),"MATH 2600".to_string(), "MATH 3120".to_string(), "MATH 3130".to_string(), "MATH 3140".to_string()] },
            Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None },
            
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["PHYS 0151".to_string(), "PHYS 0171".to_string(), "ESE 1120".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CIS 1600".to_string()] },
            
            Requirement::AnyOf { category: Some("Math and Natural Science".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0150".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1470".to_string()] },
                ] }
            ] },

            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CIS 2610".to_string(), "ESE 3010".to_string(), "STAT 4300".to_string()] },

            // Technical Electives
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUCU".to_string(), "EUCR".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUCU".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUCU".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUCU".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUCU".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUCU".to_string()]), number: 1, excluding: None, no_school: None },

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["EAS 2030".to_string(), "CIS 4230".to_string(), "CIS 5230".to_string(), "LAWM 5060".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            
            // Free Elective
            Requirement::Restriction { category: Some("Free Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None }
        ],
        schedule_hints: seas_schedule_hints(&CIS_SCHEDULE),
        concentrations: Some(BTreeMap::from([
            (
                "Computer Vision".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Computer Vision".to_string()), possibilities: vec!["CIS 5800".to_string()] },
                    Requirement::SingleCourse { category: Some("Computer Vision".to_string()), possibilities: vec!["CIS 5810".to_string()] },
                    Requirement::SingleCourse { category: Some("Computer Vision".to_string()), possibilities: vec!["CIS 6800".to_string()] },
                    Requirement::SingleCourse { category: Some("Computer Vision".to_string()), possibilities: vec!["CIS 5200".to_string()] },
                ],
            ),
            (
                "Systems".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Systems".to_string()), possibilities: vec!["NETS 2120".to_string(), "CIS 3310".to_string(), "CIS 4510".to_string(), "CIS 5510".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4500".to_string(), "CIS 5500".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string(), "CIS 5530".to_string()] },
                    Requirement::SingleCourse { category: Some("Systems".to_string()), possibilities: vec!["NETS 2120".to_string(), "CIS 3310".to_string(), "CIS 4510".to_string(), "CIS 5510".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4500".to_string(), "CIS 5500".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string(), "CIS 5530".to_string()] },
                    Requirement::SingleCourse { category: Some("Systems".to_string()), possibilities: vec!["NETS 2120".to_string(), "CIS 3310".to_string(), "CIS 4510".to_string(), "CIS 5510".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4500".to_string(), "CIS 5500".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string(), "CIS 5530".to_string()] },
                    Requirement::SingleCourse { category: Some("Systems".to_string()), possibilities: vec!["NETS 2120".to_string(), "CIS 3310".to_string(), "CIS 4510".to_string(), "CIS 5510".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4500".to_string(), "CIS 5500".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string(), "CIS 5530".to_string()] },
                    Requirement::SingleCourse { category: Some("Systems".to_string()), possibilities: vec!["NETS 2120".to_string(), "CIS 3310".to_string(), "CIS 4510".to_string(), "CIS 5510".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4500".to_string(), "CIS 5500".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string(), "CIS 5530".to_string()] },
                ],
            ),
            (
                "Artificial Intelligence".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Artificial Intelligence".to_string()), possibilities: vec!["CIS 4210".to_string(), "CIS 5210".to_string()] },
                    Requirement::SingleCourse { category: Some("Artificial Intelligence".to_string()), possibilities: vec!["CIS 4190".to_string(), "CIS 5190".to_string(), "CIS 5200".to_string()] },
                    Requirement::SingleCourse { category: Some("Artificial Intelligence".to_string()), possibilities: vec!["MEAM 5100".to_string(), "MEAM 5200".to_string(), "CIS 5220".to_string(), "CIS 5300".to_string(), "CIS 5800".to_string(), "CIS 5810".to_string(), "NETS 2130".to_string()] },
                    Requirement::SingleCourse { category: Some("Artificial Intelligence".to_string()), possibilities: vec!["MEAM 5100".to_string(), "MEAM 5200".to_string(), "CIS 5220".to_string(), "CIS 5300".to_string(), "CIS 5800".to_string(), "CIS 5810".to_string(), "NETS 2130".to_string()] },
                ],
            ),
            (
                "Software Foundations".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Software Foundations".to_string()), possibilities: vec!["CIS 3410".to_string()] },
                    Requirement::SingleCourse { category: Some("Software Foundations".to_string()), possibilities: vec!["CIS 5000".to_string()] },
                    Requirement::SingleCourse { category: Some("Software Foundations".to_string()), possibilities: vec!["CIS 3500".to_string(), "CIS 5400".to_string(), "CIS 5470".to_string(), "CIS 5520".to_string(), "CIS 6730".to_string(), "CIS 6820".to_string()] },
                    Requirement::SingleCourse { category: Some("Software Foundations".to_string()), possibilities: vec!["CIS 3500".to_string(), "CIS 5400".to_string(), "CIS 5470".to_string(), "CIS 5520".to_string(), "CIS 6730".to_string(), "CIS 6820".to_string()] },
                ],
            ),
            (
                "Data Science".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["CIS 4190".to_string(), "CIS 5190".to_string(), "CIS 5200".to_string(), "ESE 5450".to_string(), "STAT 4710".to_string()] },
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["CIS 5450".to_string(), "ESE 3050".to_string()] },
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["ENM 3210".to_string(), "ESE 4020".to_string(), "STAT 4310".to_string()] },
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["CIS 4500".to_string(), "CIS 5500".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 4210".to_string(), "CIS 5210".to_string(), "CIS 5220".to_string(), "CIS 5800".to_string(), "STAT 4350".to_string(), "STAT 4740".to_string(), "STAT 4760".to_string(), "STAT 5200".to_string()] },
                ],
            ),
            (
                "Cognitive Science".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Cognitive Science".to_string()), possibilities: vec!["COGS 1001".to_string(), "CIS 1400".to_string()] },
                    Requirement::SingleCourse { category: Some("Cognitive Science".to_string()), possibilities: vec!["CIS 4210".to_string(), "CIS 5210".to_string(), "CIS 5200".to_string(), "CIS 5300".to_string()] },
                    Requirement::SingleCourse { category: Some("Cognitive Science".to_string()), possibilities: vec!["LING 0500".to_string(), "LING 2300".to_string(), "LING 2500".to_string(), "LING 2700".to_string()] },
                    Requirement::SingleCourse { category: Some("Cognitive Science".to_string()), possibilities: vec!["PSYC 1210".to_string(), "PSYC 1340".to_string(), "PSYC 1230".to_string(), "PSYC 1310".to_string(), "PSYC 2737".to_string(), "PSYC 2377".to_string()] },
                    Requirement::SingleCourse { category: Some("Cognitive Science".to_string()), possibilities: vec!["PHIL 1710".to_string(), "PHIL 2640".to_string(), "PHIL 4721".to_string(), "PHIL 4840".to_string()] },
                ],
            ),
            (
                "Computational Biology".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Computational Biology".to_string()), possibilities: vec!["BIOL 1101".to_string(), "BIOL 1121".to_string()] },
                    Requirement::SingleCourse { category: Some("Computational Biology".to_string()), possibilities: vec!["BIOL 2210".to_string()] },
                    Requirement::SingleCourse { category: Some("Computational Biology".to_string()), possibilities: vec!["ENM 3210".to_string(), "ESE 4020".to_string(), "STAT 4310".to_string(), "BIOL 2510".to_string()] },
                    Requirement::SingleCourse { category: Some("Computational Biology".to_string()), possibilities: vec!["CIS 5450".to_string(), "ESE 3050".to_string(), "CIS 4190".to_string(), "CIS 5190".to_string(), "CIS 5200".to_string(), "ESE 5450".to_string(), "STAT 4710".to_string()] },
                    Requirement::SingleCourse { category: Some("Computational Biology".to_string()), possibilities: vec!["BIOL 4231".to_string(), "BE 5690".to_string(), "BE 4800".to_string(), "BE 3060".to_string()] },
                    Requirement::SingleCourse { category: Some("Computational Biology".to_string()), possibilities: vec!["BIOL 4526".to_string(), "CIS 4360".to_string(), "BIOL 5536".to_string(), "GCB 5360".to_string(), "BIOL 5571".to_string()] },
                ],
            ),
        ])),
    };
}

fn cis_or_nets_elective(category: &str) -> Requirement {
    Requirement::Restriction {
        category: Some("CIS Elective".to_string()),
        department: Some(vec!["CIS".to_string(), "NETS".to_string()]),
        cu: None,
        level: None,
        max_level: None,
        attr: None,
        excluding: Some(vec!["CIS 1050".to_string(), "CIS 1060".to_string(), "CIS 1070".to_string(), "CIS 1250".to_string(), "CIS 1600".to_string(), "CIS 2610".to_string(), "CIS 3333".to_string(), "CIS 4230".to_string(), "CIS 5230".to_string(), "CIS 7980".to_string()]),
        number: 1,
        no_school: None,
    }
}

const DMD_ELECTIVE_DEPTS: &[&str] = &[
    "COMM", "FNAR", "CIMS", "DSGN", "THAR", "MKTG", "ARTH", "IPD", "MUSC", "EDUC",
];

fn dmd_advisor_elective() -> Requirement {
    Requirement::Restriction {
        category: Some("DMD Electives".to_string()),
        department: Some(DMD_ELECTIVE_DEPTS.iter().map(|d| (*d).to_string()).collect()),
        cu: None,
        level: None,
        max_level: None,
        attr: None,
        excluding: None,
        number: 1,
        no_school: None,
    }
}

/// DMD schedule template — one semester per top-level requirement, in list order.
/// Based on the official Digital Media Design BSE sample four-year plan.
const DMD_SCHEDULE: [Semester; 35] = [
    // Engineering
    Y1F, Y1S, Y2F, Y2S, Y2F, Y3F, Y2S, Y3F, Y4S, Y2F, Y2S, Y3F, Y4F,
    // Math and Natural Science
    Y1F, Y1S, Y2F, Y1S, Y2S, Y2F, Y3S, Y2S,
    // DMD Electives
    Y1F, Y1S, Y4S, Y3F, Y3S, Y4F,
    // General Electives (5 SSH + 2 SSH/TBS)
    Y1F, Y1S, Y2F, Y2S, Y3S, Y4F, Y4S,
    // Free Elective
    Y4S,
];

fn dmd_schedule_hints() -> HashMap<String, ScheduleHint> {
    let mut hints = schedule_hints_from_array(&DMD_SCHEDULE);
    hints.insert("CIS 4970".to_string(), Y4S.into());
    hints
}

pub fn create_dmd_major() -> Major {
    Major {
        short_name: "DMD".to_string(),
        name: "Digital Media Design".to_string(),
        requirements: vec![
            // Engineering
            Requirement::AnyOf {
                category: Some("Engineering".to_string()),
                possibilities: vec![
                    Requirement::SingleCourse {
                        category: None,
                        possibilities: vec!["CIS 1100".to_string()],
                    },
                    cis_or_nets_elective("Engineering"),
                ],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["CIS 1200".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["CIS 1210".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["CIS 2400".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["CIS 2620".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["CIS 3200".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["CIS 4600".to_string(), "CIS 5600".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["CIS 4610".to_string(), "CIS 5610".to_string(), "CIS 4620".to_string(), "CIS 5620".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["CIS 4970".to_string()],
            },
            cis_or_nets_elective("Engineering"),
            cis_or_nets_elective("Engineering"),
            cis_or_nets_elective("Engineering"),
            cis_or_nets_elective("Engineering"),

            // Math and Natural Science
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["MATH 1400".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["MATH 2400".to_string(), "ESE 2030".to_string(), "ENM 2030".to_string(), "ENM 2400".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["CIS 1600".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec![
                    "CIS 2610".to_string(),
                    "ESE 3010".to_string(),
                    "STAT 4300".to_string(),
                ],
            },
            Requirement::AnyOf {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec![
                    Requirement::SingleCourse {
                        category: None,
                        possibilities: vec!["PHYS 0150".to_string(), "PHYS 0170".to_string()],
                    },
                    Requirement::AllOf {
                        category: None,
                        requirements: vec![
                            Requirement::SingleCourse {
                                category: None,
                                possibilities: vec!["MEAM 1100".to_string()],
                            },
                            Requirement::SingleCourse {
                                category: None,
                                possibilities: vec!["MEAM 1470".to_string()],
                            },
                        ],
                    },
                ],
            },
            Requirement::AnyOf {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec![
                    Requirement::SingleCourse {
                        category: None,
                        possibilities: vec!["BIOL 1101".to_string()],
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
                                possibilities: vec!["BIOL 1124".to_string()],
                            },
                        ],
                    },
                    Requirement::AllOf {
                        category: None,
                        requirements: vec![
                            Requirement::SingleCourse {
                                category: None,
                                possibilities: vec!["CHEM 1012".to_string()],
                            },
                            Requirement::SingleCourse {
                                category: None,
                                possibilities: vec!["CHEM 1101".to_string()],
                            },
                        ],
                    },
                    Requirement::SingleCourse {
                        category: None,
                        possibilities: vec!["ESE 1120".to_string()],
                    },
                    Requirement::SingleCourse {
                        category: None,
                        possibilities: vec!["PHYS 0151".to_string()],
                    },
                    Requirement::SingleCourse {
                        category: None,
                        possibilities: vec!["PHYS 0171".to_string()],
                    },
                ],
            },
            Requirement::Restriction {
                category: Some("Math and Natural Science".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },

            // DMD Electives
            Requirement::SingleCourse {
                category: Some("DMD Electives".to_string()),
                possibilities: vec![
                    "FNAR 0010".to_string(),
                    "FNAR 2200".to_string(),
                    "FNAR 1080".to_string(),
                ],
            },
            Requirement::SingleCourse {
                category: Some("DMD Electives".to_string()),
                possibilities: vec!["DSGN 1030".to_string(), "DSGN 2010".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("DMD Electives".to_string()),
                possibilities: vec![
                    "DSGN 2040".to_string(),
                    "FNAR 1050".to_string(),
                    "FNAR 2090".to_string(),
                    "FNAR 2100".to_string(),
                ],
            },
            dmd_advisor_elective(),
            dmd_advisor_elective(),
            dmd_advisor_elective(),

            // General Electives — 5 SSH + 2 SSH/TBS (writing seminar via EUHS)
            Requirement::Restriction {
                category: Some("General Electives".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("General Electives".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("General Electives".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("General Electives".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("General Electives".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("General Electives".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("General Electives".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },

            // Free Elective
            Requirement::Restriction {
                category: Some("Free Elective".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
        ],
        schedule_hints: dmd_schedule_hints(),
        concentrations: None,
    }
}

/// AI schedule template — one semester per top-level requirement, in list order.
const AI_SCHEDULE: [Semester; 44] = [
    Y1F, Y1F, Y1S, Y1S, Y2F,                                     // Engineering
    Y1F, Y1S, Y2F, Y2S, Y2S, Y2F, Y2S, Y3F,                      // Math and Natural Science
    Y3F, Y3S, Y2S, Y3S, Y3S, Y3S,                                // AI core
    Y3S, Y3S, Y3S, Y3S, Y3S, Y3S,                                // AI Electives (set 1)
    Y4F, Y4F, Y4F, Y4F, Y4F, Y4F,                                // AI Electives (set 2)
    Y4F, Y4S,                                                     // Senior Design
    Y3F, Y3S, Y4F,                                                // Technical Electives
    Y2F, Y2S, Y3F, Y3S, Y4F, Y4S, Y4S,                           // General Electives
    Y4S,                                                         // Free Elective
];

pub fn create_ai_major() -> Major {
    return Major {
        short_name: "AI".to_string(),
        name: "Artificial Intelligence".to_string(),
        requirements: vec![
            // Engineering
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1100".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1200".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1210".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 2450".to_string(), "CIS 5450".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 3200".to_string()] },
            
            // Math and Natural Science
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 2030".to_string()] },
            Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNS".to_string()]), number: 1, excluding: None, no_school: None },
            
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CIS 1600".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 3010".to_string(), "STAT 4300".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 4020".to_string(), "ESE 5420".to_string()] },

            // AI
            Requirement::SingleCourse { category: Some("Artificial Intelligence - Introduction to AI".to_string()), possibilities: vec!["CIS 4210".to_string(), "CIS 5210".to_string(), "ESE 2000".to_string()] },
            Requirement::SingleCourse { category: Some("Artificial Intelligence - Machine Learning".to_string()), possibilities: vec!["CIS 4190".to_string(), "CIS 5190".to_string(), "CIS 5200".to_string()] },
            Requirement::SingleCourse { category: Some("Artificial Intelligence - Signals & Systems".to_string()), possibilities: vec!["ESE 2100".to_string(), "ESE 2240".to_string()] },
            Requirement::SingleCourse { category: Some("Artificial Intelligence - Optimization & Control".to_string()), possibilities: vec!["ESE 3040".to_string(), "ESE 4210".to_string()] },
            Requirement::SingleCourse { category: Some("Artificial Intelligence - Vision & Language".to_string()), possibilities: vec!["CIS 4300".to_string(), "CIS 5300".to_string(), "CIS 4810".to_string(), "CIS 5810".to_string()] },
            Requirement::SingleCourse { category: Some("Artificial Intelligence - AI Project".to_string()), possibilities: vec!["CIS 3500".to_string(), "CIS 4300".to_string(), "CIS 5300".to_string(), "CIS 4810".to_string(), "CIS 5810".to_string(), "ESE 3060".to_string(), "ESE 3600".to_string(), "ESE 4210".to_string(), "NETS 2120".to_string(), "NETS 2130".to_string()] },

            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"].map(|x| x.to_string()).to_vec()},
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"].map(|x| x.to_string()).to_vec()},
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"].map(|x| x.to_string()).to_vec()},
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"].map(|x| x.to_string()).to_vec()},
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"].map(|x| x.to_string()).to_vec()},
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"].map(|x| x.to_string()).to_vec()},

            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"].map(|x| x.to_string()).to_vec() },
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"].map(|x| x.to_string()).to_vec() },
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"].map(|x| x.to_string()).to_vec() },
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"].map(|x| x.to_string()).to_vec() },
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"].map(|x| x.to_string()).to_vec() },
            Requirement::SingleCourse { category: Some("AI Electives".to_string()), possibilities: ["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"].map(|x| x.to_string()).to_vec() },

            // Senior Design
            Requirement::SingleCourse { category: Some("Senior Design".to_string()), possibilities: ["CIS 4000", "CIS 4100", "ESE 4500", "MEAM 4450", "BE 4950", "MSE 4950", "CBE 4000"].map(|x| x.to_string()).to_vec()},
            Requirement::SingleCourse { category: Some("Senior Design".to_string()), possibilities: ["CIS 4010", "CIS 4110", "ESE 4510", "MEAM 4460", "BE 4960", "MSE 4960", "CBE 4590"].map(|x| x.to_string()).to_vec()},
            
            // Technical Electives
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            
            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["LAWM 5060".to_string(), "CIS 4230".to_string(), "CIS 5230".to_string()] },
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["COGS 1001".to_string(), "LING 0500".to_string(), "LING 2500".to_string(), "LING 3810".to_string(), "PHIL 1710".to_string(), "PHIL 2640".to_string(), "PHIL 4721".to_string(), "PHIL 4840".to_string(), "PSYC 1210".to_string(), "PSYC 1340".to_string(), "PSYC 1230".to_string(), "PSYC 1310".to_string(), "PSYC 2737".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            
            // Free Elective
            Requirement::Restriction { category: Some("Free Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None }
        ],
        schedule_hints: seas_schedule_hints(&AI_SCHEDULE),
        concentrations: Some(BTreeMap::from([
            
            
        ])),
    };
}

/// CMPE schedule template — one semester per top-level requirement, in list order.
const CMPE_SCHEDULE: [Semester; 35] = [
    Y1F, Y1F, Y1F, Y1S, Y1S, Y2F, Y2F, Y2S, Y2S, Y3F, Y3F, // Engineering
    Y2S, Y3F, Y3S,                                            // Electives
    Y4F, Y4S,                                                 // Design and Project
    Y1F, Y1S, Y2F, Y2S, Y2F, Y2S, Y2F, Y1S,                  // Math and Natural Science
    Y3F, Y3S, Y4S,                                            // Professional Electives
    Y2F, Y2S, Y3F, Y3S, Y4F, Y4S, Y4S,                       // General Electives
    Y4S,                                                      // Free Elective
];

pub fn create_cmpe_major() -> Major {
        return Major {
        short_name: "CMPE".to_string(),
        name: "Computer Engineering".to_string(),
        requirements: vec![
            // Engineering
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 1110".to_string(), "ESE 3600".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1100".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1200".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1210".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 2150".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 2400".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 3500".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 3700".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 4480".to_string(), "CIS 5480".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 4710".to_string(), "CIS 5710".to_string()] },
            
            // Intermediate CIS or ESE Elective
            Requirement::Restriction { 
                category: Some("Intermediate CIS or ESE Elective".to_string()), 
                department: Some(vec!["ESE".to_string(), "CIS".to_string(), ]), number: 1, cu: None, level: Some(2000), max_level: None, attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), 
                no_school: None 
            },
            
            // Advanced CIS or ESE courses
            Requirement::Restriction { 
                category: Some("Advanced CIS or ESE Electives".to_string()), 
                department: Some(vec!["ESE".to_string(), "CIS".to_string()]), number: 1, cu: None, level: Some(3000), max_level: None, attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            },
            Requirement::Restriction { 
                category: Some("Advanced CIS or ESE Electives".to_string()), 
                department: Some(vec!["ESE".to_string(), "CIS".to_string()]), number: 1, cu: None, level: Some(3000), max_level: None, attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            },

            // Design and Project courses
            Requirement::SingleCourse { category: Some("Design and Project courses".to_string()), possibilities: vec!["CIS 4000".to_string(), "ESE 4500".to_string()] },
            Requirement::SingleCourse { category: Some("Design and Project courses".to_string()), possibilities: vec!["CIS 4010".to_string(), "ESE 4510".to_string()] },

            // Math and Natural Science
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 2030".to_string(), "MATH 2400".to_string(), "MATH 2600".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 3010".to_string(), "CIS 2610".to_string(), "STAT 4300".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CIS 1600".to_string()] },
            Requirement::AnyOf {
                category: Some("Math and Natural Science".to_string()), 
                possibilities: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0150".to_string()] },    
                    Requirement::AllOf {
                        category: None, 
                        requirements: vec![
                            Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1100".to_string()] },
                            Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1470".to_string(), "BIOL 1124".to_string(), "PHYS 0050".to_string(), "CHEM 1101".to_string()] },
                        ]
                    },
                    Requirement::AllOf {
                        category: None, 
                        requirements: vec![
                            Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0140".to_string()] },
                            Requirement::SingleCourse { category: None, possibilities: vec!["BIOL 1124".to_string(), "PHYS 0050".to_string(), "MEAM 1470".to_string(), "CHEM 1101".to_string()] },
                        ]
                    },
                    Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0170".to_string()] },
                ]
            },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 1120".to_string()] },
            Requirement::SingleCourse { 
                category: Some("Math and Natural Science".to_string()), 
                possibilities: vec![
                    "CHEM 1012".to_string(),
                    "EAS 0091".to_string(),
                    "BIOL 1121".to_string(),
                    "BIOL 1101".to_string(),
                    "PHYS 1240".to_string()
                ]
            },
            Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None },

            // Professional Electives
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::AnyOf {
                category: Some("Professional Electives".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 4000".to_string(), "EAS 5450".to_string(), "ESE 5950".to_string(), "MGMT 2370".to_string(), "OIDD 2360".to_string()] },
                ]
            },

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["LAWM 5060".to_string(), "EAS 2030".to_string(), "CIS 4230".to_string(), "CIS 5230".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, max_level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },

            // Free Elective
            Requirement::Restriction { category: Some("Free Elective".to_string()), department: None, cu: None, level: None, max_level: None, attr: None, number: 1, excluding: None, no_school: None },
        ],
        schedule_hints: seas_schedule_hints(&CMPE_SCHEDULE),
        concentrations: Some(BTreeMap::from([
            (
                "AI & Robotics".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("AI & Robotics".to_string()), possibilities: vec!["CIS 2450".to_string(), "CIS 4190".to_string(), "CIS 5190".to_string(), "CIS 5200".to_string(), "CIS 4300".to_string(), "CIS 5300".to_string(), "CIS 4810".to_string(), "CIS 5810".to_string(), "CIS 5210".to_string(), "CIS 5650".to_string(), "ESE 2000".to_string(), "ESE 2240".to_string(), "ESE 3060".to_string(), "ESE 3600".to_string(), "ESE 4210".to_string(), "ESE 5050".to_string(), "ESE 5390".to_string(), "ESE 6150".to_string(), "ESE 6500".to_string(), "MEAM 5200".to_string()] },
                    Requirement::SingleCourse { category: Some("AI & Robotics".to_string()), possibilities: vec!["CIS 2450".to_string(), "CIS 4190".to_string(), "CIS 5190".to_string(), "CIS 5200".to_string(), "CIS 4300".to_string(), "CIS 5300".to_string(), "CIS 4810".to_string(), "CIS 5810".to_string(), "CIS 5210".to_string(), "CIS 5650".to_string(), "ESE 2000".to_string(), "ESE 2240".to_string(), "ESE 3060".to_string(), "ESE 3600".to_string(), "ESE 4210".to_string(), "ESE 5050".to_string(), "ESE 5390".to_string(), "ESE 6150".to_string(), "ESE 6500".to_string(), "MEAM 5200".to_string()] },
                    Requirement::SingleCourse { category: Some("AI & Robotics".to_string()), possibilities: vec!["CIS 2450".to_string(), "CIS 4190".to_string(), "CIS 5190".to_string(), "CIS 5200".to_string(), "CIS 4300".to_string(), "CIS 5300".to_string(), "CIS 4810".to_string(), "CIS 5810".to_string(), "CIS 5210".to_string(), "CIS 5650".to_string(), "ESE 2000".to_string(), "ESE 2240".to_string(), "ESE 3060".to_string(), "ESE 3600".to_string(), "ESE 4210".to_string(), "ESE 5050".to_string(), "ESE 5390".to_string(), "ESE 6150".to_string(), "ESE 6500".to_string(), "MEAM 5200".to_string()] },
                    Requirement::SingleCourse { category: Some("AI & Robotics".to_string()), possibilities: vec!["CIS 2450".to_string(), "CIS 4190".to_string(), "CIS 5190".to_string(), "CIS 5200".to_string(), "CIS 4300".to_string(), "CIS 5300".to_string(), "CIS 4810".to_string(), "CIS 5810".to_string(), "CIS 5210".to_string(), "CIS 5650".to_string(), "ESE 2000".to_string(), "ESE 2240".to_string(), "ESE 3060".to_string(), "ESE 3600".to_string(), "ESE 4210".to_string(), "ESE 5050".to_string(), "ESE 5390".to_string(), "ESE 6150".to_string(), "ESE 6500".to_string(), "MEAM 5200".to_string()] },
                ],
            ),
            (
                "CHIPs".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Chips".to_string()), possibilities: vec!["ESE 3190".to_string(), "ESE 4190".to_string(), "ESE 5720".to_string(), "ESE 4730".to_string(), "ESE 5730".to_string(), "ESE 4750".to_string(), "ESE 5750".to_string(), "ESE 5320".to_string(), "ESE 5780".to_string(), "ESE 5800".to_string(), "ESE 6680".to_string(), "CIS 6010".to_string()] },
                    Requirement::SingleCourse { category: Some("Chips".to_string()), possibilities: vec!["ESE 3190".to_string(), "ESE 4190".to_string(), "ESE 5720".to_string(), "ESE 4730".to_string(), "ESE 5730".to_string(), "ESE 4750".to_string(), "ESE 5750".to_string(), "ESE 5320".to_string(), "ESE 5780".to_string(), "ESE 5800".to_string(), "ESE 6680".to_string(), "CIS 6010".to_string()] },
                    Requirement::SingleCourse { category: Some("Chips".to_string()), possibilities: vec!["ESE 3190".to_string(), "ESE 4190".to_string(), "ESE 5720".to_string(), "ESE 4730".to_string(), "ESE 5730".to_string(), "ESE 4750".to_string(), "ESE 5750".to_string(), "ESE 5320".to_string(), "ESE 5780".to_string(), "ESE 5800".to_string(), "ESE 6680".to_string(), "CIS 6010".to_string()] },
                    Requirement::SingleCourse { category: Some("Chips".to_string()), possibilities: vec!["ESE 3190".to_string(), "ESE 4190".to_string(), "ESE 5720".to_string(), "ESE 4730".to_string(), "ESE 5730".to_string(), "ESE 4750".to_string(), "ESE 5750".to_string(), "ESE 5320".to_string(), "ESE 5780".to_string(), "ESE 5800".to_string(), "ESE 6680".to_string(), "CIS 6010".to_string()] },
                ],
            ),
            (
                "Networks".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Networks".to_string()), possibilities: vec!["NETS 2120".to_string(), "ESE 4070".to_string(), "ESE 5070".to_string(), "CIS 5530".to_string(), "ESE 6650".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string()] },
                    Requirement::SingleCourse { category: Some("Networks".to_string()), possibilities: vec!["NETS 2120".to_string(), "ESE 4070".to_string(), "ESE 5070".to_string(), "CIS 5530".to_string(), "ESE 6650".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string()] },
                    Requirement::SingleCourse { category: Some("Networks".to_string()), possibilities: vec!["NETS 2120".to_string(), "ESE 4070".to_string(), "ESE 5070".to_string(), "CIS 5530".to_string(), "ESE 6650".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string()] },
                    Requirement::SingleCourse { category: Some("Networks".to_string()), possibilities: vec!["NETS 2120".to_string(), "ESE 4070".to_string(), "ESE 5070".to_string(), "CIS 5530".to_string(), "ESE 6650".to_string(), "CIS 4550".to_string(), "CIS 5550".to_string(), "CIS 5050".to_string()] },
                ],
            ),
            (
                "Security and Safety".to_string(),
                vec![
                    Requirement::SingleCourse { category: Some("Security and Safety".to_string()), possibilities: vec!["CIS 2330".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4510".to_string(), "CIS 5470".to_string(), "CIS 5560".to_string(), "ESE 5370".to_string()] },
                    Requirement::SingleCourse { category: Some("Security and Safety".to_string()), possibilities: vec!["CIS 2330".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4510".to_string(), "CIS 5470".to_string(), "CIS 5560".to_string(), "ESE 5370".to_string()] },
                    Requirement::SingleCourse { category: Some("Security and Safety".to_string()), possibilities: vec!["CIS 2330".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4510".to_string(), "CIS 5470".to_string(), "CIS 5560".to_string(), "ESE 5370".to_string()] },
                    Requirement::SingleCourse { category: Some("Security and Safety".to_string()), possibilities: vec!["CIS 2330".to_string(), "CIS 4410".to_string(), "CIS 5410".to_string(), "CIS 4510".to_string(), "CIS 5470".to_string(), "CIS 5560".to_string(), "ESE 5370".to_string()] },
                ],
            ),
        ])),
    };
}

fn be_conc_courses(courses: &[&str]) -> Vec<String> {
    courses.iter().map(|c| c.to_string()).collect()
}

fn be_conc_slot(category: &str, courses: &[&str]) -> Requirement {
    Requirement::SingleCourse {
        category: Some(category.to_string()),
        possibilities: be_conc_courses(courses),
    }
}

/// Optional overlay concentration: 2 CU from `primary`, then 2 CU from `extended`.
fn be_overlay_concentration(
    name: &str,
    primary: &[&str],
    extended: &[&str],
) -> (String, Vec<Requirement>) {
    let cat = name.to_string();
    (
        cat.clone(),
        vec![
            be_conc_slot(&cat, primary),
            be_conc_slot(&cat, primary),
            be_conc_slot(&cat, extended),
            be_conc_slot(&cat, extended),
        ],
    )
}

fn be_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    BTreeMap::from([
        be_overlay_concentration(
            "Biomedical Data Science and Computational Medicine",
            &["BE 4900", "BE 5040", "BE 5210", "BE 5300", "BE 5590", "BE 5660"],
            &[
                "BE 4900", "BE 5040", "BE 5210", "BE 5300", "BE 5590", "BE 5660",
                "CIS 5210", "CIS 4500", "CIS 5200", "CIS 5450", "CIS 5350", "CIS 5360",
                "CBE 5250", "ESE 3050", "ESE 5420", "BIOM 5350", "MTR 5350", "BIOL 4511",
                "BIOL 5536", "GCB 5360", "GCB 5370", "STAT 9915",
            ],
        ),
        be_overlay_concentration(
            "Biomedical Devices",
            &[
                "BE 4700", "BE 4720", "BE 4900", "BE 5020", "BE 5140", "BE 5180",
                "BE 5210", "BE 5280", "BE 5290", "BE 5510", "BE 5560", "BE 5700",
            ],
            &[
                "BE 4700", "BE 4720", "BE 4900", "BE 5020", "BE 5140", "BE 5180",
                "BE 5210", "BE 5280", "BE 5290", "BE 5510", "BE 5560", "BE 5700",
                "ESE 2150", "ESE 5050", "MEAM 5130", "ESE 5290", "MEAM 1010", "MEAM 2010",
                "MEAM 5100", "MEAM 4150", "OIDD 4150", "MEAM 5140", "MEAM 5200",
            ],
        ),
        be_overlay_concentration(
            "Cellular/Tissue Engineering and Biomaterials",
            &[
                "BE 3300", "BE 4900", "BE 5120", "BE 5400", "BE 5530", "BE 5580",
                "BE 5650", "BE 5690", "BE 5780",
            ],
            &[
                "BE 3300", "BE 4900", "BE 5120", "BE 5400", "BE 5530", "BE 5580",
                "BE 5650", "BE 5690", "BE 5780", "CBE 4300", "MSE 4300", "CBE 5570",
                "MEAM 5140", "MSE 5850", "BE 5850", "MSE 5180",
            ],
        ),
        be_overlay_concentration(
            "Biomedical Imaging and Radiation Physics",
            &[
                "BE 4900", "BE 5180", "BE 5370", "BE 5470", "BE 5810", "BE 5830",
                "BE 6500",
            ],
            &[
                "BE 4900", "BE 5180", "BE 5370", "BE 5470", "BE 5810", "BE 5830",
                "BE 6500", "MPHY 6030", "MPHY 6070", "PHYS 4421",
            ],
        ),
        be_overlay_concentration(
            "Systems and Synthetic Biology",
            &[
                "BE 4900", "BE 5270", "BE 5400", "BE 5580", "BE 5590", "BE 5650",
                "BE 5690",
            ],
            &[
                "BE 4900", "BE 5270", "BE 5400", "BE 5580", "BE 5590", "BE 5650",
                "BE 5690", "CBE 4790", "CBE 4800", "CBE 5170", "CBE 5270", "CBE 5540",
                "CBE 5570", "MEAM 6630", "BIOL 5262",
            ],
        ),
        be_overlay_concentration(
            "Neuroengineering",
            &[
                "BE 4900", "BE 5060", "BE 5210", "BE 5300", "BE 5660", "BE 5850",
                "BE 5950", "BE 6100",
            ],
            &[
                "BE 4900", "BE 5060", "BE 5210", "BE 5300", "BE 5660", "BE 5850",
                "BE 5950", "BE 6100", "NRSC 2249", "PSYC 1230", "NRSC 2110", "BIOL 2110",
                "BIOL 4110", "BIOL 5110", "BIOL 4142", "NRSC 2205", "NRSC 3334",
                "NGG 5720", "NGG 5730",
            ],
        ),
        be_overlay_concentration(
            "Multiscale Biomechanics",
            &["BE 4900", "BE 5100", "BE 5140", "BE 5500", "BE 5700", "BE 5610"],
            &["BE 4900", "BE 5100", "BE 5140", "BE 5500", "BE 5700", "BE 5610"],
        ),
        be_overlay_concentration(
            "Therapeutics, Drug Delivery & Nanomedicine",
            &[
                "BE 4900", "BE 5550", "CBE 5550", "MEAM 5550", "BE 5570", "BE 5620",
                "BE 5780",
            ],
            &[
                "BE 4900", "BE 5550", "CBE 5550", "MEAM 5550", "BE 5570", "BE 5620",
                "BE 5780", "BIOL 4810", "CBE 5570", "CBE 5640",
            ],
        ),
        be_overlay_concentration(
            "Immune Engineering",
            &[
                "BE 4900", "BE 5120", "BE 4260", "BE 5260", "BIOL 4004", "BE 5270",
                "BE 5570",
            ],
            &[
                "BE 4900", "BE 5120", "BE 4260", "BE 5260", "BIOL 4004", "BE 5270",
                "BE 5570", "ENGR 4500", "IMUN 5060", "IMUN 5070", "IMUN 6090",
                "CAMB 6090", "REG 6180",
            ],
        ),
    ])
}

fn be_attr_constraint(label: &str, attrs: &[&str], count: i32, group: &str) -> PoolConstraint {
    PoolConstraint {
        requirement: Requirement::Restriction {
            category: Some(label.to_string()),
            department: None,
            cu: None,
            level: None,
            max_level: None,
            attr: Some(attrs.iter().map(|s| s.to_string()).collect()),
            excluding: None,
            number: 1,
            no_school: None,
        },
        count,
        consumption_group: Some(group.to_string()),
    }
}

fn be_ethics_constraint() -> PoolConstraint {
    PoolConstraint {
        requirement: Requirement::SingleCourse {
            category: Some("Ethics".to_string()),
            possibilities: vec![
                "EAS 2030".to_string(),
                "HSOC 1330".to_string(),
                "HSOC 2457".to_string(),
                "LGST 1000".to_string(),
                "LGST 2200".to_string(),
                "NURS 3300".to_string(),
                "NURS 5250".to_string(),
                "BIOE 4010".to_string(),
                "BIOE 4020".to_string(),
                "PHIL 1342".to_string(),
                "PHIL 4330".to_string(),
            ],
        },
        count: 1,
        consumption_group: Some("be:ethics".to_string()),
    }
}

/// 7 courses, 8 coverage requirements (ethics + 2 SS + 2 Hum + 1 flex SSH + 2 SSH/TBS).
/// Exactly one double-count is permitted — only ethics may overlap a distribution slot
/// (`be:ethics` vs shared `be:distribution`; no SSH/TBS cross-double-count).
fn be_general_electives_pool() -> Requirement {
    Requirement::CoursePool {
        category: Some("General Electives".to_string()),
        fixed_slots: vec![],
        flexible_slots: 7,
        constraints: vec![
            be_ethics_constraint(),
            be_attr_constraint("Social Science", &["EUSS"], 2, "be:distribution"),
            be_attr_constraint("Humanities", &["EUHS"], 2, "be:distribution"),
            be_attr_constraint("SSH Elective", &["EUSS", "EUHS"], 1, "be:distribution"),
            be_attr_constraint(
                "Technology & Business",
                &["EUHS", "EUSS", "EUTB"],
                2,
                "be:distribution",
            ),
        ],
    }
}

/// BE schedule template — one semester per top-level requirement, in list order.
/// Based on the official Bioengineering BSE sample curriculum.
const BE_SCHEDULE: [Semester; 33] = [
    // Engineering
    Y1F, Y1S, Y2F, Y2S, Y2S, Y3F, Y3S, Y3F, Y3S, Y3S, Y4F, Y4S, Y4F, Y4S, Y4F, Y4S,
    // Math and Natural Science
    Y1F, Y1S, Y2F, Y2S, Y1F, Y1S, Y1F, Y1F, Y1S, Y1S, Y2F, Y2F, Y3F,
    // General Electives (CoursePool)
    Y2F,
    // Free Electives
    Y3F, Y3S, Y4F,
];

pub fn create_be_major() -> Major {
    Major {
        short_name: "BE".to_string(),
        name: "Bioengineering".to_string(),
        requirements: vec![
            // Engineering
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 1000".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec![
                    "ENGR 1050".to_string()
                ],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 2000".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 2200".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 2700".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 3010".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 3060".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 3090".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 3100".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 3500".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 4950".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Engineering".to_string()),
                possibilities: vec!["BE 4960".to_string()],
            },
            Requirement::Restriction {
                category: Some("BE Elective (4000 or 5000 level)".to_string()),
                department: Some(vec!["BE".to_string()]),
                cu: None,
                level: Some(4000),
                max_level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("BE Elective (4000 or 5000 level)".to_string()),
                department: Some(vec!["BE".to_string()]),
                cu: None,
                level: Some(4000),
                max_level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("Engineering Elective".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUNG".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("Engineering Elective".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: Some(vec!["EUNG".to_string()]),
                number: 1,
                excluding: None,
                no_school: None,
            },

            // Math and Natural Science
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["MATH 1400".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["MATH 1410".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["ESE 2030".to_string(), "ENM 2400".to_string(), "MATH 2400".to_string(), "ENM 2030".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["ENM 3750".to_string(), "ENGR 3440".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["PHYS 0140".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["PHYS 0141".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["CHEM 1012".to_string(), "CHEM 1151".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["CHEM 1101".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["CHEM 1102".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["CHEM 1022".to_string(), "CHEM 1161".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["BIOL 1121".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["BIOL 1123".to_string()],
            },
            Requirement::SingleCourse {
                category: Some("Math and Natural Science".to_string()),
                possibilities: vec!["BIOL 3310".to_string()],
            },

            be_general_electives_pool(),

            // Free Electives
            Requirement::Restriction {
                category: Some("Free Elective".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("Free Elective".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
            Requirement::Restriction {
                category: Some("Free Elective".to_string()),
                department: None,
                cu: None,
                level: None,
                max_level: None,
                attr: None,
                number: 1,
                excluding: None,
                no_school: None,
            },
        ],
        schedule_hints: seas_schedule_hints(&BE_SCHEDULE),
        concentrations: Some(be_concentrations()),
    }
}