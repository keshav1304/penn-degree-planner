use std::collections::{BTreeMap, HashMap};

use crate::Major;
use crate::Requirement;
use crate::penn_data::requirement_builders::{
    all_of, any_of, attr_pool_constraint, attrs_pool_constraint, code, course_group,
    course_pool, no_school_pool_constraint, repeat_req, restriction, single,
    unrestricted_elective,
};
use crate::schedule_template::{
    append_semester, insert_fixed_course_hints, scheduled, ScheduleHint, Y1F, Y1S, Y2F, Y2S,
    Y3F, Y3S,
};

/// WH 1010 is mandatory Y1 Fall for every Wharton template; M&T adds OIDD 2340 (Y1 Fall)
/// and MGMT 2370 (Y2 Spring).
fn apply_wh_fixed_hints(hints: &mut HashMap<String, ScheduleHint>, mt: bool) {
    let mut fixed = vec![("WH 1010", Y1F)];
    if mt {
        fixed.extend([("OIDD 2340", Y1F), ("MGMT 2370", Y2S)]);
    }
    insert_fixed_course_hints(hints, &fixed);
}

/// WH_FL: 7 LAS courses, 11 coverage units. Double-count policy via consumption groups:
/// - `wh:cc_fl`: FL + CC slots are mutually exclusive per course
/// - `wh:ssh`: WUHM / WUSS / WUNM mutually exclusive per course (CC may overlap)
/// - `wh:non_wh`: non-Wharton slots (CC and FL may overlap)
fn wh_fl_las_pool() -> Requirement {
    course_pool(
        "Liberal Arts and Sciences",
        vec![],
        7,
        vec![
            attr_pool_constraint("Humanities (WUHM)", "WUHM", 1, "wh:ssh"),
            attr_pool_constraint("Natural Science & Math (WUNM)", "WUNM", 1, "wh:ssh"),
            attr_pool_constraint("Social Science (WUSS)", "WUSS", 1, "wh:ssh"),
            no_school_pool_constraint("Non-Wharton course", "WH", 3, "wh:non_wh"),
            attr_pool_constraint("Foreign Language (WUFL)", "WUFL", 2, "wh:cc_fl"),
            attr_pool_constraint("Cross-Cultural (WUCN)", "WUCN", 2, "wh:cc_fl"),
            attrs_pool_constraint(
                "Cross-Cultural (WUCN/WUCU)",
                &["WUCN", "WUCU"],
                1,
                "wh:cc_fl",
            ),
        ],
    )
}

/// WH_NOFL SSH: CC may double-count into SSH and non-Wharton; SSH tags are mutually exclusive.
fn wh_ssh_las_pool() -> Requirement {
    course_pool(
        "Liberal Arts and Sciences",
        vec![],
        6,
        vec![
            attrs_pool_constraint("Humanities (WUHM)", &["WUHM"], 1, "wh:ssh"),
            attr_pool_constraint("Natural Science & Math (WUNM)", "WUNM", 1, "wh:ssh"),
            attr_pool_constraint("Social Science (WUSS)", "WUSS", 1, "wh:ssh"),
            no_school_pool_constraint("Non-Wharton course", "WH", 3, "wh:non_wh"),
            attr_pool_constraint("Cross-Cultural (WUCN)", "WUCN", 2, "wh:cross_cultural"),
            attrs_pool_constraint(
                "Cross-Cultural (WUCN/WUCU)",
                &["WUCN", "WUCU"],
                1,
                "wh:cross_cultural",
            ),
        ],
    )
}

/// M&T FL-required LAS: 4 courses, 6 coverage units. The four non-FL requirements
/// share `wh:mt_las` (no cross-double-count among them); WUFL uses `wh:wufl` and
/// may double-count with any `wh:mt_las` slot.
fn wh_fl_mt_las_pool() -> Requirement {
    course_pool(
        "Liberal Arts and Sciences",
        vec![],
        4,
        vec![
            attrs_pool_constraint(
                "Humanities / Social Science (WUHM/WUSS)",
                &["WUHM", "WUSS"],
                1,
                "wh:mt_las",
            ),
            attrs_pool_constraint(
                "Humanities / Social Science (WUHM/WUSS)",
                &["WUHM", "WUSS"],
                1,
                "wh:mt_las",
            ),
            attr_pool_constraint("Cross-Cultural (WUCN)", "WUCN", 1, "wh:mt_las"),
            attrs_pool_constraint(
                "Cross-Cultural (WUCN/WUCU)",
                &["WUCN", "WUCU"],
                1,
                "wh:mt_las",
            ),
            attr_pool_constraint("Foreign Language (WUFL)", "WUFL", 1, "wh:wufl"),
            attr_pool_constraint("Foreign Language (WUFL)", "WUFL", 1, "wh:wufl"),
        ],
    )
}

pub fn concentration_names() -> Vec<String> {
    let mut names: Vec<String> = create_wh_concentrations().keys().cloned().collect();
    names.sort();
    names
}

/// Resolve a catalog key from the key itself or legacy UI labels.
pub fn resolve_wh_concentration_key(input: &str) -> Option<String> {
    let catalog = create_wh_concentrations();
    if catalog.contains_key(input) {
        return Some(input.to_string());
    }
    let legacy = match input {
        "Marketing & Operations Management" => Some("MAOM"),
        "Accounting" => Some("ACCT"),
        "Business Economics and Public Policy" => Some("BEPP"),
        "Business Analytics" => Some("BUAN"),
        "Finance" => Some("FNCE"),
        "Management" => Some("MGMT"),
        "Marketing" => Some("MKTG"),
        "Statistics and Data Science" => Some("STAT"),
        "Health Care Management" => Some("HCMG"),
        "OIDD: Decision Processes" => Some("ODDP"),
        "OIDD: General" => Some("ODGN"),
        "OIDD: Information Systems" => Some("ODIS"),
        "OIDD: Operations Management" => Some("ODOM"),
        _ => None,
    };
    legacy
        .filter(|k| catalog.contains_key(*k))
        .map(|k| k.to_string())
}

fn wh_conc_dept_slots(category: &str, dept: &str, exclude: &[&str], count: usize) -> Vec<Requirement> {
    let slot: Requirement = restriction(1)
        .category(category)
        .departments(&[dept])
        .excluding(exclude)
        .into();
    repeat_req(&slot, count)
}

fn wh_marketing_operations_management_requirements() -> Vec<Requirement> {
    let category = "Concentration - MAOM";
    vec![
        single(category, &["OIDD 2200"]),
        any_of(
            category,
            vec![code(&["OIDD 2360"]), code(&["OIDD 3140"]), code(&["OIDD 4110"]), code(&["OIDD 4150"]), code(&["OIDD 6590"])],
        ),
        course_group(
            category,
            2,
            vec![
                code(&["MKTG 2250"]),
                code(&["MKTG 2270"]),
                code(&["MKTG 2340"]),
                code(&["MKTG 2470"]),
                code(&["MKTG 2540"]),
                code(&["MKTG 2680"]),
                code(&["MKTG 2770"]),
                code(&["MKTG 2790"]),
                code(&["MKTG 2880"]),
                code(&["MKTG 4760"]),
                code(&["MKTG 4710"]),
            ],
        ),
    ]
}

fn wh_hcmg_concentration_requirements() -> Vec<Requirement> {
    let category = "Concentration - HCMG";
    let elective: Requirement = restriction(1)
        .category(category)
        .departments(&["HCMG"])
        .level(2000)
        .max_level(4000)
        .excluding(&["HCMG 1010"])
        .into();
    let mut reqs = vec![single(category, &["HCMG 1010"])];
    reqs.extend(repeat_req(&elective, 3));
    reqs
}

fn wh_oidd_decision_processes_requirements() -> Vec<Requirement> {
    let category = "Concentration - ODDP";
    let electives = &[
        "OIDD 2000",
        "OIDD 2210",
        "OIDD 2610",
        "OIDD 2920",
        "OIDD 2990",
        "OIDD 3190",
        "OIDD 4690",
        "BEPP 2840",
        "MGMT 2380",
        "MGMT 2950",
        "FNCE 2390",
        "MKTG 2110",
        "MKTG 2370",
        "MKTG 2380",
        "PSYC 2737",
        "PSYC 2750",
    ];
    vec![
        single(category, &["OIDD 2900"]),
        single(category, &["OIDD 2910"]),
        single(category, electives),
        single(category, electives),
    ]
}

fn wh_oidd_general_requirements() -> Vec<Requirement> {
    vec![restriction(4)
        .category("Concentration - ODGN")
        .attr(&["WUOD"])
        .into()]
}

fn wh_oidd_information_systems_requirements() -> Vec<Requirement> {
    let category = "Concentration - ODIS";
    let electives = &[
        "OIDD 1050",
        "OIDD 2550",
        "OIDD 2900",
        "OIDD 3140",
        "OIDD 3150",
        "OIDD 3190",
        "OIDD 4690",
    ];
    vec![
        single(category, electives),
        single(category, electives),
        single(category, electives),
        single(category, electives),
    ]
}

fn wh_oidd_operations_management_requirements() -> Vec<Requirement> {
    let category = "Concentration - ODOM";
    let electives = &[
        "OIDD 2200",
        "OIDD 2210",
        "OIDD 2360",
        "OIDD 3800",
        "OIDD 4150",
        "OIDD 6970",
    ];
    vec![
        single(category, &["OIDD 2200", "OIDD 2210"]),
        single(category, electives),
        single(category, electives),
        single(category, electives),
    ]
}

pub fn create_wh_concentrations() -> BTreeMap<String, Vec<Requirement>> {
    BTreeMap::from([
        (
            "FNCE".to_string(),
            wh_conc_dept_slots("Concentration - FNCE", "FNCE", &["FNCE 1010", "FNCE 1000"], 4),
        ),
        (
            "STAT".to_string(),
            wh_conc_dept_slots(
                "Concentration - STAT",
                "STAT",
                &["STAT 1010", "STAT 1020", "STAT 4300", "STAT 4310"],
                4,
            ),
        ),
        (
            "MKTG".to_string(),
            wh_conc_dept_slots("Concentration - MKTG", "MKTG", &["MKTG 1010"], 4),
        ),
        (
            "MGMT".to_string(),
            wh_conc_dept_slots("Concentration - MGMT", "MGMT", &["MGMT 1010"], 4),
        ),
        (
            "ACCT".to_string(),
            wh_conc_dept_slots("Concentration - ACCT", "ACCT", &["ACCT 1010"], 4),
        ),
        (
            "BEPP".to_string(),
            wh_conc_dept_slots("Concentration - BEPP", "BEPP", &["BEPP 1010"], 4),
        ),
        (
            "BUAN".to_string(),
            vec![
                restriction(1)
                    .category("Concentration - BUAN")
                    .attr(&["WUBD"])
                    .into(),
                restriction(1)
                    .category("Concentration - BUAN")
                    .attr(&["WUBC"])
                    .into(),
                restriction(1)
                    .category("Concentration - BUAN")
                    .attr(&["WUBO"])
                    .into(),
                restriction(1)
                    .category("Concentration - BUAN")
                    .attr(&["WUBD", "WUBC", "WUBO", "WUBN"])
                    .into(),
            ],
        ),
        (
            "MAOM".to_string(),
            wh_marketing_operations_management_requirements(),
        ),
        (
            "HCMG".to_string(),
            wh_hcmg_concentration_requirements(),
        ),
        (
            "ODDP".to_string(),
            wh_oidd_decision_processes_requirements(),
        ),
        (
            "ODGN".to_string(),
            wh_oidd_general_requirements(),
        ),
        (
            "ODIS".to_string(),
            wh_oidd_information_systems_requirements(),
        ),
        (
            "ODOM".to_string(),
            wh_oidd_operations_management_requirements(),
        ),
    ])
}

/// Up to two distinct Wharton concentrations; unknown names are dropped.
pub fn normalize_wh_concentrations(concentrations: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for c in concentrations {
        let Some(key) = resolve_wh_concentration_key(c) else {
            continue;
        };
        if !out.contains(&key) {
            out.push(key);
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

fn bb_excluded_departments(concentrations: &[String]) -> Vec<String> {
    let mut excluded = Vec::new();
    for concentration in concentrations {
        let key = resolve_wh_concentration_key(concentration)
            .unwrap_or_else(|| concentration.to_string());
        match key.as_str() {
            "MAOM" => {
                excluded.push("OIDD".to_string());
                excluded.push("MKTG".to_string());
            }
            "ODDP" | "ODGN" | "ODIS" | "ODOM" => {
                excluded.push("OIDD".to_string());
            }
            dept => excluded.push(dept.to_string()),
        }
    }
    excluded
}

fn bb_department_options(
    concentrations: &[String],
    pool: &[&str],
    exclusions: &[&str],
) -> Vec<Requirement> {
    let mut depts: Vec<String> = pool.iter().map(|s| s.to_string()).collect();
    if concentrations.len() < 2 {
        for dept in bb_excluded_departments(concentrations) {
            depts.retain(|d| d != &dept);
        }
    }
    depts
        .into_iter()
        .map(|dept| {
            restriction(1)
                .departments(&[&dept])
                .excluding(exclusions)
                .into()
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

/// M&T needs two business breadths. MGMT 2370 counts as one breadth when MGMT is not a
/// concentration; with two non-MGMT concentrations, a concentration course covers the
/// second breadth as well — no standalone breadth slots remain.
fn mt_business_breadth_labels(concentrations: &[String]) -> Vec<String> {
    let mgmt_is_conc = concentrations.iter().any(|c| c == "MGMT");
    let double_conc = concentrations.len() >= 2;

    if double_conc && !mgmt_is_conc {
        return vec![];
    }

    let default_labels: Vec<&str> = if mgmt_is_conc {
        vec!["Business Breadth", "Business Breadth"]
    } else {
        vec!["Business Breadth"]
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
    let ex_refs: Vec<&str> = exclusions.iter().map(|s| s.as_str()).collect();
    let opts = bb_department_options(concentrations, pool, &ex_refs);
    slot_labels
        .iter()
        .map(|label| any_of(label, opts.clone()))
        .collect()
}

fn mt_mgmt2370_soph() -> Requirement {
    single("M&T Soph Course", &["MGMT 2370"])
}

fn wh_unrestricted_electives(count: usize) -> Vec<Requirement> {
    repeat_req(&unrestricted_elective("Unrestricted Electives"), count)
}

fn wh_first_year_foundations() -> Requirement {
    any_of(
        "First-Year Foundations",
        vec![
            code(&["BEPP 1000"]),
            all_of(
                None,
                vec![code(&["ECON 0100"]), code(&["ECON 0200"])],
            ),
        ],
    )
}

fn wh_leadership_journey_fl() -> [(crate::schedule_template::Semester, Requirement); 4] {
    [
        (Y1F, single("Leadership Journey", &["WH 1010"])),
        (Y2F, single("Leadership Journey", &["WH 2010", "WH 2011"])),
        (Y3F, single("Leadership Journey", &["MGMT 3010"])),
        (
            Y3S,
            restriction(1)
                .category("Leadership Journey")
                .attr(&["WUCP"])
                .into(),
        ),
    ]
}

fn wh_fundamentals_fl() -> Vec<(crate::schedule_template::Semester, Requirement)> {
    vec![
        (Y2F, single("Fundamentals", &["ACCT 1010"])),
        (Y2S, single("Fundamentals", &["ACCT 1020"])),
        (Y1S, single("Fundamentals", &["BEPP 2500", "BEPP 2508"])),
        (Y2F, single("Fundamentals", &["FNCE 1000", "FNCE 1008"])),
        (Y2S, single("Fundamentals", &["FNCE 1010", "FNCE 1018"])),
        (
            Y1F,
            single(
                "Fundamentals",
                &["LGST 1000", "LGST 1010", "LGST 1008", "LGST 1018"],
            ),
        ),
        (Y1S, single("Fundamentals", &["MGMT 1010", "MKTG 1018"])),
        (Y1S, single("Fundamentals", &["MKTG 1010"])),
        (Y1S, single("Fundamentals", &["OIDD 1010"])),
        (
            Y1S,
            single(
                "Fundamentals",
                &["STAT 1010", "STAT 4300", "ESE 3010", "STAT 1018"],
            ),
        ),
        (
            Y2F,
            single(
                "Fundamentals",
                &["STAT 1020", "STAT 4310", "ESE 4020", "STAT 1028"],
            ),
        ),
        (
            Y3F,
            restriction(1).category("Flex Fundamentals").attr(&["WUGE"]).into(),
        ),
        (
            Y3S,
            restriction(1).category("Flex Fundamentals").attr(&["WUTI"]).into(),
        ),
    ]
}

fn wh_leadership_journey_nofl() -> [(crate::schedule_template::Semester, Requirement); 4] {
    [
        (Y1F, single("Leadership Journey", &["WH 1010"])),
        (Y2F, single("Leadership Journey", &["WH 2010", "WH 2011"])),
        (Y3F, single("Leadership Journey", &["MGMT 3010"])),
        (
            Y3S,
            restriction(1)
                .category("Undergraduate Capstone")
                .attr(&["WUCP"])
                .into(),
        ),
    ]
}

fn wh_leadership_journey_mt() -> [(crate::schedule_template::Semester, Requirement); 3] {
    [
        (Y1F, single("Leadership Journey", &["WH 1010"])),
        (Y2F, single("Leadership Journey", &["WH 2010", "WH 2011"])),
        (Y3F, single("Leadership Journey", &["MGMT 3010"])),
    ]
}

fn wh_fundamentals_nofl() -> Vec<(crate::schedule_template::Semester, Requirement)> {
    vec![
        (Y2F, single("Fundamentals", &["ACCT 1010"])),
        (Y2S, single("Fundamentals", &["ACCT 1020"])),
        (Y1S, single("Fundamentals", &["BEPP 2500", "BEPP 2508"])),
        (Y2F, single("Fundamentals", &["FNCE 1000", "FNCE 1008"])),
        (Y2S, single("Fundamentals", &["FNCE 1010", "FNCE 1018"])),
        (
            Y1S,
            single(
                "Fundamentals",
                &["LGST 1000", "LGST 1010", "LGST 1008", "LGST 1018"],
            ),
        ),
        (Y1F, single("Fundamentals", &["MGMT 1010", "MKTG 1018"])),
        (Y1S, single("Fundamentals", &["MKTG 1010"])),
        (Y1S, single("Fundamentals", &["OIDD 1010"])),
        (
            Y1S,
            single(
                "Fundamentals",
                &["STAT 1010", "STAT 4300", "ESE 3010", "STAT 1018"],
            ),
        ),
        (
            Y2F,
            single(
                "Fundamentals",
                &["STAT 1020", "STAT 4310", "ESE 4020", "STAT 1028"],
            ),
        ),
        (
            Y2F,
            restriction(1).category("Flex Fundamentals").attr(&["WUGE"]).into(),
        ),
        (
            Y2S,
            restriction(1).category("Flex Fundamentals").attr(&["WUTI"]).into(),
        ),
    ]
}

fn wh_fundamentals_mt() -> Vec<(crate::schedule_template::Semester, Requirement)> {
    vec![
        (Y1S, single("Fundamentals", &["ACCT 1010"])),
        (Y2F, single("Fundamentals", &["ACCT 1020"])),
        (Y1S, single("Fundamentals", &["BEPP 2500", "BEPP 2508"])),
        (Y2F, single("Fundamentals", &["FNCE 1000", "FNCE 1008"])),
        (Y2F, single("Fundamentals", &["FNCE 1010", "FNCE 1018"])),
        (Y1S, single("Fundamentals", &["MGMT 1010", "MKTG 1018"])),
        (Y1S, single("Fundamentals", &["MKTG 1010"])),
        (
            Y1S,
            single(
                "Fundamentals",
                &["STAT 4300", "ESE 3010", "STAT 1018"],
            ),
        ),
        (
            Y2F,
            single(
                "Fundamentals",
                &["STAT 4310", "ESE 4020", "STAT 1028"],
            ),
        ),
        (
            Y2F,
            restriction(1).category("Flex Fundamentals").attr(&["WUGE"]).into(),
        ),
    ]
}

fn wh_nofl_mt_las_standalone() -> Vec<Requirement> {
    vec![
        restriction(1)
            .category("Liberal Arts and Sciences - Humanities and Social Science")
            .attr(&["WUHM", "WUSS"])
            .into(),
        restriction(1)
            .category("Liberal Arts and Sciences - Humanities and Social Science")
            .attr(&["WUHM", "WUSS"])
            .into(),
        restriction(1)
            .category("Liberal Arts and Sciences - Cross Cultural")
            .attr(&["WUCN"])
            .into(),
        restriction(1)
            .category("Liberal Arts and Sciences - Cross Cultural")
            .attr(&["WUCN", "WUCU"])
            .into(),
    ]
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

    let (mut requirements, mut schedule_hints) = scheduled({
        let mut entries = vec![
            (Y1F, wh_first_year_foundations()),
            (
                Y1F,
                single("First-Year Foundations", &["MATH 1400", "MATH 1070"]),
            ),
            (
                Y1F,
                restriction(1)
                    .category("First-Year Foundations")
                    .departments(&["WRIT"])
                    .into(),
            ),
        ];
        entries.extend(wh_leadership_journey_fl());
        entries.extend(wh_fundamentals_fl());
        entries
    });

    append_semester(&mut requirements, &mut schedule_hints, Y3F, bb_reqs);
    append_semester(
        &mut requirements,
        &mut schedule_hints,
        Y2S,
        std::iter::once(wh_fl_las_pool())
            .chain(wh_unrestricted_electives(5))
            .collect(),
    );
    append_semester(&mut requirements, &mut schedule_hints, Y3F, wh_concentration_requirements(&concs));

    apply_wh_fixed_hints(&mut schedule_hints, false);

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
        &["Business Breadth", "Business Breadth", "Business Breadth"],
        &concs,
    );
    let bb_reqs = business_breadth_requirements(&concs, &bb_pool, &bb_labels, false);

    let (mut requirements, mut schedule_hints) = scheduled({
        let mut entries = vec![
            (Y1F, wh_first_year_foundations()),
            (
                Y1F,
                single("First-Year Foundations", &["MATH 1400", "MATH 1070"]),
            ),
            (
                Y1F,
                restriction(1)
                    .category("First-Year Foundations")
                    .departments(&["WRIT"])
                    .into(),
            ),
        ];
        entries.extend(wh_leadership_journey_nofl());
        entries.extend(wh_fundamentals_nofl());
        entries
    });

    append_semester(&mut requirements, &mut schedule_hints, Y3F, bb_reqs);
    append_semester(
        &mut requirements,
        &mut schedule_hints,
        Y2S,
        std::iter::once(wh_ssh_las_pool())
            .chain(wh_unrestricted_electives(5))
            .collect(),
    );
    append_semester(&mut requirements, &mut schedule_hints, Y3F, wh_concentration_requirements(&concs));

    apply_wh_fixed_hints(&mut schedule_hints, false);

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

    let mut mt_fundamentals = wh_fundamentals_mt();
    mt_fundamentals.push((Y2S, mt_mgmt2370_soph()));
    mt_fundamentals.push((
        Y1F,
        single("M&T Freshman Course", &["OIDD 2340"]),
    ));

    let (mut requirements, mut schedule_hints) = scheduled({
        let mut entries = vec![
            (Y1F, wh_first_year_foundations()),
            (Y1F, single("First-Year Foundations", &["MATH 1400"])),
            (Y1S, single("First-Year Foundations", &["MATH 1410"])),
            (
                Y1S,
                restriction(1)
                    .category("First-Year Foundations")
                    .departments(&["WRIT"])
                    .into(),
            ),
        ];
        entries.extend(wh_leadership_journey_mt());
        entries.extend(mt_fundamentals);
        entries
    });

    append_semester(&mut requirements, &mut schedule_hints, Y3F, extra_bb);
    append_semester(
        &mut requirements,
        &mut schedule_hints,
        Y2S,
        wh_nofl_mt_las_standalone(),
    );
    append_semester(&mut requirements, &mut schedule_hints, Y3F, conc_reqs);

    apply_wh_fixed_hints(&mut schedule_hints, true);

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

    let mut mt_fundamentals = wh_fundamentals_mt();
    mt_fundamentals.push((Y2S, mt_mgmt2370_soph()));
    mt_fundamentals.push((
        Y1F,
        single("M&T Freshman Course", &["OIDD 2340"]),
    ));

    let (mut requirements, mut schedule_hints) = scheduled({
        let mut entries = vec![
            (Y1F, wh_first_year_foundations()),
            (Y1F, single("First-Year Foundations", &["MATH 1400"])),
            (Y1S, single("First-Year Foundations", &["MATH 1410"])),
            (
                Y1F,
                restriction(1)
                    .category("First-Year Foundations")
                    .departments(&["WRIT"])
                    .into(),
            ),
        ];
        entries.extend(wh_leadership_journey_mt());
        entries.extend(mt_fundamentals);
        entries
    });

    append_semester(&mut requirements, &mut schedule_hints, Y3F, extra_bb);
    append_semester(
        &mut requirements,
        &mut schedule_hints,
        Y2S,
        vec![wh_fl_mt_las_pool()],
    );
    append_semester(&mut requirements, &mut schedule_hints, Y3F, conc_reqs);

    apply_wh_fixed_hints(&mut schedule_hints, true);

    Major {
        short_name: "WH".to_string(),
        name: "M&T - Foreign Language Required".to_string(),
        requirements,
        schedule_hints,
        concentrations: Some(wh_concentrations),
    }
}

