use std::collections::HashMap;

use crate::Major;
use crate::Requirement;

fn placeholder_ms_major(short_name: &str, display_name: &str) -> Major {
    Major {
        short_name: short_name.to_string(),
        name: display_name.to_string(),
        requirements: vec![Requirement::Restriction {
            category: Some("Program Requirements (placeholder)".to_string()),
            department: None,
            cu: None,
            level: None,
            attr: None,
            excluding: None,
            number: 10,
            no_school: None,
        }],
        schedule_hints: HashMap::new(),
        concentrations: None,
    }
}

pub fn create_ms_ee_major() -> Major {
    // TODO: populate MS Electrical Engineering requirements
    placeholder_ms_major("MS_EE", "Electrical Engineering, MSE")
}

fn ms_robo_ai_area() -> Requirement {
    Requirement::SingleCourse {
        category: Some("Artificial Intelligence".to_string()),
        possibilities: vec![
            "CIS 5190".to_string(),
            "CIS 5200".to_string(),
            "CIS 5210".to_string(),
            "ESE 6500".to_string(),
        ],
    }
}

fn ms_robo_robot_design_area() -> Requirement {
    Requirement::SingleCourse {
        category: Some("Robot Design and Analysis".to_string()),
        possibilities: vec![
            "MEAM 5100".to_string(),
            "MEAM 5200".to_string(),
            "MEAM 6200".to_string(),
        ],
    }
}

fn ms_robo_control_area() -> Requirement {
    Requirement::SingleCourse {
        category: Some("Control".to_string()),
        possibilities: vec![
            "ESE 5000".to_string(),
            "ESE 5050".to_string(),
            "MEAM 5130".to_string(),
            "MEAM 5170".to_string(),
        ],
    }
}

fn ms_robo_perception_area() -> Requirement {
    Requirement::SingleCourse {
        category: Some("Perception".to_string()),
        possibilities: vec![
            "CIS 5800".to_string(),
            "CIS 5810".to_string(),
            "CIS 6800".to_string(),
        ],
    }
}

/// Complete 1 course from 3 out of 4 foundational areas (3 courses total).
fn ms_robo_foundational_courses() -> Requirement {
    Requirement::CourseGroup {
        category: Some("Foundational Courses".to_string()),
        number: 3,
        possibilities: vec![
            ms_robo_ai_area(),
            ms_robo_robot_design_area(),
            ms_robo_control_area(),
            ms_robo_perception_area(),
        ],
    }
}

pub fn create_ms_robo_major() -> Major {
    Major {
        short_name: "MS_ROBO".to_string(),
        name: "Robotics, MSE".to_string(),
        requirements: vec![
            ms_robo_foundational_courses(),
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Technical Elective".to_string()), department: None, cu: None, level: None, attr: Some(vec!["EMRT".to_string()]), excluding: None, number: 1, no_school: None },

            Requirement::Restriction { 
                category: Some("General Elective".to_string()), 
                department: Some(vec![
                    "CIS".to_string(),
                    "ESE".to_string(),
                    "MEAM".to_string(),
                    "EAS".to_string(),
                    "CIT".to_string(),
                    "ENM".to_string(),
                    "IPD".to_string(),
                    "MATH".to_string(),
                ]), cu: None, level: Some(5000), attr: None, 
                excluding: Some(vec!["EAS 8950".to_string(), "EAS 8960".to_string(), "EAS 8970".to_string()]), number: 1, no_school: None 
            },
            Requirement::Restriction { 
                category: Some("General Elective".to_string()), 
                department: Some(vec![
                    "CIS".to_string(),
                    "ESE".to_string(),
                    "MEAM".to_string(),
                    "EAS".to_string(),
                    "CIT".to_string(),
                    "ENM".to_string(),
                    "IPD".to_string(),
                    "MATH".to_string(),
                ]), cu: None, level: Some(5000), attr: None, 
                excluding: Some(vec!["EAS 8950".to_string(), "EAS 8960".to_string(), "EAS 8970".to_string()]), number: 1, no_school: None 
            },
            Requirement::Restriction { 
                category: Some("General Elective".to_string()), 
                department: Some(vec![
                    "CIS".to_string(),
                    "ESE".to_string(),
                    "MEAM".to_string(),
                    "EAS".to_string(),
                    "CIT".to_string(),
                    "ENM".to_string(),
                    "IPD".to_string(),
                    "MATH".to_string(),
                ]), cu: None, level: Some(5000), attr: None, 
                excluding: Some(vec!["EAS 8950".to_string(), "EAS 8960".to_string(), "EAS 8970".to_string()]), number: 1, no_school: None 
            },
        ],
        schedule_hints: HashMap::new(),
        concentrations: None,
    }
}

pub fn create_ms_meam_major() -> Major {
    // TODO: populate MS Mechanical Engineering and Applied Mechanics requirements
    placeholder_ms_major("MS_MEAM", "Mechanical Engineering and Applied Mechanics, MSE")
}

pub fn create_ms_cis_major() -> Major {
    // TODO: populate MS Computer Science requirements
    placeholder_ms_major("MS_CIS", "Computer Science, MSE")
}

pub fn create_ms_mse_major() -> Major {
    // TODO: populate MS Materials Science and Engineering requirements
    placeholder_ms_major("MS_MSE", "Materials Science and Engineering, MSE")
}

