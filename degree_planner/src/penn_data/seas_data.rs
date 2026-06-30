use std::collections::{BTreeMap, HashMap};
use crate::Major;
use crate::Requirement;
use crate::penn_data::requirement_builders::{
    all_of, any_of, any_of_opt, attr_pool_constraint, attrs_pool_constraint, code,
    course_group, course_pool, repeat_req, restriction, single, single_pool_constraint,
};
use crate::schedule_template::{
    schedule_hints_from_array, scheduled, insert_fixed_course_hints, ScheduleHint, Semester,
    Y1F, Y1S, Y2F, Y2S, Y3F, Y3S, Y4F, Y4S,
};

// --- Generic builders --- (imported)

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

// --- Majors ---

pub fn create_ee_major() -> Major {
    let (requirements, mut schedule_hints) = scheduled(vec![
            // Engineering
            (Y1F, single("Engineering", &["CIS 1100"])),
            (Y1F, any_of("Engineering", vec![
                    code(&["ESE 1110", "MEAM 1010"]),
                    restriction(1).category("Sub for ESE 1110").departments(&["ESE"]).excluding(&["ESE 1120", "ESE 2030", "ESE 3010", "ESE 4020"]).into(),
                ])),
            (Y1S, single("Engineering", &["CIS 1200", "CIS 2400"])),
            (Y2F, single("Engineering", &["ESE 2150"])),
            (Y2F, single("Engineering", &["ESE 2180"])),
            (Y2S, single("Engineering", &["ESE 2240"])),
            
            // Intermediate or Advanced ESE Elective
            (Y2S, restriction(1).category("Intermediate or Advanced ESE Elective").departments(&["ESE"]).level(2000).excluding(&["ESE 3010", "ESE 4020", "ESE 2030"]).into()),
            
            // Advanced ESE courses
            (Y3F, restriction(1).category("Advanced ESE courses").departments(&["ESE"]).level(3000).excluding(&["ESE 3010", "ESE 4020", "ESE 2030"]).into()),
            (Y3F, restriction(1).category("Advanced ESE courses").departments(&["ESE"]).level(3000).excluding(&["ESE 3010", "ESE 4020", "ESE 2030"]).into()),
            (Y3S, restriction(1).category("Advanced ESE courses").departments(&["ESE"]).level(3000).excluding(&["ESE 3010", "ESE 4020", "ESE 2030"]).into()),

            (Y4F, any_of("Advanced ESE courses", vec![
                    code(&["CIS 5200"]),
                    code(&["BE 5210"]),
                    code(&["CIS 4710"]),
                    restriction(1).departments(&["ESE"]).level(3000).into(),
                ])),

            // Design and Project courses
            (Y3S, any_of("Design and Project courses", vec![
                    code(&["ESE 4210"]),
                    all_of(Some("Design and Project courses (Research)".to_string()), vec![
                        code(&["ESE 2900"]),
                        code(&["ESE 2910"]),
                    ]),
                    code(&["ESE 3190"]),
                    code(&["ESE 3360"]),
                    code(&["ESE 3500"]),
                    all_of(None, vec![
                        code(&["BE 4700"]),
                        restriction(1).category("Extra half-credit course with BE 4700").cu(5).into(),
                    ]),
                ])),
            (Y4F, single("Design and Project courses", &["ESE 4500"])),
            (Y4S, single("Design and Project courses", &["ESE 4510"])),

            // Math and Natural Science
            (Y1F, single("Math and Natural Science", &["MATH 1400"])),
            (Y1S, single("Math and Natural Science", &["MATH 1410"])),
            (Y2F, single("Math and Natural Science", &["MATH 2400", "ESE 2030"])),
            (Y2S, single("Math and Natural Science", &["ESE 3010"])),
            (Y1F, any_of("Math and Natural Science", vec![
                    code(&["PHYS 0150"]),
                    all_of(None, vec![
                        code(&["MEAM 1100"]),
                        code(&["MEAM 1470", "BIOL 1124", "PHYS 0050", "CHEM 1101"]),
                    ]),
                    all_of(None, vec![
                        code(&["PHYS 0140"]),
                        code(&["BIOL 1124", "PHYS 0050", "MEAM 1470", "CHEM 1101"]),
                    ]),
                    code(&["PHYS 0170"]),
                ])),
            (Y1S, single("Math and Natural Science", &["ESE 1120"])),
            (Y1S, single("Math and Natural Science", &["CHEM 1012", "EAS 0091", "BIOL 1121", "BIOL 1101"])),
            (Y3F, restriction(1).category("Math and Natural Science").attr(&["EUMA"]).into()),
            (Y3S, restriction(1).category("Math and Natural Science").attr(&["EUMA", "EUNS"]).into()),

            // Professional Electives
            (Y3F, restriction(1).category("Professional Electives").attr(&["EUNG", "EUMA", "EUNS"]).into()),
            (Y3S, restriction(1).category("Professional Electives").attr(&["EUNG", "EUMA", "EUNS"]).into()),
            (Y4S, restriction(1).category("Professional Electives").attr(&["EUNG", "EUMA", "EUNS"]).into()),
            (Y3S, any_of("Professional Electives", vec![
                    code(&["ESE 4000", "EAS 5450", "ESE 5950", "MGMT 2370", "OIDD 2360"]),
                    restriction(1).attr(&["EUNG", "EUMA", "EUNS"]).into(),
                ])),

            // General Electives
            (Y2F, single("General Electives", &["EAS 2030", "LAWM 5060"])),
            (Y2S, restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into()),
            (Y3F, restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into()),
            (Y4F, restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into()),
            (Y4S, restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into()),
            (Y3S, restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into()),
            (Y3S, restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into()),
            (Y4S, restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into()),
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
                    single("Data Science", &["ESE 3060"]),
                    single("Data Science", &["ESE 4020"]),
                    single("Data Science", &["NETS 3120", "ESE 5140", "CIS 5200", "CIS 5450", "ESE 5460", "ESE 6500"]),
                    single("Data Science", &["NETS 3120", "ESE 5140", "CIS 5200", "CIS 5450", "ESE 5460", "ESE 6500"])
                ]
            ),
            (
                "Mixed-Signal and RF Integrated Circuits".to_string(), 
                vec![
                    single("Mixed-Signal and RF Integrated Circuits", &["ESE 3190"]),
                    single("Mixed-Signal and RF Integrated Circuits", &["ESE 3700"]),
                    any_of("Mixed-Signal and RF Integrated Circuits", vec![
                    all_of(None, vec![
                        code(&["ESE 5730"]),
                        code(&["ESE 5750"]),
                    ]),
                    all_of(None, vec![
                        code(&["ESE 4190", "ESE 5780", "ESE 5800", "ESE 6680", "ESE 6710", "ESE 6720"]),
                        code(&["ESE 4190", "ESE 5780", "ESE 5800", "ESE 6680", "ESE 6710", "ESE 6720"]),
                    ]),
                ])
                ]
            ),
            (
                "System-on-A-Chip Design".to_string(), 
                vec![
                    single("System-on-A-Chip Design", &["ESE 3700"]),
                    any_of("System-on-A-Chip Design", vec![
                    all_of(None, vec![
                        code(&["ESE 5730"]),
                        code(&["ESE 5750"]),
                        code(&["CIS 4710", "ESE 5320", "ESE 5390"]),
                    ]),
                    all_of(None, vec![
                        code(&["CIS 4710"]),
                        code(&["ESE 5320"]),
                        code(&["ESE 5390"]),
                    ]),
                ])
                ]
            ),
            (
                "Photonics and Quantum Technology".to_string(), 
                vec![
                    single("Photonics and Quantum Technology", &["ESE 3200"]),
                    single("Photonics and Quantum Technology", &["ESE 3300"]),
                    single("Photonics and Quantum Technology", &["ESE 5090", "ESE 5100", "ESE 5130", "ESE 5230", "ESE 5360", "ESE 6730"]),
                    single("Photonics and Quantum Technology", &["ESE 5090", "ESE 5100", "ESE 5130", "ESE 5230", "ESE 5360", "ESE 6730"]),
                ]
            ),
            (
                "Microsystems and Nanotechnology".to_string(), 
                vec![
                    single("Microsystems and Nanotechnology", &["ESE 5250"]),
                    single("Microsystems and Nanotechnology", &["ESE 3300", "ESE 5100", "ESE 5210", "ESE 5290", "ESE 5360", "ESE 6210", "ESE 6250"]),
                    single("Microsystems and Nanotechnology", &["ESE 3300", "ESE 5100", "ESE 5210", "ESE 5290", "ESE 5360", "ESE 6210", "ESE 6250"]),
                    single("Microsystems and Nanotechnology", &["ESE 3300", "ESE 5100", "ESE 5210", "ESE 5290", "ESE 5360", "ESE 6210", "ESE 6250"]),
                ]
            ),
            (
                "Robotics".to_string(), 
                vec![
                    single("Robotics", &["ESE 4210"]),
                    single("Robotics", &["ESE 5000", "ESE 5050", "MEAM 5200", "ESE 6150", "ESE 6190", "ESE 6250", "ESE 6500", "MEAM 6200"]),
                    single("Robotics", &["ESE 5000", "ESE 5050", "MEAM 5200", "ESE 6150", "ESE 6190", "ESE 6250", "ESE 6500", "MEAM 6200"]),
                    single("Robotics", &["ESE 5000", "ESE 5050", "MEAM 5200", "ESE 6150", "ESE 6190", "ESE 6250", "ESE 6500", "MEAM 6200"]),
                ]
            ),
        ])),
    };
}

// --- Shared constants ---

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

// --- Domain helpers ---

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
                    code(&["MEAM 3200"]),
                    code(&["MEAM 3210"]),
                    restriction(1).departments(&["MEAM"]).level(3000).into(),
                    restriction(1).departments(&["MEAM"]).level(5000).into(),

                    restriction(1).attr(&["EUNG"]).into(), // tech elective from before
                ]
            ),
            (
                "Energy, Fluids and Thermal Systems".to_string(), 
                vec![
                    code(&["MEAM 3020"]),
                    code(&["MEAM 3330"]),
                    restriction(1).departments(&["MEAM"]).level(3000).into(),
                    restriction(1).departments(&["MEAM"]).level(5000).into(),

                    restriction(1).attr(&["EUNG"]).into(), // tech elective from before
                ]
            ),
            (
                "Mechanics of Materials, Structures and Design".to_string(), 
                vec![
                    code(&["MEAM 3210"]),
                    code(&["MEAM 3540"]),
                    restriction(1).departments(&["MEAM"]).level(3000).into(),
                    restriction(1).departments(&["MEAM"]).level(5000).into(),

                    restriction(1).attr(&["EUNG"]).into(), // tech elective from before
                ]
            ),
            (
                "General".to_string(), 
                vec![
                    code(&["MEAM 3020"]),
                    code(&["MEAM 3210"]),
                    code(&["MEAM 3330"]),
                    code(&["MEAM 3540"]),

                    restriction(1).level(5000).attr(&["EUNG"]).into(), // tech elective from before (one must be upper level if general conc.)
                ]
            ),
        ]);

    let conc_slots = meam_concentration_slots(&concentration_name, &meam_concentrations);

    let mut requirements = vec![
            // MEAM Core
            single("MEAM Core", &["MEAM 2020"]),
            single("MEAM Core", &["MEAM 2030"]),
            single("MEAM Core", &["MEAM 2100"]),
            single("MEAM Core", &["MEAM 2110"]),
            single("MEAM Core", &["MEAM 2470"]),
            single("MEAM Core", &["MEAM 2480"]),
            single("MEAM Core", &["MEAM 3470"]),
            single("MEAM Core", &["MEAM 3480"]),
            single("MEAM Core", &["MEAM 4450"]),
            single("MEAM Core", &["MEAM 4460"]),
            
            // Math and Natural Science
            single("Math and Natural Science", &["MATH 1400"]),
            single("Math and Natural Science", &["MATH 1410", "MEAM 1610"]),
            single("Math and Natural Science", &["MATH 2400", "ESE 2030", "MEAM 2600", "MEAM 2200"]),
            single("Math and Natural Science", &["ENM 2510", "MATH 2410", "MEAM 2300"]),
            any_of("Math and Natural Science", vec![
                    code(&["PHYS 0150"]),
                    all_of(None, vec![
                        code(&["MEAM 1100"]),
                        code(&["MEAM 1470"]),
                    ]),
                ]),
            single("Math and Natural Science", &["PHYS 0151", "ESE 1120"]),
            single("Math and Natural Science", &["CHEM 1012", "BIOL 1121"]),
            restriction(1).category("Math and Natural Science").attr(&["EUMA"]).into(),
            restriction(1).category("Math and Natural Science").attr(&["EUMA", "EUNS"]).into(),

            // Professional Electives
            single("Professional Electives", &["ENGR 1050", "CIS 1100", "CIS 1200"]),
            restriction(2).category("Professional Electives").departments(&["MEAM"]).level(5000).excluding(&["MEAM 5990"]).into(),
            restriction(2).category("Professional Electives").attr(&["EUNG"]).into(), // one tech elective in concentration section
            restriction(1).category("Professional Electives").level(2000).attr(&["EUNG"]).into(), // at max 3 prof. electives including first one at 1000 level

            // General Electives
            single("General Electives", &["EAS 2030"]),
            restriction(1).category("General Electives").attr(&["EUSS"]).into(),
            restriction(2).category("General Electives").attr(&["EUHS"]).into(),
            restriction(1).category("General Electives").attr(&["EUSS", "EUHS"]).into(),
            restriction(2).category("General Electives").attr(&["EUSS", "EUHS", "EUTB"]).into(),
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
            single("Engineering", &["MSE 1010"]),
            single("Engineering", &["MSE 2010"]),
            single("Engineering", &["MSE 2020"]),
            single("Engineering", &["MSE 2150"]),
            single("Engineering", &["MSE 2200"]),
            single("Engineering", &["MSE 2600"]),
            single("Engineering", &["MSE 3010"]),
            single("Engineering", &["MSE 3300"]),
            single("Engineering", &["MSE 3600"]),
            single("Engineering", &["MSE 3930"]),
            single("Engineering", &["MSE 4050"]),
            single("Engineering", &["MSE 4400"]),
            single("Engineering", &["MSE 4600"]),
            single("Engineering", &["MSE 4950"]),
            single("Engineering", &["MSE 4960"]),

            // Math and Natural Science
            single("Math and Natural Science", &["MATH 1400"]),
            single("Math and Natural Science", &["MATH 1410", "MATH 1610"]),
            single("Math and Natural Science", &["MATH 2400", "ESE 2030"]),
            single("Math and Natural Science", &["MATH 2410", "ENM 2510"]),
            restriction(1).category("Math and Natural Science").attr(&["EUMA"]).into(),
            single("Math and Natural Science", &["PHYS 0140", "MEAM 1100"]),
            single("Math and Natural Science", &["PHYS 0141"]),
            single("Math and Natural Science", &["CHEM 1012"]),
            single("Math and Natural Science", &["CHEM 1101"]),
            single("Math and Natural Science", &["CHEM 1022"]),
            single("Math and Natural Science", &["MSE 2210"]),

            // Technical Electives
            single("Technical Electives", &["ENGR 1050"]),
            restriction(1).category("Technical Electives - MSE Elective").departments(&["MSE"]).into(),
            restriction(1).category("Technical Electives - MSE Elective").departments(&["MSE"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUNG"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUNG"]).into(),

            // General Electives
            single("General Electives", &["EAS 2030"]),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),

            restriction(1).category("Free Elective").into()
        ],
        schedule_hints: seas_schedule_hints(&MSE_SCHEDULE),
        concentrations: Some(BTreeMap::from([
            (
                "Biomaterials and Biomimetics".to_string(),
                vec![
                    single("Biomaterials and Biomimetics", &["MSE 4300"]),
                    single("Biomaterials and Biomimetics", &["MSE 5850", "BE 5850"]),
                    single("Biomaterials and Biomimetics", &["MSE 5180", "MSE 4650", "MSE 5650", "MSE 0099", "CBE 5110", "CBE 5640", "BE 2200", "BE 5120", "PHYS 2280"]),
                    single("Biomaterials and Biomimetics", &["MSE 5180", "MSE 4650", "MSE 5650", "MSE 0099", "CBE 5110", "CBE 5640", "BE 2200", "BE 5120", "PHYS 2280"]),
                ],
            ),
            (
                "Electronic & Optical Devices and Sensors".to_string(),
                vec![
                    single("Electronic & Optical Devices and Sensors", &["MSE 5360"]),
                    single("Electronic & Optical Devices and Sensors", &["MSE 4650", "MSE 5650"]),
                    single("Electronic & Optical Devices and Sensors", &["MSE 5250", "MSE 6400", "MSE 0099", "ESE 2150", "ESE 2180", "ESE 5100", "ESE 5210", "ESE 5230"]),
                    single("Electronic & Optical Devices and Sensors", &["MSE 5250", "MSE 6400", "MSE 0099", "ESE 2150", "ESE 2180", "ESE 5100", "ESE 5210", "ESE 5230"]),
                ],
            ),
            (
                "Energy and Sustainability".to_string(),
                vec![
                    single("Energy and Sustainability", &["MSE 4550"]),
                    single("Energy and Sustainability", &["MSE 5450"]),
                    single("Energy and Sustainability", &["MSE 5020", "MSE 0099", "CBE 5450", "EAS 3010", "EAS 4010", "EAS 4020", "EAS 4030", "MEAM 5020", "MEAM 5030", "MEAM 5800"]),
                    single("Energy and Sustainability", &["MSE 5020", "MSE 0099", "CBE 5450", "EAS 3010", "EAS 4010", "EAS 4020", "EAS 4030", "MEAM 5020", "MEAM 5030", "MEAM 5800"]),
                ],
            ),
            (
                "Nanotechnology".to_string(),
                vec![
                    single("Nanotechnology", &["MSE 5250"]),
                    single("Nanotechnology", &["MSE 4650", "MSE 5650"]),
                    single("Nanotechnology", &["MSE 0099", "MSE 6100", "MSE 5360", "ESE 3360", "ESE 4230", "ESE 5360", "ESE 6210", "MEAM 5290"]),
                    single("Nanotechnology", &["MSE 0099", "MSE 6100", "MSE 5360", "ESE 3360", "ESE 4230", "ESE 5360", "ESE 6210", "MEAM 5290"]),
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
            single("Engineering", &["CIS 1100"]),
            single("Engineering", &["CIS 1200"]),
            single("Engineering", &["CIS 1210"]),
            single("Engineering", &["CIS 2400"]),
            single("Engineering", &["CIS 2620"]),
            single("Engineering", &["CIS 3200"]),
            single("Engineering", &["CIS 4480", "CIS 5480"]),
            single("Engineering", &["CIS 4710", "CIS 5710"]),
            single("Engineering", &["CIS 4000", "CIS 4100"]),
            single("Engineering", &["CIS 4010", "CIS 4110"]),

            any_of("Engineering", vec![
                    restriction(1).category("CIS Elective").departments(&["CIS"]).into(),
                    restriction(1).category("CIS Elective").departments(&["NETS"]).into(),
                ]),
            any_of("Engineering", vec![
                    restriction(1).category("CIS Elective").departments(&["CIS"]).into(),
                    restriction(1).category("CIS Elective").departments(&["NETS"]).into(),
                ]),
            any_of("Engineering", vec![
                    restriction(1).category("CIS Elective").departments(&["CIS"]).into(),
                    restriction(1).category("CIS Elective").departments(&["NETS"]).into(),
                ]),
            any_of("Engineering", vec![
                    restriction(1).category("CIS Elective").departments(&["CIS"]).into(),
                    restriction(1).category("CIS Elective").departments(&["NETS"]).into(),
                ]),
            
            // Math and Natural Science
            single("Math and Natural Science", &["MATH 1400"]),
            single("Math and Natural Science", &["MATH 1410", "MATH 1610"]),
            single("Math and Natural Science", &["MATH 2400", "ESE 2030", "MATH 2600", "MATH 3120", "MATH 3130", "MATH 3140"]),
            restriction(1).category("Math and Natural Science").attr(&["EUMA", "EUNS"]).into(),
            
            single("Math and Natural Science", &["PHYS 0151", "PHYS 0171", "ESE 1120"]),
            single("Math and Natural Science", &["CIS 1600"]),
            
            any_of("Math and Natural Science", vec![
                    code(&["PHYS 0150"]),
                    all_of(None, vec![
                        code(&["MEAM 1100"]),
                        code(&["MEAM 1470"]),
                    ]),
                ]),

            single("Math and Natural Science", &["CIS 2610", "ESE 3010", "STAT 4300"]),

            // Technical Electives
            restriction(1).category("Technical Electives").attr(&["EUCU", "EUCR"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUCU"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUCU"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUCU"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUCU"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUCU"]).into(),

            // General Electives
            single("General Electives", &["EAS 2030", "CIS 4230", "CIS 5230", "LAWM 5060"]),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),
            
            // Free Elective
            restriction(1).category("Free Elective").into()
        ],
        schedule_hints: seas_schedule_hints(&CIS_SCHEDULE),
        concentrations: Some(BTreeMap::from([
            (
                "Computer Vision".to_string(),
                vec![
                    single("Computer Vision", &["CIS 5800"]),
                    single("Computer Vision", &["CIS 5810"]),
                    single("Computer Vision", &["CIS 6800"]),
                    single("Computer Vision", &["CIS 5200"]),
                ],
            ),
            (
                "Systems".to_string(),
                vec![
                    single("Systems", &["NETS 2120", "CIS 3310", "CIS 4510", "CIS 5510", "CIS 4410", "CIS 5410", "CIS 4500", "CIS 5500", "CIS 4550", "CIS 5550", "CIS 5050", "CIS 5530"]),
                    single("Systems", &["NETS 2120", "CIS 3310", "CIS 4510", "CIS 5510", "CIS 4410", "CIS 5410", "CIS 4500", "CIS 5500", "CIS 4550", "CIS 5550", "CIS 5050", "CIS 5530"]),
                    single("Systems", &["NETS 2120", "CIS 3310", "CIS 4510", "CIS 5510", "CIS 4410", "CIS 5410", "CIS 4500", "CIS 5500", "CIS 4550", "CIS 5550", "CIS 5050", "CIS 5530"]),
                    single("Systems", &["NETS 2120", "CIS 3310", "CIS 4510", "CIS 5510", "CIS 4410", "CIS 5410", "CIS 4500", "CIS 5500", "CIS 4550", "CIS 5550", "CIS 5050", "CIS 5530"]),
                    single("Systems", &["NETS 2120", "CIS 3310", "CIS 4510", "CIS 5510", "CIS 4410", "CIS 5410", "CIS 4500", "CIS 5500", "CIS 4550", "CIS 5550", "CIS 5050", "CIS 5530"]),
                ],
            ),
            (
                "Artificial Intelligence".to_string(),
                vec![
                    single("Artificial Intelligence", &["CIS 4210", "CIS 5210"]),
                    single("Artificial Intelligence", &["CIS 4190", "CIS 5190", "CIS 5200"]),
                    single("Artificial Intelligence", &["MEAM 5100", "MEAM 5200", "CIS 5220", "CIS 5300", "CIS 5800", "CIS 5810", "NETS 2130"]),
                    single("Artificial Intelligence", &["MEAM 5100", "MEAM 5200", "CIS 5220", "CIS 5300", "CIS 5800", "CIS 5810", "NETS 2130"]),
                ],
            ),
            (
                "Software Foundations".to_string(),
                vec![
                    single("Software Foundations", &["CIS 3410"]),
                    single("Software Foundations", &["CIS 5000"]),
                    single("Software Foundations", &["CIS 3500", "CIS 5400", "CIS 5470", "CIS 5520", "CIS 6730", "CIS 6820"]),
                    single("Software Foundations", &["CIS 3500", "CIS 5400", "CIS 5470", "CIS 5520", "CIS 6730", "CIS 6820"]),
                ],
            ),
            (
                "Data Science".to_string(),
                vec![
                    single("Data Science", &["CIS 4190", "CIS 5190", "CIS 5200", "ESE 5450", "STAT 4710"]),
                    single("Data Science", &["CIS 5450", "ESE 3050"]),
                    single("Data Science", &["ENM 3210", "ESE 4020", "STAT 4310"]),
                    single("Data Science", &["CIS 4500", "CIS 5500", "CIS 4550", "CIS 5550", "CIS 4210", "CIS 5210", "CIS 5220", "CIS 5800", "STAT 4350", "STAT 4740", "STAT 4760", "STAT 5200"]),
                ],
            ),
            (
                "Cognitive Science".to_string(),
                vec![
                    single("Cognitive Science", &["COGS 1001", "CIS 1400"]),
                    single("Cognitive Science", &["CIS 4210", "CIS 5210", "CIS 5200", "CIS 5300"]),
                    single("Cognitive Science", &["LING 0500", "LING 2300", "LING 2500", "LING 2700"]),
                    single("Cognitive Science", &["PSYC 1210", "PSYC 1340", "PSYC 1230", "PSYC 1310", "PSYC 2737", "PSYC 2377"]),
                    single("Cognitive Science", &["PHIL 1710", "PHIL 2640", "PHIL 4721", "PHIL 4840"]),
                ],
            ),
            (
                "Computational Biology".to_string(),
                vec![
                    single("Computational Biology", &["BIOL 1101", "BIOL 1121"]),
                    single("Computational Biology", &["BIOL 2210"]),
                    single("Computational Biology", &["ENM 3210", "ESE 4020", "STAT 4310", "BIOL 2510"]),
                    single("Computational Biology", &["CIS 5450", "ESE 3050", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 5450", "STAT 4710"]),
                    single("Computational Biology", &["BIOL 4231", "BE 5690", "BE 4800", "BE 3060"]),
                    single("Computational Biology", &["BIOL 4526", "CIS 4360", "BIOL 5536", "GCB 5360", "BIOL 5571"]),
                ],
            ),
        ])),
    };
}

fn cis_or_nets_elective(_category: &str) -> Requirement {
    restriction(1)
        .category("CIS Elective")
        .departments(&["CIS", "NETS"])
        .excluding(&[
            "CIS 1050", "CIS 1060", "CIS 1070", "CIS 1250", "CIS 1600", "CIS 2610",
            "CIS 3333", "CIS 4230", "CIS 5230", "CIS 7980",
        ])
        .into()
}

const DMD_ELECTIVE_DEPTS: &[&str] = &[
    "COMM", "FNAR", "CIMS", "DSGN", "THAR", "MKTG", "ARTH", "IPD", "MUSC", "EDUC",
];

fn dmd_advisor_elective() -> Requirement {
    restriction(1)
        .category("DMD Electives")
        .departments(DMD_ELECTIVE_DEPTS)
        .into()
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
            any_of("Engineering", vec![
                    code(&["CIS 1100"]),
                    cis_or_nets_elective("Engineering"),
                ]),
            single("Engineering", &["CIS 1200"]),
            single("Engineering", &["CIS 1210"]),
            single("Engineering", &["CIS 2400"]),
            single("Engineering", &["CIS 2620"]),
            single("Engineering", &["CIS 3200"]),
            single("Engineering", &["CIS 4600", "CIS 5600"]),
            single("Engineering", &["CIS 4610", "CIS 5610", "CIS 4620", "CIS 5620", "CIS 4550", "CIS 5550"]),
            single("Engineering", &["CIS 4970"]),
            cis_or_nets_elective("Engineering"),
            cis_or_nets_elective("Engineering"),
            cis_or_nets_elective("Engineering"),
            cis_or_nets_elective("Engineering"),

            // Math and Natural Science
            single("Math and Natural Science", &["MATH 1400"]),
            single("Math and Natural Science", &["MATH 1410", "MATH 1610"]),
            single("Math and Natural Science", &["MATH 2400", "ESE 2030", "ENM 2030", "ENM 2400"]),
            single("Math and Natural Science", &["CIS 1600"]),
            single("Math and Natural Science", &["CIS 2610", "ESE 3010", "STAT 4300"]),
            any_of("Math and Natural Science", vec![
                    code(&["PHYS 0150", "PHYS 0170"]),
                    all_of(None, vec![
                        code(&["MEAM 1100"]),
                        code(&["MEAM 1470"]),
                    ]),
                ]),
            any_of("Math and Natural Science", vec![
                    code(&["BIOL 1101"]),
                    all_of(None, vec![
                        code(&["BIOL 1121"]),
                        code(&["BIOL 1124"]),
                    ]),
                    all_of(None, vec![
                        code(&["CHEM 1012"]),
                        code(&["CHEM 1101"]),
                    ]),
                    code(&["ESE 1120"]),
                    code(&["PHYS 0151"]),
                    code(&["PHYS 0171"]),
                ]),
            restriction(1).category("Math and Natural Science").attr(&["EUMA", "EUNS"]).into(),

            // DMD Electives
            single("DMD Electives", &["FNAR 0010", "FNAR 2200", "FNAR 1080"]),
            single("DMD Electives", &["DSGN 1030", "DSGN 2010"]),
            single("DMD Electives", &["DSGN 2040", "FNAR 1050", "FNAR 2090", "FNAR 2100"]),
            dmd_advisor_elective(),
            dmd_advisor_elective(),
            dmd_advisor_elective(),

            // General Electives — 5 SSH + 2 SSH/TBS (writing seminar via EUHS)
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),

            // Free Elective
            restriction(1).category("Free Elective").into(),
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
            single("Engineering", &["CIS 1100"]),
            single("Engineering", &["CIS 1200"]),
            single("Engineering", &["CIS 1210"]),
            single("Engineering", &["CIS 2450", "CIS 5450"]),
            single("Engineering", &["CIS 3200"]),
            
            // Math and Natural Science
            single("Math and Natural Science", &["MATH 1400"]),
            single("Math and Natural Science", &["MATH 1410", "MATH 1610"]),
            single("Math and Natural Science", &["ESE 2030"]),
            restriction(1).category("Math and Natural Science").attr(&["EUNS"]).into(),
            
            single("Math and Natural Science", &["CIS 1600"]),
            single("Math and Natural Science", &["ESE 3010", "STAT 4300"]),
            single("Math and Natural Science", &["ESE 4020", "ESE 5420"]),

            // AI
            single("Artificial Intelligence - Introduction to AI", &["CIS 4210", "CIS 5210", "ESE 2000"]),
            single("Artificial Intelligence - Machine Learning", &["CIS 4190", "CIS 5190", "CIS 5200"]),
            single("Artificial Intelligence - Signals & Systems", &["ESE 2100", "ESE 2240"]),
            single("Artificial Intelligence - Optimization & Control", &["ESE 3040", "ESE 4210"]),
            single("Artificial Intelligence - Vision & Language", &["CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810"]),
            single("Artificial Intelligence - AI Project", &["CIS 3500", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "ESE 3060", "ESE 3600", "ESE 4210", "NETS 2120", "NETS 2130"]),

            single("AI Electives", &["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"]),
            single("AI Electives", &["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"]),
            single("AI Electives", &["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"]),
            single("AI Electives", &["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"]),
            single("AI Electives", &["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"]),
            single("AI Electives", &["CIS 4210", "CIS 5210", "ESE 2000", "CIS 4190", "CIS 5190", "CIS 5200", "ESE 2100", "ESE 2240", "ESE 3040", "ESE 4210", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 3500", "ESE 3060", "ESE 3600", "NETS 2120", "NETS 2130"]),

            single("AI Electives", &["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"]),
            single("AI Electives", &["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"]),
            single("AI Electives", &["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"]),
            single("AI Electives", &["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"]),
            single("AI Electives", &["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"]),
            single("AI Electives", &["CIS 3333", "CIS 4270", "CIS 5270", "CIS 6200", "CIS 6250", "ESE 4380", "ESE 5380", "ESE 5140", "ESE 5460", "ESE 6450", "ESE 6740", "ESE 3030", "ESE 5000", "ESE 5050", "ESE 5060", "ESE 6050", "ESE 6060", "ESE 6180", "ESE 6190", "BE 5210", "CIS 4120", "CIS 5120", "CIS 4500", "CIS 5500", "CIS 5360", "CIS 5800", "CIS 6500", "MEAM 5200", "MEAM 6200", "ESE 4040", "ESE 6150", "ESE 6500", "ESE 6510", "NETS 3120", "NETS 4120"]),

            // Senior Design
            single("Senior Design", &["CIS 4000", "CIS 4100", "ESE 4500", "MEAM 4450", "BE 4950", "MSE 4950", "CBE 4000"]),
            single("Senior Design", &["CIS 4010", "CIS 4110", "ESE 4510", "MEAM 4460", "BE 4960", "MSE 4960", "CBE 4590"]),
            
            // Technical Electives
            restriction(1).category("Technical Electives").attr(&["EUNG"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUNG"]).into(),
            restriction(1).category("Technical Electives").attr(&["EUNG"]).into(),
            
            // General Electives
            single("General Electives", &["LAWM 5060", "CIS 4230", "CIS 5230"]),
            single("General Electives", &["COGS 1001", "LING 0500", "LING 2500", "LING 3810", "PHIL 1710", "PHIL 2640", "PHIL 4721", "PHIL 4840", "PSYC 1210", "PSYC 1340", "PSYC 1230", "PSYC 1310", "PSYC 2737"]),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),
            
            // Free Elective
            restriction(1).category("Free Elective").into()
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
            single("Engineering", &["ESE 1110", "ESE 3600"]),
            single("Engineering", &["CIS 1100"]),
            single("Engineering", &["CIS 1200"]),
            single("Engineering", &["CIS 1210"]),
            single("Engineering", &["ESE 2150"]),
            single("Engineering", &["CIS 2400"]),
            single("Engineering", &["ESE 3500"]),
            single("Engineering", &["ESE 3700"]),
            single("Engineering", &["CIS 4480", "CIS 5480"]),
            single("Engineering", &["CIS 4710", "CIS 5710"]),
            
            // Intermediate CIS or ESE Elective
            restriction(1).category("Intermediate CIS or ESE Elective").departments(&["ESE", "CIS"]).level(2000).excluding(&["ESE 3010", "ESE 4020", "ESE 2030"]).into(),
            
            // Advanced CIS or ESE courses
            restriction(1).category("Advanced CIS or ESE Electives").departments(&["ESE", "CIS"]).level(3000).excluding(&["ESE 3010", "ESE 4020", "ESE 2030"]).into(),
            restriction(1).category("Advanced CIS or ESE Electives").departments(&["ESE", "CIS"]).level(3000).excluding(&["ESE 3010", "ESE 4020", "ESE 2030"]).into(),

            // Design and Project courses
            single("Design and Project courses", &["CIS 4000", "ESE 4500"]),
            single("Design and Project courses", &["CIS 4010", "ESE 4510"]),

            // Math and Natural Science
            single("Math and Natural Science", &["MATH 1400"]),
            single("Math and Natural Science", &["MATH 1410", "MATH 1610"]),
            single("Math and Natural Science", &["ESE 2030", "MATH 2400", "MATH 2600"]),
            single("Math and Natural Science", &["ESE 3010", "CIS 2610", "STAT 4300"]),
            single("Math and Natural Science", &["CIS 1600"]),
            any_of("Math and Natural Science", vec![
                    code(&["PHYS 0150"]),
                    all_of(None, vec![
                        code(&["MEAM 1100"]),
                        code(&["MEAM 1470", "BIOL 1124", "PHYS 0050", "CHEM 1101"]),
                    ]),
                    all_of(None, vec![
                        code(&["PHYS 0140"]),
                        code(&["BIOL 1124", "PHYS 0050", "MEAM 1470", "CHEM 1101"]),
                    ]),
                    code(&["PHYS 0170"]),
                ]),
            single("Math and Natural Science", &["ESE 1120"]),
            single("Math and Natural Science", &["CHEM 1012", "EAS 0091", "BIOL 1121", "BIOL 1101", "PHYS 1240"]),
            restriction(1).category("Math and Natural Science").attr(&["EUMA", "EUNS"]).into(),

            // Professional Electives
            restriction(1).category("Professional Electives").attr(&["EUNG"]).into(),
            restriction(1).category("Professional Electives").attr(&["EUNG"]).into(),
            any_of("Professional Electives", vec![
                    restriction(1).attr(&["EUNG"]).into(),
                    code(&["ESE 4000", "EAS 5450", "ESE 5950", "MGMT 2370", "OIDD 2360"]),
                ]),

            // General Electives
            single("General Electives", &["LAWM 5060", "EAS 2030", "CIS 4230", "CIS 5230"]),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),
            restriction(1).category("General Electives").attr(&["EUHS", "EUSS", "EUTB"]).into(),

            // Free Elective
            restriction(1).category("Free Elective").into(),
        ],
        schedule_hints: seas_schedule_hints(&CMPE_SCHEDULE),
        concentrations: Some(BTreeMap::from([
            (
                "AI & Robotics".to_string(),
                vec![
                    single("AI & Robotics", &["CIS 2450", "CIS 4190", "CIS 5190", "CIS 5200", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 5210", "CIS 5650", "ESE 2000", "ESE 2240", "ESE 3060", "ESE 3600", "ESE 4210", "ESE 5050", "ESE 5390", "ESE 6150", "ESE 6500", "MEAM 5200"]),
                    single("AI & Robotics", &["CIS 2450", "CIS 4190", "CIS 5190", "CIS 5200", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 5210", "CIS 5650", "ESE 2000", "ESE 2240", "ESE 3060", "ESE 3600", "ESE 4210", "ESE 5050", "ESE 5390", "ESE 6150", "ESE 6500", "MEAM 5200"]),
                    single("AI & Robotics", &["CIS 2450", "CIS 4190", "CIS 5190", "CIS 5200", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 5210", "CIS 5650", "ESE 2000", "ESE 2240", "ESE 3060", "ESE 3600", "ESE 4210", "ESE 5050", "ESE 5390", "ESE 6150", "ESE 6500", "MEAM 5200"]),
                    single("AI & Robotics", &["CIS 2450", "CIS 4190", "CIS 5190", "CIS 5200", "CIS 4300", "CIS 5300", "CIS 4810", "CIS 5810", "CIS 5210", "CIS 5650", "ESE 2000", "ESE 2240", "ESE 3060", "ESE 3600", "ESE 4210", "ESE 5050", "ESE 5390", "ESE 6150", "ESE 6500", "MEAM 5200"]),
                ],
            ),
            (
                "CHIPs".to_string(),
                vec![
                    single("Chips", &["ESE 3190", "ESE 4190", "ESE 5720", "ESE 4730", "ESE 5730", "ESE 4750", "ESE 5750", "ESE 5320", "ESE 5780", "ESE 5800", "ESE 6680", "CIS 6010"]),
                    single("Chips", &["ESE 3190", "ESE 4190", "ESE 5720", "ESE 4730", "ESE 5730", "ESE 4750", "ESE 5750", "ESE 5320", "ESE 5780", "ESE 5800", "ESE 6680", "CIS 6010"]),
                    single("Chips", &["ESE 3190", "ESE 4190", "ESE 5720", "ESE 4730", "ESE 5730", "ESE 4750", "ESE 5750", "ESE 5320", "ESE 5780", "ESE 5800", "ESE 6680", "CIS 6010"]),
                    single("Chips", &["ESE 3190", "ESE 4190", "ESE 5720", "ESE 4730", "ESE 5730", "ESE 4750", "ESE 5750", "ESE 5320", "ESE 5780", "ESE 5800", "ESE 6680", "CIS 6010"]),
                ],
            ),
            (
                "Networks".to_string(),
                vec![
                    single("Networks", &["NETS 2120", "ESE 4070", "ESE 5070", "CIS 5530", "ESE 6650", "CIS 4550", "CIS 5550", "CIS 5050"]),
                    single("Networks", &["NETS 2120", "ESE 4070", "ESE 5070", "CIS 5530", "ESE 6650", "CIS 4550", "CIS 5550", "CIS 5050"]),
                    single("Networks", &["NETS 2120", "ESE 4070", "ESE 5070", "CIS 5530", "ESE 6650", "CIS 4550", "CIS 5550", "CIS 5050"]),
                    single("Networks", &["NETS 2120", "ESE 4070", "ESE 5070", "CIS 5530", "ESE 6650", "CIS 4550", "CIS 5550", "CIS 5050"]),
                ],
            ),
            (
                "Security and Safety".to_string(),
                vec![
                    single("Security and Safety", &["CIS 2330", "CIS 4410", "CIS 5410", "CIS 4510", "CIS 5470", "CIS 5560", "ESE 5370"]),
                    single("Security and Safety", &["CIS 2330", "CIS 4410", "CIS 5410", "CIS 4510", "CIS 5470", "CIS 5560", "ESE 5370"]),
                    single("Security and Safety", &["CIS 2330", "CIS 4410", "CIS 5410", "CIS 4510", "CIS 5470", "CIS 5560", "ESE 5370"]),
                    single("Security and Safety", &["CIS 2330", "CIS 4410", "CIS 5410", "CIS 4510", "CIS 5470", "CIS 5560", "ESE 5370"]),
                ],
            ),
        ])),
    };
}

fn be_conc_slot(category: &str, courses: &[&str]) -> Requirement {
    single(category, courses)
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

fn be_general_electives_pool() -> Requirement {
    course_pool(
        "General Electives",
        vec![],
        7,
        vec![
            single_pool_constraint(
                "Engineering Ethics",
                &["EAS 2030", "HSOC 1330", "HSOC 2457", "LGST 1000", "LGST 2200", "NURS 3300", "NURS 5250", "BIOE 4010", "BIOE 4020", "PHIL 1342", "PHIL 4330"],
                1,
                "be:ethics",
            ),
            attrs_pool_constraint("Social Science", &["EUSS"], 2, "be:distribution"),
            attrs_pool_constraint("Humanities", &["EUHS"], 2, "be:distribution"),
            attrs_pool_constraint("SSH Elective", &["EUSS", "EUHS"], 1, "be:distribution"),
            attrs_pool_constraint(
                "Technology & Business",
                &["EUHS", "EUSS", "EUTB"],
                2,
                "be:distribution",
            ),
        ],
    )
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
            single("Engineering", &["BE 1000"]),
            single("Engineering", &["ENGR 1050"]),
            single("Engineering", &["BE 2000"]),
            single("Engineering", &["BE 2200"]),
            single("Engineering", &["BE 2700"]),
            single("Engineering", &["BE 3010"]),
            single("Engineering", &["BE 3060"]),
            single("Engineering", &["BE 3090"]),
            single("Engineering", &["BE 3100"]),
            single("Engineering", &["BE 3500"]),
            single("Engineering", &["BE 4950"]),
            single("Engineering", &["BE 4960"]),
            restriction(1).category("BE Elective (4000 or 5000 level)").departments(&["BE"]).level(4000).into(),
            restriction(1).category("BE Elective (4000 or 5000 level)").departments(&["BE"]).level(4000).into(),
            restriction(1).category("Engineering Elective").attr(&["EUNG"]).into(),
            restriction(1).category("Engineering Elective").attr(&["EUNG"]).into(),

            // Math and Natural Science
            single("Math and Natural Science", &["MATH 1400"]),
            single("Math and Natural Science", &["MATH 1410"]),
            single("Math and Natural Science", &["ESE 2030", "ENM 2400", "MATH 2400", "ENM 2030"]),
            single("Math and Natural Science", &["ENM 3750", "ENGR 3440"]),
            single("Math and Natural Science", &["PHYS 0140"]),
            single("Math and Natural Science", &["PHYS 0141"]),
            single("Math and Natural Science", &["CHEM 1012", "CHEM 1151"]),
            single("Math and Natural Science", &["CHEM 1101"]),
            single("Math and Natural Science", &["CHEM 1102"]),
            single("Math and Natural Science", &["CHEM 1022", "CHEM 1161"]),
            single("Math and Natural Science", &["BIOL 1121"]),
            single("Math and Natural Science", &["BIOL 1123"]),
            single("Math and Natural Science", &["BIOL 3310"]),

            be_general_electives_pool(),

            // Free Electives
            restriction(1).category("Free Elective").into(),
            restriction(1).category("Free Elective").into(),
            restriction(1).category("Free Elective").into(),
        ],
        schedule_hints: seas_schedule_hints(&BE_SCHEDULE),
        concentrations: Some(be_concentrations()),
    }
}

const EENT_CORE_EXCLUSIONS: &[&str] = &[
    "EAS 5450", "EAS 5460", "EAS 5490", "EAS 5410", "EAS 5430",
];

pub fn eent_concentration_names() -> Vec<String> {
    vec!["Standard".to_string(), "Fellows".to_string()]
}

fn eent_elective_slot() -> Requirement {
    restriction(1)
        .category("EENT Electives")
        .cu(10)
        .attr(&["EUNP"])
        .excluding(EENT_CORE_EXCLUSIONS)
        .into()
}

fn eent_elective_slots() -> Vec<Requirement> {
    (0..4).map(|_| eent_elective_slot()).collect()
}

/// Engineering Entrepreneurship minor (6 CU) per Penn catalog.
pub fn create_eent_minor(concentration: &str) -> Major {
    let core = if concentration == "Fellows" {
        vec![
            single("EENT Core", &["EAS 5410"]),
            single("EENT Core", &["EAS 5430"]),
        ]
    } else {
        vec![
            single("EENT Core", &["EAS 5450"]),
            any_of("EENT Core", vec![
                    single("EENT Core", &["EAS 5460"]),
                    single("EENT Core", &["EAS 5490"]),
                ]),
        ]
    };

    Major {
        short_name: "EENT".to_string(),
        name: "Engineering Entrepreneurship".to_string(),
        requirements: core.into_iter().chain(eent_elective_slots()).collect(),
        concentrations: None,
        schedule_hints: HashMap::new(),
    }
}