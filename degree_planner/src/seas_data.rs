use std::collections::BTreeMap;
use crate::Major;
use crate::Requirement;

pub fn create_ee_major() -> Major {
        return Major {
        short_name: "EE".to_string(),
        name: "Electrical Engineering".to_string(),
        requirements: vec![
            // Engineering
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1100".to_string()] },
            Requirement::AnyOf { 
                category: Some("Engineering".to_string()), 
                possibilities: vec![
<<<<<<< HEAD
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 1110".to_string()] },
=======
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 1110".to_string(), "MEAM 1010".to_string()] },
>>>>>>> 0dc7dc2 (cu stuff changes)
                    Requirement::Restriction { category: Some("Sub for ESE 1110".to_string()), department:Some(vec!["ESE".to_string()]), number: 1, cu: None, level: None, attr: None, excluding: Some(["ESE 1120", "ESE 2030", "ESE 3010", "ESE 4020"].map(String::from).to_vec()), no_school: None }
                ] 
            },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["CIS 1200".to_string(), "CIS 2400".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 2150".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 2180".to_string()] },
            Requirement::SingleCourse { category: Some("Engineering".to_string()), possibilities: vec!["ESE 2240".to_string()] },
            
            // Intermediate or Advanced ESE Elective
            Requirement::Restriction { 
                category: Some("Intermediate or Advanced ESE Elective".to_string()), 
                department:Some(vec!["ESE".to_string()]), number:1, cu: None, level: Some(2000), attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), 
                no_school: None 
            },
            
            // Advanced ESE courses
            Requirement::Restriction { 
                category: Some("Advanced ESE courses".to_string()), 
                department:Some(vec!["ESE".to_string()]), number:1, cu: None, level: Some(3000), attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            },
            Requirement::Restriction { 
                category: Some("Advanced ESE courses".to_string()), 
                department:Some(vec!["ESE".to_string()]), number:1, cu: None, level: Some(3000), attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            },
            Requirement::Restriction { 
                category: Some("Advanced ESE courses".to_string()), 
                department:Some(vec!["ESE".to_string()]), number:1, cu: None, level: Some(3000), attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            },

            Requirement::AnyOf { 
                category: Some("Advanced ESE courses (other dept. options possible)".to_string()),
                possibilities: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["CIS 5200".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["BE 5210".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["CIS 4710".to_string()] },
                    Requirement::Restriction { 
                        category: None, 
                        department: Some(vec!["ESE".to_string()]), number: 1, cu: None, level: Some(3000), attr: None, excluding: None, 
                        no_school: None 
                    },
                ] 
            },

            // Design and Project courses
            Requirement::AnyOf { 
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
                        Requirement::Restriction { category: Some("Extra half-credit course with BE 4700".to_string()), department: None, cu: Some(5), level: None, attr: None, number: 1, excluding: None, no_school: None  }
                    ] },
                ] 
            },
            Requirement::SingleCourse { category: Some("Design and Project courses".to_string()), possibilities: vec!["ESE 4500".to_string()] },
            Requirement::SingleCourse { category: Some("Design and Project courses".to_string()), possibilities: vec!["ESE 4510".to_string()] },

            // Math and Natural Science
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 2400".to_string(), "ESE 2030".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 3010".to_string()] },
            Requirement::AnyOf {
                category: Some("Math and Natural Science - Mechanics".to_string()), 
                possibilities: vec![
                    Requirement::AllOf {
                        category: None, 
                        requirements: vec![
                            Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1100".to_string()] },
                            Requirement::SingleCourse { category: None, possibilities: vec!["BIOL 1124".to_string(), "PHYS 0050".to_string(), "MEAM 1470".to_string(), "CHEM 1101".to_string()] },
                        ]
                    },
                    Requirement::AllOf {
                        category: None, 
                        requirements: vec![
                            Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0140".to_string()] },
                            Requirement::SingleCourse { category: None, possibilities: vec!["BIOL 1124".to_string(), "PHYS 0050".to_string(), "MEAM 1470".to_string(), "CHEM 1101".to_string()] },
                        ]
                    },
                    Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0150".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0170".to_string()] },
                ]
            },
            Requirement::SingleCourse { category: Some("Math and Natural Science - E&M".to_string()), possibilities: vec!["ESE 1120".to_string()] },
            Requirement::SingleCourse { 
                category: Some("Math and Natural Science - Chem/Bio".to_string()), 
                possibilities: vec![
                    "CHEM 1012".to_string(),
                    "EAS 0091".to_string(),
                    "BIOL 1121".to_string(),
                    "BIOL 1101".to_string()
                ]
            },
            Requirement::Restriction { category: Some("Math and Natural Science - Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUMA".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Math and Natural Science - Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None },

            // Professional Electives
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::AnyOf {
                category: Some("Professional Electives".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 4000".to_string(), "EAS 5450".to_string(), "ESE 5950".to_string(), "MGMT 2370".to_string(), "OIDD 2360".to_string()] },
                ]
            },

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives - Ethics".to_string()), possibilities: vec!["LAWM 5060".to_string(), "EAS 2030".to_string()] },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS/TBS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS/TBS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
        ],
        concentrations: Some(BTreeMap::from([
            (
                "Data Science".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["ESE 3060".to_string()] },
                    Requirement::SingleCourse { category: Some("Data Science".to_string()), possibilities: vec!["ESE 4020".to_string()] },
                    Requirement::CourseGroup { category: Some("Data Science".to_string()), number: 2, possibilities: vec!["NETS 3120".to_string(), "ESE 5140".to_string(), "CIS 5200".to_string(), "CIS 5450".to_string(), "ESE 5460".to_string(), "ESE 6500".to_string()] }
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
                            Requirement::CourseGroup { category: None, number: 2, possibilities: vec!["ESE 4190".to_string(), "ESE 5780".to_string(), "ESE 5800".to_string(), "ESE 6680".to_string(), "ESE 6710".to_string(), "ESE 6720".to_string()] },
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
                                    Requirement::CourseGroup { category: None, number: 1, possibilities: vec!["CIS 4710".to_string(), "ESE 5320".to_string(), "ESE 5390".to_string()] },
                                ]
                            },
                            Requirement::CourseGroup { category: None,number: 3, possibilities: vec!["CIS 4710".to_string(), "ESE 5320".to_string(), "ESE 5390".to_string()] },
                        ]
                    }
                ]
            ),
            (
                "Photonics and Quantum Technology".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Photonics and Quantum Technology".to_string()), possibilities: vec!["ESE 3200".to_string()] },
                    Requirement::SingleCourse { category: Some("Photonics and Quantum Technology".to_string()), possibilities: vec!["ESE 3300".to_string()] },
                    Requirement::CourseGroup { category: Some("Photonics and Quantum Technology".to_string()), number: 2, possibilities: vec!["ESE 5090".to_string(), "ESE 5100".to_string(), "ESE 5130".to_string(), "ESE 5230".to_string(), "ESE 5360".to_string(), "ESE 6730".to_string()] },
                ]
            ),
            (
                "Microsystems and Nanotechnology".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Microsystems and Nanotechnology".to_string()), possibilities: vec!["ESE 5250".to_string()] },
                    Requirement::CourseGroup { category: Some("Microsystems and Nanotechnology".to_string()), number: 3, possibilities: vec!["ESE 3300".to_string(), "ESE 5100".to_string(), "ESE 5210".to_string(), "ESE 5290".to_string(), "ESE 5360".to_string(), "ESE 6210".to_string(), "ESE 6250".to_string()] },
                ]
            ),
            (
                "Robotics".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Robotics".to_string()), possibilities: vec!["ESE 4210".to_string()] },
                    Requirement::CourseGroup { category: Some("Robotics".to_string()), number: 3, possibilities: vec!["ESE 5000".to_string(), "ESE 5050".to_string(), "MEAM 5200".to_string(), "ESE 6150".to_string(), "ESE 6190".to_string(), "ESE 6250".to_string(), "ESE 6500".to_string(), "MEAM 6200".to_string()] },
                ]
            ),
        ])),
    };
}

pub fn create_meam_major(concentration_name: String) -> Major {

    let meam_concentrations = BTreeMap::from([
            (
                "Dynamics, Controls, and Robotics".to_string(), 
                vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3200".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3210".to_string()] },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(3000), attr: None, excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(5000), attr: None, excluding: None, number: 1, no_school: None },

                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None  }, // tech elective from before
                ]
            ),
            (
                "Energy, Fluids and Thermal Systems".to_string(), 
                vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3020".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3330".to_string()] },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(3000), attr: None, excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(5000), attr: None, excluding: None, number: 1, no_school: None },

                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None  }, // tech elective from before
                ]
            ),
            (
                "Mechanics of Materials, Structures and Design".to_string(), 
                vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3210".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3540".to_string()] },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(3000), attr: None, excluding: None, number: 1, no_school: None },
                    Requirement::Restriction { category: None, department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(5000), attr: None, excluding: None, number: 1, no_school: None },

                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None }, // tech elective from before
                ]
            ),
            (
                "General".to_string(), 
                vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3020".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3210".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3330".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 3540".to_string()] },

                    Requirement::Restriction { category: None, department: None, cu: None, level: Some(5000), attr: Some(vec!["EUNG".to_string()]), number: 1, no_school: None, excluding: None }, // tech elective from before (one must be upper level if general conc.)
                ]
            ),
        ]);

    return Major { 
        short_name: "MEAM".to_string(), 
        name: "Mechanical Engineering".to_string(), 
        requirements: vec![
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
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 2400".to_string()] },
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
            Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUMA".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Math and Natural Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None },

            // Professional Electives
            Requirement::SingleCourse { category: Some("Professional Electives".to_string()), possibilities: vec!["ENGR 1050".to_string(), "CIS 1100".to_string(), "CIS 1200".to_string()] },
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: Some(vec!["MEAM".to_string()]), cu: None, level: Some(5000), attr: None, number: 2, excluding: Some(vec!["MEAM 5990".to_string()]), no_school: None },
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 2, excluding: None, no_school: None }, // one tech elective in concentration section
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: Some(2000), attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None }, // at max 3 prof. electives including first one at 1000 level

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["EAS 2030".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUSS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string()]), excluding: None, number: 2, no_school: None },
            Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["EUSS".to_string(), "EUHS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["EUSS".to_string(), "EUHS".to_string(), "EUTB".to_string()]), excluding: None, number: 2, no_school: None },

            // Concentration
            Requirement::Concentration { category: Some("Concentration".to_string()), number: 4, requirements: meam_concentrations.get(&concentration_name).unwrap().clone() }
        ], 
        concentrations: Some(meam_concentrations),
    };
}

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
            Requirement::Restriction { category: Some("Math and Natural Science - Math Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUMA".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::SingleCourse { category: Some("Math and Natural Science - Mechanics".to_string()), possibilities: vec!["PHYS 0140".to_string(), "MEAM 1100".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science - E&M".to_string()), possibilities: vec!["PHYS 0141".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CHEM 1012".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CHEM 1101".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CHEM 1022".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MSE 2210".to_string()] },

            // Technical Electives
            Requirement::SingleCourse { category: Some("Technical Electives".to_string()), possibilities: vec!["ENGR 1050".to_string()] },
            Requirement::Restriction { category: Some("Technical Electives - MSE Elective".to_string()), department: Some(vec!["MSE".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives - MSE Elective".to_string()), department: Some(vec!["MSE".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["EAS 2030".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },

            Requirement::Restriction { category: Some("Free Elective".to_string()), department: None, cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None }
        ],
        concentrations: Some(BTreeMap::from([
            (
                "Biomaterials and Biomimetics".to_string(), 
                vec![
                    Requirement::SingleCourse { category: Some("Biomaterials and Biomimetics".to_string()), possibilities: vec!["MSE 4300".to_string()] },
                    Requirement::SingleCourse { category: Some("Biomaterials and Biomimetics".to_string()), possibilities: vec!["MSE 5850".to_string()] },
                    Requirement::SingleCourse { category: Some("Biomaterials and Biomimetics".to_string()), possibilities: vec!["MSE 5180".to_string(), "MSE 4650".to_string(), "MSE 0099".to_string(), "CBE 5110".to_string(), "CBE 5640".to_string(), "BE 2200".to_string(), "BE 5120".to_string(), "PHYS 2280".to_string()] },
                    Requirement::SingleCourse { category: Some("Biomaterials and Biomimetics".to_string()), possibilities: vec!["MSE 5180".to_string(), "MSE 4650".to_string(), "MSE 0099".to_string(), "CBE 5110".to_string(), "CBE 5640".to_string(), "BE 2200".to_string(), "BE 5120".to_string(), "PHYS 2280".to_string()] },
                ]
            ),
            
        ])),
    };
}

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
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["NETS".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
                ]
            },
            Requirement::AnyOf {
                category: Some("Engineering".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["NETS".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
                ]
            },
            Requirement::AnyOf {
                category: Some("Engineering".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["NETS".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
                ]
            },
            Requirement::AnyOf {
                category: Some("Engineering".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["CIS".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
                    Requirement::Restriction { category: Some("CIS Elective".to_string()), department: Some(vec!["NETS".to_string()]), cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
                ]
            },
            
            // Math and Natural Science
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 2400".to_string(), "ESE 2030".to_string(),"MATH 2600".to_string(), "MATH 3120".to_string(), "MATH 3130".to_string(), "MATH 3140".to_string()] },
            Requirement::Restriction { category: Some("Math and Natural Science - Math/Science Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None },
            
            Requirement::SingleCourse { category: Some("Math and Natural Science - E&M".to_string()), possibilities: vec!["PHYS 0151".to_string(), "PHYS 0171".to_string(), "ESE 1120".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CIS 1600".to_string()] },
            
            Requirement::AnyOf { category: Some("Math and Natural Science - Mechanics".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["PHYS 0150".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["MEAM 1470".to_string()] },
                ] }
            ] },

            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CIS 2610".to_string(), "ESE 3010".to_string(), "STAT 4300".to_string()] },

            // Technical Electives
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["EAS 2030".to_string(), "CIS 4230".to_string(), "CIS 5230".to_string(), "LAWM 5060".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            
            // Free Elective
            Requirement::Restriction { category: Some("Free Elective".to_string()), department: None, cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None }
        ],
        concentrations: Some(BTreeMap::from([
            
            
        ])),
    };
}

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
            Requirement::Restriction { category: Some("Math and Natural Science - NS Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNS".to_string()]), number: 1, excluding: None, no_school: None },
            
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
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Technical Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            
            // General Electives
            Requirement::SingleCourse { category: Some("General Electives".to_string()), possibilities: vec!["LAWM 5060".to_string(), "CIS 4230".to_string(), "CIS 5230".to_string()] },
            Requirement::SingleCourse { category: Some("Cognitive Science Elective".to_string()), possibilities: vec!["COGS 1001".to_string(), "LING 0500".to_string(), "LING 2500".to_string(), "LING 3810".to_string(), "PHIL 1710".to_string(), "PHIL 2640".to_string(), "PHIL 4721".to_string(), "PHIL 4840".to_string(), "PSYC 1210".to_string(), "PSYC 1340".to_string(), "PSYC 1230".to_string(), "PSYC 1310".to_string(), "PSYC 2737".to_string()] },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            
            // Free Elective
            Requirement::Restriction { category: Some("Free Elective".to_string()), department: None, cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None }
        ],
        concentrations: Some(BTreeMap::from([
            
            
        ])),
    };
}

pub fn create_compe_major() -> Major {
        return Major {
        short_name: "CE".to_string(),
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
                department: Some(vec!["ESE".to_string(), "CIS".to_string(), ]), number: 1, cu: None, level: Some(2000), attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), 
                no_school: None 
            },
            
            // Advanced CIS or ESE courses
            Requirement::Restriction { 
                category: Some("Advanced CIS or ESE Electives".to_string()), 
                department: Some(vec!["ESE".to_string(), "CIS".to_string()]), number: 1, cu: None, level: Some(3000), attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            },
            Requirement::Restriction { 
                category: Some("Advanced CIS or ESE Electives".to_string()), 
                department: Some(vec!["ESE".to_string(), "CIS".to_string()]), number: 1, cu: None, level: Some(3000), attr: None, 
                excluding: Some(vec!["ESE 3010".to_string(), "ESE 4020".to_string(), "ESE 2030".to_string()]), no_school: None 
            },

            // Design and Project courses
            Requirement::SingleCourse { category: Some("Design and Project courses".to_string()), possibilities: vec!["CIS 4000".to_string(), "ESE 4500".to_string()] },
            Requirement::SingleCourse { category: Some("Design and Project courses".to_string()), possibilities: vec!["CIS 4010".to_string(), "ESE 4510".to_string()] },

            // Math and Natural Science
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1400".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["MATH 1410".to_string(), "MATH 1610".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 2030".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["ESE 3010".to_string(), "CIS 2610".to_string(), "STAT 4300".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science".to_string()), possibilities: vec!["CIS 1600".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science - Mechanics".to_string()), possibilities: vec!["MEAM 1100".to_string(), "PHYS 0140".to_string(), "PHYS 0150".to_string(), "PHYS 0170".to_string()] },
            Requirement::SingleCourse { category: Some("Math and Natural Science - E&M".to_string()), possibilities: vec!["ESE 1120".to_string()] },
            Requirement::SingleCourse { 
                category: Some("Math and Natural Science - Chem/Bio".to_string()), 
                possibilities: vec![
                    "CHEM 1012".to_string(),
                    "EAS 0091".to_string(),
                    "BIOL 1121".to_string(),
                    "BIOL 1101".to_string(),
                    "PHYS 1240".to_string()
                ]
            },
            Requirement::Restriction { category: Some("Math and Natural Science - Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUMA".to_string(), "EUNS".to_string()]), number: 1, excluding: None, no_school: None },

            // Professional Electives
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("Professional Electives".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::AnyOf {
                category: Some("Professional Electives".to_string()), 
                possibilities: vec![
                    Requirement::Restriction { category: None, department: None, cu: None, level: None, attr: Some(vec!["EUNG".to_string()]), number: 1, excluding: None, no_school: None },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ESE 4000".to_string(), "EAS 5450".to_string(), "ESE 5950".to_string(), "MGMT 2370".to_string(), "OIDD 2360".to_string()] },
                ]
            },

            // General Electives
            Requirement::SingleCourse { category: Some("General Electives - Ethics".to_string()), possibilities: vec!["LAWM 5060".to_string(), "EAS 2030".to_string(), "CIS 4230".to_string(), "CIS 5230".to_string()] },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS/TBS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },
            Requirement::Restriction { category: Some("General Electives - Humanities/SS/TBS".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EUHS".to_string(), "EUSS".to_string(), "EUTB".to_string()]), number: 1, excluding: None, no_school: None },

            // Free Elective
            Requirement::Restriction { category: Some("Free Elective".to_string()), department: None, cu: None, level: None, attr: None, number: 1, excluding: None, no_school: None },
        ],
        concentrations: Some(BTreeMap::from([

            
        ])),
    };
}