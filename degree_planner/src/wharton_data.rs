use std::collections::BTreeMap;

use crate::Requirement;
use crate::Major;
use crate::requirement::PoolConstraint;
use crate::schedule_template::{
    append_semester, scheduled, Y1F, Y1S, Y2F, Y2S, Y3F, Y3S,
};

fn wh_attr_constraint(label: &str, attr: &str, count: i32, group: &str) -> PoolConstraint {
    PoolConstraint {
        requirement: Requirement::Restriction {
            category: Some(label.to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(vec![attr.to_string()]),
            excluding: None,
            number: 1,
            no_school: None,
        },
        count,
        consumption_group: Some(group.to_string()),
    }
}

fn wh_attrs_constraint(label: &str, attrs: &[&str], count: i32, group: &str) -> PoolConstraint {
    PoolConstraint {
        requirement: Requirement::Restriction {
            category: Some(label.to_string()),
            department: None,
            cu: None,
            level: None,
            attr: Some(attrs.iter().map(|s| s.to_string()).collect()),
            excluding: None,
            number: 1,
            no_school: None,
        },
        count,
        consumption_group: Some(group.to_string()),
    }
}

fn wh_non_wh_constraint(label: &str, count: i32) -> PoolConstraint {
    PoolConstraint {
        requirement: Requirement::Restriction {
            category: Some(label.to_string()),
            department: None,
            cu: None,
            level: None,
            attr: None,
            excluding: None,
            number: 1,
            no_school: Some("WH".to_string()),
        },
        count,
        consumption_group: Some("wh:non_wh".to_string()),
    }
}

/// WH_FL: 7 LAS courses, 11 coverage units. Double-count policy via consumption groups:
/// - `wh:cc_fl`: FL + CC slots are mutually exclusive per course
/// - `wh:ssh`: WUHM / WUSS / WUNM mutually exclusive per course (CC may overlap)
/// - `wh:non_wh`: non-Wharton slots (CC and FL may overlap)
fn wh_fl_las_pool() -> Requirement {
    Requirement::CoursePool {
        category: Some("Liberal Arts and Sciences".to_string()),
        fixed_slots: vec![],
        flexible_slots: 7,
        constraints: vec![
            wh_attr_constraint("Humanities (WUHM)", "WUHM", 1, "wh:ssh"),
            wh_attr_constraint("Natural Science & Math (WUNM)", "WUNM", 1, "wh:ssh"),
            wh_attr_constraint("Social Science (WUSS)", "WUSS", 1, "wh:ssh"),
            wh_non_wh_constraint("Non-Wharton course", 3),
            wh_attr_constraint("Foreign Language (WUFL)", "WUFL", 2, "wh:cc_fl"),
            wh_attr_constraint("Cross-Cultural (WUCN)", "WUCN", 2, "wh:cc_fl"),
            wh_attrs_constraint(
                "Cross-Cultural (WUCN/WUCU)",
                &["WUCN", "WUCU"],
                1,
                "wh:cc_fl",
            ),
        ],
    }
}

/// WH_NOFL SSH: CC may double-count into SSH and non-Wharton; SSH tags are mutually exclusive.
fn wh_ssh_las_pool() -> Requirement {
    Requirement::CoursePool {
        category: Some("Liberal Arts and Sciences - SSH".to_string()),
        fixed_slots: vec![],
        flexible_slots: 6,
        constraints: vec![
            wh_attrs_constraint("Humanities (WUHM)", &["WUHM"], 1, "wh:ssh"),
            wh_attr_constraint("Natural Science & Math (WUNM)", "WUNM", 1, "wh:ssh"),
            wh_attr_constraint("Social Science (WUSS)", "WUSS", 1, "wh:ssh"),
            wh_non_wh_constraint("Non-Wharton course", 3),
            wh_attr_constraint("Cross-Cultural (WUCN)", "WUCN", 2, "wh:cross_cultural"),
            wh_attrs_constraint(
                "Cross-Cultural (WUCN/WUCU)",
                &["WUCN", "WUCU"],
                1,
                "wh:cross_cultural",
            ),
        ],
    }
}

/// M&T FL-required LAS: 4 courses, 6 coverage units. The four non-FL requirements
/// share `wh:mt_las` (no cross-double-count among them); WUFL uses `wh:wufl` and
/// may double-count with any `wh:mt_las` slot.
fn wh_fl_mt_las_pool() -> Requirement {
    Requirement::CoursePool {
        category: Some("Liberal Arts and Sciences".to_string()),
        fixed_slots: vec![],
        flexible_slots: 4,
        constraints: vec![
            wh_attrs_constraint(
                "Humanities / Social Science (WUHM/WUSS)",
                &["WUHM", "WUSS"],
                1,
                "wh:mt_las",
            ),
            wh_attrs_constraint(
                "Humanities / Social Science (WUHM/WUSS)",
                &["WUHM", "WUSS"],
                1,
                "wh:mt_las",
            ),
            wh_attr_constraint("Cross-Cultural (WUCN)", "WUCN", 1, "wh:mt_las"),
            wh_attrs_constraint(
                "Cross-Cultural (WUCN/WUCU)",
                &["WUCN", "WUCU"],
                1,
                "wh:mt_las",
            ),
            wh_attr_constraint("Foreign Language (WUFL)", "WUFL", 1, "wh:wufl"),
            wh_attr_constraint("Foreign Language (WUFL)", "WUFL", 1, "wh:wufl"),
        ],
    }
}

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
                    cu: None, level: None, attr: Some(vec!["WUBO".to_string()]), excluding: None, number: 1, no_school: None 
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

/// One fewer breadth slot when double concentrating (one breadth may count toward a conc).
fn wh_bb_slot_labels(default_labels: &[&str], concentrations: &[String]) -> Vec<String> {
    let mut labels: Vec<String> = default_labels.iter().map(|s| s.to_string()).collect();
    if concentrations.len() >= 2 && labels.len() > 1 {
        labels.pop();
    }
    labels
}

/// M&T needs two business breadths. MGMT 2370 counts as breadth I when MGMT is not a
/// concentration; with two non-MGMT concentrations, a concentration course covers breadth II
/// as well — no standalone breadth slots remain.
fn mt_business_breadth_labels(concentrations: &[String]) -> Vec<String> {
    let mgmt_is_conc = concentrations.iter().any(|c| c == "MGMT");
    let double_conc = concentrations.len() >= 2;

    if double_conc && !mgmt_is_conc {
        return vec![];
    }

    let default_labels: Vec<&str> = if mgmt_is_conc {
        vec!["Business Breadth - I", "Business Breadth - II"]
    } else {
        vec!["Business Breadth - II"]
    };
    wh_bb_slot_labels(&default_labels, concentrations)
}

fn business_breadth_requirements(
    concentrations: &[String],
    pool: &[&str],
    slot_labels: &[String],
    mt: bool,
) -> Vec<Requirement> {
    let exclusions = bb_standard_exclusions(mt);
    let opts = bb_department_options(concentrations, pool, &exclusions);
    slot_labels
        .iter()
        .map(|label| Requirement::AnyOf {
            category: Some(label.clone()),
            possibilities: opts.clone(),
        })
        .collect()
}

fn mt_mgmt2370_soph() -> Requirement {
    Requirement::SingleCourse {
        category: Some("M&T Soph Course".to_string()),
        possibilities: vec!["MGMT 2370".to_string()],
    }
}

fn wh_concentration_requirements(concentrations: &[String]) -> Vec<Requirement> {
    wh_concentration_requirements_inner(concentrations, false)
}

fn wh_concentration_requirements_skip_mgmt_first(concentrations: &[String]) -> Vec<Requirement> {
    wh_concentration_requirements_inner(concentrations, true)
}

fn wh_concentration_requirements_inner(
    concentrations: &[String],
    skip_first_mgmt: bool,
) -> Vec<Requirement> {
    let catalog = create_wh_concentrations();
    let mut reqs = Vec::new();
    for name in concentrations {
        if let Some(chain) = catalog.get(name) {
            if skip_first_mgmt && name == "MGMT" && chain.len() > 1 {
                reqs.extend(chain[1..].iter().cloned());
            } else {
                reqs.extend(chain.clone());
            }
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
    let bb_labels = wh_bb_slot_labels(&["Business Breadth"], &concs);
    let bb_reqs = business_breadth_requirements(&concs, &bb_pool, &bb_labels, false);

    let (mut requirements, mut schedule_hints) = scheduled(vec![
            // First-Year Foundations
            (Y1F, Requirement::AnyOf { category: Some("First-Year Foundations".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["BEPP 1000".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0200".to_string()] },
                ] }
            ] }),
            (Y1F, Requirement::SingleCourse { category: Some("First-Year Foundations".to_string()), possibilities: vec!["MATH 1400".to_string(), "MATH 1070".to_string()] }),
            (Y1F, Requirement::Restriction { category: Some("First-Year Foundations".to_string()), department: Some(vec!["WRIT".to_string()]), cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None }),

            // Leadership Journey
            (Y1F, Requirement::SingleCourse { category: Some("Leadership Journey".to_string()), possibilities: vec!["WH 1010".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Leadership Journey".to_string()), possibilities: vec!["WH 2010".to_string(), "WH 2011".to_string()] }),
            (Y3F, Requirement::SingleCourse { category: Some("Leadership Journey".to_string()), possibilities: vec!["MGMT 3010".to_string()] }),
            (Y3S, Requirement::Restriction { category: Some("Leadership Journey".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCP".to_string()]), excluding: None, number: 1, no_school: None }),

            // Fundamentals
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1010".to_string()] }),
            (Y2S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1020".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["BEPP 2500".to_string(), "BEPP 2508".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1000".to_string(), "FNCE 1008".to_string()] }),
            (Y2S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1010".to_string(), "FNCE 1018".to_string()] }),
            (Y1F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["LGST 1000".to_string(), "LGST 1010".to_string(), "LGST 1008".to_string(), "LGST 1018".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MGMT 1010".to_string(), "MKTG 1018".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MKTG 1010".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["OIDD 1010".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 1010".to_string(), "STAT 4300".to_string(), "ESE 3010".to_string(), "STAT 1018".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 1020".to_string(), "STAT 4310".to_string(), "ESE 4020".to_string(), "STAT 1028".to_string()] }),

            // Flex Fundamentals
            (Y3F, Requirement::Restriction { category: Some("Flex Fundamentals".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUGE".to_string()]), excluding: None, number: 1, no_school: None }),
            (Y3S, Requirement::Restriction { category: Some("Flex Fundamentals".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUTI".to_string()]), excluding: None, number: 1, no_school: None }),
    ]);

    append_semester(&mut requirements, &mut schedule_hints, Y3F, bb_reqs);
    append_semester(&mut requirements, &mut schedule_hints, Y2S, vec![
            wh_fl_las_pool(),
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
    ]);
    append_semester(&mut requirements, &mut schedule_hints, Y3F, wh_concentration_requirements(&concs));

    return Major {
        short_name: "WH".to_string(), 
        name: "Wharton Undergraduate".to_string(), 
        requirements,
        schedule_hints,
        concentrations: Some(wh_concentrations),
    }
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
    let bb_labels = wh_bb_slot_labels(
        &["Business Breadth 1", "Business Breadth 2", "Business Breadth 3"],
        &concs,
    );
    let bb_reqs = business_breadth_requirements(&concs, &bb_pool, &bb_labels, false);

    let (mut requirements, mut schedule_hints) = scheduled(vec![
             // First-Year Foundations
            (Y1F, Requirement::AnyOf { category: Some("First-Year Foundations - Econ".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["BEPP 1000".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0200".to_string()] },
                ] }
            ] }),
            (Y1F, Requirement::SingleCourse { category: Some("First-Year Foundations - Math".to_string()), possibilities: vec!["MATH 1400".to_string(), "MATH 1070".to_string()] }),
            (Y1F, Requirement::Restriction { category: Some("First-Year Foundations - Writing Sem".to_string()), department: Some(vec!["WRIT".to_string()]), cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None }),

            // Leadership Journey
            (Y1S, Requirement::SingleCourse { category: Some("Leadership Journey - WH 1010".to_string()), possibilities: vec!["WH 1010".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Leadership Journey - WH 2010/2011".to_string()), possibilities: vec!["WH 2010".to_string(), "WH 2011".to_string()] }),
            (Y3F, Requirement::SingleCourse { category: Some("Leadership Journey - MGMT 3010".to_string()), possibilities: vec!["MGMT 3010".to_string()] }),
            (Y3S, Requirement::Restriction { category: Some("Undergraduate Capstone".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCP".to_string()]), excluding: None, number: 1, no_school: None }),

            // Fundamentals
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1010".to_string()] }),
            (Y2S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1020".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["BEPP 2500".to_string(), "BEPP 2508".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1000".to_string(), "FNCE 1008".to_string()] }),
            (Y2S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1010".to_string(), "FNCE 1018".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["LGST 1000".to_string(), "LGST 1010".to_string(), "LGST 1008".to_string(), "LGST 1018".to_string()] }),
            (Y1F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MGMT 1010".to_string(), "MKTG 1018".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MKTG 1010".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["OIDD 1010".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 1010".to_string(), "STAT 4300".to_string(), "ESE 3010".to_string(), "STAT 1018".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 1020".to_string(), "STAT 4310".to_string(), "ESE 4020".to_string(), "STAT 1028".to_string()] }),

            // Flex Fundamentals
            (Y2F, Requirement::Restriction { category: Some("Flex Fundamentals".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUGE".to_string()]), excluding: None, number: 1, no_school: None }),
            (Y2S, Requirement::Restriction { category: Some("Flex Fundamentals".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUTI".to_string()]), excluding: None, number: 1, no_school: None }),
    ]);

    append_semester(&mut requirements, &mut schedule_hints, Y3F, bb_reqs);
    append_semester(&mut requirements, &mut schedule_hints, Y2S, vec![
            wh_ssh_las_pool(),
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Unrestricted Electives".to_string()), department: None, cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None },
    ]);
    append_semester(&mut requirements, &mut schedule_hints, Y3F, wh_concentration_requirements(&concs));

    return Major {
        short_name: "WH".to_string(), 
        name: "Wharton Undergraduate".to_string(), 
        requirements,
        schedule_hints,
        concentrations: Some(wh_concentrations),
    }
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
    let mgmt_is_conc = concs.iter().any(|c| c == "MGMT");

    let extra_bb_labels = mt_business_breadth_labels(&concs);
    let extra_bb = business_breadth_requirements(&concs, &bb_pool, &extra_bb_labels, true);

    let conc_reqs = if mgmt_is_conc {
        wh_concentration_requirements_skip_mgmt_first(&concs)
    } else {
        wh_concentration_requirements(&concs)
    };

    let (mut requirements, mut schedule_hints) = scheduled(vec![
             // First-Year Foundations
            (Y1F, Requirement::AnyOf { category: Some("First-Year Foundations - Econ".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["BEPP 1000".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0200".to_string()] },
                ] }
            ] }),
            (Y1F, Requirement::SingleCourse { category: Some("First-Year Foundations - Math".to_string()), possibilities: vec!["MATH 1400".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("First-Year Foundations - Math".to_string()), possibilities: vec!["MATH 1410".to_string()] }),
            (Y1S, Requirement::Restriction { category: Some("First-Year Foundations - Writing Sem".to_string()), department: Some(vec!["WRIT".to_string()]), cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None }),

            // Leadership Journey
            (Y1F, Requirement::SingleCourse { category: Some("Leadership Journey - WH 1010".to_string()), possibilities: vec!["WH 1010".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Leadership Journey - WH 2010/2011".to_string()), possibilities: vec!["WH 2010".to_string(), "WH 2011".to_string()] }),
            (Y3F, Requirement::SingleCourse { category: Some("Leadership Journey - MGMT 3010".to_string()), possibilities: vec!["MGMT 3010".to_string()] }),

            // Fundamentals
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1010".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1020".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["BEPP 2500".to_string(), "BEPP 2508".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1000".to_string(), "FNCE 1008".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1010".to_string(), "FNCE 1018".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MGMT 1010".to_string(), "MKTG 1018".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MKTG 1010".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 4300".to_string(), "ESE 3010".to_string(), "STAT 1018".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 4310".to_string(), "ESE 4020".to_string(), "STAT 1028".to_string()] }),

            // Flex Fundamentals
            (Y2F, Requirement::Restriction { category: Some("Flex Fundamentals".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUGE".to_string()]), excluding: None, number: 1, no_school: None }),
            
            // M&T Soph (MGMT 2370); one fewer BB or conc course below accounts for overlap
            (Y2S, mt_mgmt2370_soph()),
            (Y1F, Requirement::SingleCourse { category: Some("M&T Freshman Course".to_string()), possibilities: vec!["OIDD 2340".to_string()] }),
    ]);

    append_semester(&mut requirements, &mut schedule_hints, Y3F, extra_bb);
    // NOFL M&T LAS: four standalone 1-CU requirements (no CoursePool).
    append_semester(&mut requirements, &mut schedule_hints, Y2S, vec![
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Humanities and Social Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUHM".to_string(), "WUSS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Humanities and Social Science".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUHM".to_string(), "WUSS".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Cross Cultural".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string()]), excluding: None, number: 1, no_school: None },
            Requirement::Restriction { category: Some("Liberal Arts and Sciences - Cross Cultural".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUCN".to_string(), "WUCU".to_string()]), excluding: None, number: 1, no_school: None },
    ]);
    append_semester(&mut requirements, &mut schedule_hints, Y3F, conc_reqs);

    return Major {
        short_name: "WH_NOFL_MT".to_string(), 
        name: "M&T - Foreign Language Exempt".to_string(), 
        requirements,
        schedule_hints,
        concentrations: Some(wh_concentrations),
    }
}

pub fn create_wh_fl_mt_major(concentrations: Vec<String>) -> Major {
    let concentrations = normalize_wh_concentrations(&concentrations);
    let concs = if concentrations.is_empty() {
        vec!["FNCE".to_string()]
    } else {
        concentrations
    };
    let wh_concentrations = create_wh_concentrations();
    let bb_pool = ["FNCE", "ACCT", "BEPP", "MGMT", "MKTG", "HCMG", "REAL", "OIDD", "STAT", "LGST"];
    let mgmt_is_conc = concs.iter().any(|c| c == "MGMT");

    let extra_bb_labels = mt_business_breadth_labels(&concs);
    let extra_bb = business_breadth_requirements(&concs, &bb_pool, &extra_bb_labels, true);

    let conc_reqs = if mgmt_is_conc {
        wh_concentration_requirements_skip_mgmt_first(&concs)
    } else {
        wh_concentration_requirements(&concs)
    };

    let (mut requirements, mut schedule_hints) = scheduled(vec![
            (Y1F, Requirement::AnyOf { category: Some("First-Year Foundations - Econ".to_string()), possibilities: vec![
                Requirement::SingleCourse { category: None, possibilities: vec!["BEPP 1000".to_string()] },
                Requirement::AllOf { category: None, requirements: vec![
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0100".to_string()] },
                    Requirement::SingleCourse { category: None, possibilities: vec!["ECON 0200".to_string()] },
                ] }
            ] }),
            (Y1F, Requirement::SingleCourse { category: Some("First-Year Foundations - Math".to_string()), possibilities: vec!["MATH 1400".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("First-Year Foundations - Math".to_string()), possibilities: vec!["MATH 1410".to_string()] }),
            (Y1F, Requirement::Restriction { category: Some("First-Year Foundations - Writing Sem".to_string()), department: Some(vec!["WRIT".to_string()]), cu: None, level: None, attr: None, excluding: None, number: 1, no_school: None }),

            (Y1F, Requirement::SingleCourse { category: Some("Leadership Journey - WH 1010".to_string()), possibilities: vec!["WH 1010".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Leadership Journey - WH 2010/2011".to_string()), possibilities: vec!["WH 2010".to_string(), "WH 2011".to_string()] }),
            (Y3F, Requirement::SingleCourse { category: Some("Leadership Journey - MGMT 3010".to_string()), possibilities: vec!["MGMT 3010".to_string()] }),

            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1010".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["ACCT 1020".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["BEPP 2500".to_string(), "BEPP 2508".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1000".to_string(), "FNCE 1008".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["FNCE 1010".to_string(), "FNCE 1018".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MGMT 1010".to_string(), "MKTG 1018".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["MKTG 1010".to_string()] }),
            (Y1S, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 4300".to_string(), "ESE 3010".to_string(), "STAT 1018".to_string()] }),
            (Y2F, Requirement::SingleCourse { category: Some("Fundamentals".to_string()), possibilities: vec!["STAT 4310".to_string(), "ESE 4020".to_string(), "STAT 1028".to_string()] }),

            (Y2F, Requirement::Restriction { category: Some("Flex Fundamentals".to_string()), department: None, cu: None, level: None, attr: Some(vec!["WUGE".to_string()]), excluding: None, number: 1, no_school: None }),

            (Y2S, mt_mgmt2370_soph()),
            (Y1F, Requirement::SingleCourse { category: Some("M&T Freshman Course".to_string()), possibilities: vec!["OIDD 2340".to_string()] }),
    ]);

    append_semester(&mut requirements, &mut schedule_hints, Y3F, extra_bb);
    append_semester(&mut requirements, &mut schedule_hints, Y2S, vec![
            wh_fl_mt_las_pool(),
    ]);
    append_semester(&mut requirements, &mut schedule_hints, Y3F, conc_reqs);

    Major {
        short_name: "WH".to_string(),
        name: "M&T - Foreign Language Required".to_string(),
        requirements,
        schedule_hints,
        concentrations: Some(wh_concentrations),
    }
}

#[cfg(test)]
mod tests {
    use super::{create_wh_fl_mt_major, create_wh_nofl_mt_major, Requirement};
    use crate::major::resolve_major;
    use crate::attributes_data;
    use crate::courses_data;
    use crate::requirement::{evaluate_pool_constraints, extract_concentration_info, validate_courses_for_degree};
    use std::collections::HashMap;

    #[test]
    fn nofl_mt_concentration_tracker_excludes_flex_fundamental_course() {
        let major = resolve_major("WH", "WH_NOFL_MT", &["FNCE".to_string()])
            .expect("WH_NOFL_MT major");
        let cu_map: HashMap<String, f64> = courses_data::all_courses()
            .iter()
            .map(|c| (c.course_code.clone(), c.cu))
            .collect();
        let taken = vec![
            "FNCE 2310".to_string(),
            "FNCE 2030".to_string(),
            "FNCE 2050".to_string(),
            "FNCE 2070".to_string(),
        ];

        let validation =
            validate_courses_for_degree(major.requirements.clone(), &taken, &cu_map);
        let conc_in_validation: Vec<_> = validation
            .fulfilled
            .iter()
            .filter(|m| m.requirement.get_category() == "Concentration - FNCE")
            .flat_map(|m| m.course_ids.clone())
            .collect();
        assert!(
            !conc_in_validation.contains(&"FNCE 2310".to_string()),
            "Flex Fundamental course should not fill a concentration slot"
        );
        assert_eq!(conc_in_validation.len(), 3);

        let conc_info = extract_concentration_info(
            &major.requirements,
            &major.concentrations,
            &["FNCE".to_string()],
            &taken,
            &cu_map,
            Some(&validation),
        );
        assert_eq!(conc_info.len(), 1);
        assert_eq!(conc_info[0].requirements_fulfilled, 3);
        assert!(
            !conc_info[0]
                .matched_courses
                .iter()
                .flatten()
                .any(|c| c == "FNCE 2310"),
            "Concentration tracker should match requirements panel"
        );
    }

    #[test]
    fn nofl_mt_has_no_course_pool() {
        let major = create_wh_nofl_mt_major(vec!["STAT".to_string()]);
        assert!(
            !major
                .requirements
                .iter()
                .any(|r| matches!(r, Requirement::CoursePool { .. })),
            "NOFL M&T should use standalone LAS requirements only"
        );
    }

    #[test]
    fn fl_mt_pool_blocks_cc_ssh_overlap_without_fl() {
        let major = create_wh_fl_mt_major(vec!["FNCE".to_string()]);
        let pool_req = major
            .requirements
            .iter()
            .find(|r| matches!(r, Requirement::CoursePool { .. }))
            .expect("FL M&T LAS pool");
        let Requirement::CoursePool { constraints, .. } = pool_req else {
            panic!("expected CoursePool");
        };

        let mut attributes = attributes_data::create_attributes();
        for attr in ["WUCN", "WUHM"] {
            attributes
                .entry(attr.to_string())
                .or_default()
                .push("ANTH 0001".to_string());
        }
        let cu_map = HashMap::from([("ANTH 0001".to_string(), 1.0)]);
        let pool = vec!["ANTH 0001".to_string()];

        let evaluations = evaluate_pool_constraints(&pool, constraints, &attributes, &cu_map);
        let mt_las_fulfilled = evaluations
            .iter()
            .filter(|e| e.consumption_group == "wh:mt_las" && e.fulfilled)
            .count();
        assert_eq!(
            mt_las_fulfilled, 1,
            "CC and SSH share wh:mt_las — one course covers at most one slot"
        );
    }

    #[test]
    fn fl_mt_pool_allows_wufl_to_double_with_mt_las() {
        let major = create_wh_fl_mt_major(vec!["FNCE".to_string()]);
        let pool_req = major
            .requirements
            .iter()
            .find(|r| matches!(r, Requirement::CoursePool { .. }))
            .expect("FL M&T LAS pool");
        let Requirement::CoursePool { constraints, .. } = pool_req else {
            panic!("expected CoursePool");
        };

        let mut attributes = attributes_data::create_attributes();
        for attr in ["WUFL", "WUHM"] {
            attributes
                .entry(attr.to_string())
                .or_default()
                .push("SPAN 0100".to_string());
        }
        let cu_map = HashMap::from([("SPAN 0100".to_string(), 1.0)]);
        let pool = vec!["SPAN 0100".to_string()];

        let evaluations = evaluate_pool_constraints(&pool, constraints, &attributes, &cu_map);
        assert!(
            evaluations
                .iter()
                .any(|e| e.consumption_group == "wh:mt_las" && e.fulfilled),
            "WUHM/WUSS slot"
        );
        assert!(
            evaluations
                .iter()
                .any(|e| e.consumption_group == "wh:wufl" && e.fulfilled),
            "WUFL may double-count with mt_las"
        );
    }

    #[test]
    fn mt_double_conc_non_mgmt_has_no_business_breadth() {
        for major in [
            create_wh_nofl_mt_major(vec!["FNCE".to_string(), "STAT".to_string()]),
            create_wh_fl_mt_major(vec!["FNCE".to_string(), "STAT".to_string()]),
        ] {
            let bb_count = major
                .requirements
                .iter()
                .filter(|r| {
                    r.get_category()
                        .to_lowercase()
                        .contains("business breadth")
                })
                .count();
            assert_eq!(
                bb_count, 0,
                "{}: MGMT 2370 + conc overlap cover both breadths",
                major.name
            );
        }
    }

    #[test]
    fn mt_double_conc_mgmt_has_one_business_breadth() {
        let major = create_wh_nofl_mt_major(vec!["MGMT".to_string(), "FNCE".to_string()]);
        let bb: Vec<_> = major
            .requirements
            .iter()
            .filter(|r| {
                r.get_category()
                    .to_lowercase()
                    .contains("business breadth")
            })
            .collect();
        assert_eq!(bb.len(), 1);
        assert!(bb[0].get_category().contains("Business Breadth - I"));
    }

    #[test]
    fn mt_single_stat_includes_business_breadth_ii() {
        let major = create_wh_nofl_mt_major(vec!["STAT".to_string()]);
        let bb: Vec<_> = major
            .requirements
            .iter()
            .filter(|r| {
                r.get_category()
                    .to_lowercase()
                    .contains("business breadth")
            })
            .collect();
        assert_eq!(bb.len(), 1);
        assert!(bb[0].get_category().contains("Business Breadth - II"));
    }

    #[test]
    fn mt_single_stat_surfaces_business_breadth_in_validation() {
        let major = create_wh_nofl_mt_major(vec!["STAT".to_string()]);
        let cu_map: HashMap<String, f64> = courses_data::all_courses()
            .iter()
            .map(|c| (c.course_code.clone(), c.cu))
            .collect();
        let taken: Vec<String> = vec![];
        let validation =
            validate_courses_for_degree(major.requirements.clone(), &taken, &cu_map);
        let fulfilled = validation.fulfilled;
        let unfulfilled = validation.unfulfilled;
        let bb_unfulfilled: Vec<_> = unfulfilled
            .iter()
            .chain(fulfilled.iter())
            .filter(|m| {
                m.requirement
                    .get_category()
                    .to_lowercase()
                    .contains("business breadth")
            })
            .collect();
        assert_eq!(
            bb_unfulfilled.len(),
            1,
            "categories: {:?}",
            unfulfilled
                .iter()
                .map(|m| m.requirement.get_category())
                .collect::<Vec<_>>()
        );
    }
}
